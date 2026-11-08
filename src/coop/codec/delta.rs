//! Exact byte deltas for ordered snapshots. Saves and game-state schemas stay
//! unchanged. Full packets remain the fallback for first/poorly matching frames.
use super::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const MAGIC: &[u8; 5] = b"LGCD1";
const HEADER: usize = 41; // magic, SHA-256 of base, reconstructed byte length
const BLOCK: usize = 64;

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0_u64, |h, &b| h.wrapping_mul(257).wrapping_add(u64::from(b)))
}
fn insert(patch: &mut Vec<u8>, bytes: &[u8]) {
    if bytes.is_empty() { return; }
    patch.push(1);
    patch.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    patch.extend_from_slice(bytes);
}

pub(super) fn encode(bytes: &[u8], base: Option<&[u8]>) -> Result<Vec<u8>> {
    let full = super::encode(bytes)?;
    let Some(base) = base.filter(|b| (BLOCK..=MAX_STATE).contains(&b.len())) else { return Ok(full); };
    let mut blocks = HashMap::new();
    for offset in (0..=base.len()-BLOCK).step_by(BLOCK) {
        blocks.entry(hash(&base[offset..offset+BLOCK])).or_insert(offset);
    }
    let power = 257_u64.wrapping_pow((BLOCK-1) as u32);
    let mut patch = Vec::new();
    let (mut at, mut literal) = (0, 0);
    let mut rolling = bytes.get(..BLOCK).map(hash).unwrap_or(0);
    while at+BLOCK <= bytes.len() {
        // Hash collisions cannot affect data: compare the actual bytes too.
        if let Some(&offset) = blocks.get(&rolling).filter(|&&offset|
            base[offset..offset+BLOCK] == bytes[at..at+BLOCK]) {
            insert(&mut patch, &bytes[literal..at]);
            let mut size = BLOCK;
            while offset+size < base.len() && at+size < bytes.len()
                && base[offset+size] == bytes[at+size] { size += 1; }
            patch.push(0);
            patch.extend_from_slice(&(offset as u32).to_le_bytes());
            patch.extend_from_slice(&(size as u32).to_le_bytes());
            at += size; literal = at;
            rolling = bytes.get(at..at+BLOCK).map(hash).unwrap_or(0);
        } else {
            if at+BLOCK < bytes.len() {
                rolling = rolling.wrapping_sub(u64::from(bytes[at]).wrapping_mul(power))
                    .wrapping_mul(257).wrapping_add(u64::from(bytes[at+BLOCK]));
            }
            at += 1;
        }
    }
    insert(&mut patch, &bytes[literal..]);
    if patch.len() > MAX_STATE { return Ok(full); }
    let compressed = super::encode(&patch)?;
    if HEADER+compressed.len() >= full.len() { return Ok(full); }
    let mut output = MAGIC.to_vec();
    output.extend_from_slice(&Sha256::digest(base));
    output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    output.extend_from_slice(&compressed);
    Ok(output)
}

fn number(bytes: &[u8], cursor: &mut usize) -> Result<usize> {
    let part = bytes.get(*cursor..*cursor+4).ok_or_else(|| anyhow::anyhow!("Truncated snapshot delta"))?;
    *cursor += 4;
    Ok(u32::from_le_bytes(part.try_into().unwrap()) as usize)
}

pub(super) fn decode<'a>(bytes: &'a [u8], base: Option<&[u8]>) -> Result<Cow<'a, [u8]>> {
    if !bytes.starts_with(MAGIC) { return super::decode(bytes); }
    ensure!(bytes.len() <= MAX_STATE && bytes.len() > HEADER, "Invalid snapshot delta size");
    let base = base.ok_or_else(|| anyhow::anyhow!("Snapshot delta has no preceding state"))?;
    ensure!(base.len() <= MAX_STATE && Sha256::digest(base)[..] == bytes[5..37], "Snapshot delta base differs");
    let size = u32::from_le_bytes(bytes[37..HEADER].try_into().unwrap()) as usize;
    ensure!(size > 0 && size <= MAX_STATE, "Snapshot delta exceeds its decoded limit");
    ensure!(bytes[HEADER..].starts_with(super::MAGIC), "Invalid snapshot delta encoding");
    let patch = super::decode(&bytes[HEADER..])?;
    let mut output = Vec::with_capacity(size);
    let mut cursor = 0;
    while cursor < patch.len() {
        let operation = patch[cursor]; cursor += 1;
        let source = match operation {
            0 => {
                let offset = number(&patch, &mut cursor)?;
                let length = number(&patch, &mut cursor)?;
                let end = offset.checked_add(length).ok_or_else(|| anyhow::anyhow!("Snapshot copy overflow"))?;
                base.get(offset..end).ok_or_else(|| anyhow::anyhow!("Snapshot copy exceeds its base"))?
            },
            1 => {
                let length = number(&patch, &mut cursor)?;
                let end = cursor.checked_add(length).ok_or_else(|| anyhow::anyhow!("Snapshot insert overflow"))?;
                let source = patch.get(cursor..end).ok_or_else(|| anyhow::anyhow!("Truncated snapshot insert"))?;
                cursor = end; source
            },
            _ => anyhow::bail!("Unknown snapshot delta operation"),
        };
        ensure!(!source.is_empty() && source.len() <= size-output.len(), "Snapshot delta exceeds its declared size");
        output.extend_from_slice(source);
    }
    ensure!(output.len() == size, "Incomplete snapshot delta");
    Ok(Cow::Owned(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> Vec<u8> {
        let mut random = 7_u32;
        (0..100_000).map(|_| { random=random.wrapping_mul(1664525).wrapping_add(1013904223); (random>>24) as u8 }).collect()
    }
    #[test]
    fn shifted_and_changed_snapshots_reconstruct_exactly_with_less_wire_data() {
        let base = state();
        let mut next = base.clone();
        next.splice(1000..1005, b"a longer replacement".iter().copied());
        next.splice(40000..40500, []);
        next.extend_from_slice(b"a new effect");
        let packet = encode(&next, Some(&base)).unwrap();
        assert!(packet.starts_with(MAGIC));
        assert!(packet.len() < super::super::encode(&next).unwrap().len()/20);
        assert_eq!(decode(&packet, Some(&base)).unwrap().as_ref(), next);
        let third = encode(&base, Some(&next)).unwrap();
        assert_eq!(decode(&third, Some(&next)).unwrap().as_ref(), base);
        assert!(decode(&packet, None).is_err());
        assert!(decode(&packet, Some(&next)).is_err());
        for size in [0, 1, 63, 64, 127] {
            let data = &base[..size.max(1)];
            assert_eq!(decode(&encode(data, Some(&next)).unwrap(), Some(&next)).unwrap().as_ref(), data);
        }
    }
    #[test]
    fn malformed_deltas_cannot_escape_copy_insert_or_output_bounds() {
        let base = state();
        let packet = encode(&base, Some(&base)).unwrap();
        for end in 5..packet.len() { assert!(decode(&packet[..end], Some(&base)).is_err()); }
        let mut forged = packet.clone(); forged[37..HEADER].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&forged, Some(&base)).is_err());
        let mut overflow = vec![0];
        overflow.extend_from_slice(&u32::MAX.to_le_bytes()); overflow.extend_from_slice(&12_u32.to_le_bytes());
        for patch in [vec![2], vec![0], vec![1, 4, 0, 0, 0, 7], vec![0;9], overflow] {
            let mut forged = packet[..HEADER].to_vec();
            forged.extend_from_slice(&super::super::encode(&patch).unwrap());
            assert!(decode(&forged, Some(&base)).is_err());
        }
    }
    #[test]
    fn ordered_frames_handle_insertions_deletions_and_full_fallbacks() {
        let mut base = state();
        for frame in 0..80 {
            let mut next = base.clone();
            let at = (frame*1301) % (next.len()-128);
            match frame%4 {
                0 => { next.splice(at..at+65, [frame as u8; 3]); },
                1 => { next.splice(at..at, [frame as u8; 97]); },
                2 => next[at..at+128].fill(frame as u8),
                _ => next.iter_mut().for_each(|b| *b = b.wrapping_add(13)),
            }
            let packet = encode(&next, Some(&base)).unwrap();
            assert!(packet.len() <= super::super::encode(&next).unwrap().len());
            assert_eq!(decode(&packet, Some(&base)).unwrap().as_ref(), next);
            base = next;
        }
    }
}
