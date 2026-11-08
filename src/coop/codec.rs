//! Full animation poses compress well. Bound both wire bytes and decoded bytes;
//! a peer cannot turn a small packet into an unbounded allocation.
use super::transport::MAX_STATE;
use anyhow::{ensure, Result};
use flate2::{write::ZlibEncoder, Compression, Decompress, FlushDecompress, Status};
use std::{borrow::Cow, io::Write};

const MAGIC: &[u8; 5] = b"LGCF1";
const HEADER: usize = 9;

pub fn encode(bytes: &[u8]) -> Result<Vec<u8>> {
    ensure!(
        !bytes.is_empty() && bytes.len() <= MAX_STATE,
        "Co-op state exceeds its decoded limit"
    );
    let mut output = MAGIC.to_vec();
    output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    let mut encoder = ZlibEncoder::new(output, Compression::fast());
    encoder.write_all(bytes)?;
    let output = encoder.finish()?;
    ensure!(
        output.len() <= MAX_STATE,
        "Compressed co-op state exceeds its wire limit"
    );
    Ok(output)
}
pub fn decode(bytes: &[u8]) -> Result<Cow<'_, [u8]>> {
    ensure!(bytes.len() <= MAX_STATE, "Co-op packet exceeds its limit");
    if !bytes.starts_with(MAGIC) {
        return Ok(Cow::Borrowed(bytes));
    }
    ensure!(bytes.len() >= HEADER, "Incomplete co-op state header");
    let size = u32::from_le_bytes(bytes[5..HEADER].try_into().unwrap()) as usize;
    ensure!(size > 0 && size <= MAX_STATE, "Invalid co-op state size");
    let mut decoder = Decompress::new(true);
    let mut output = vec![0; size + 1];
    let status = decoder.decompress(&bytes[HEADER..], &mut output, FlushDecompress::Finish)?;
    ensure!(
        status == Status::StreamEnd
            && decoder.total_out() as usize == size
            && decoder.total_in() as usize == bytes.len() - HEADER,
        "Incomplete or oversized compressed co-op state"
    );
    output.truncate(size);
    Ok(Cow::Owned(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_compression_roundtrips_and_rejects_bombs_truncation_and_trailing_bytes() {
        let state = b"{\"pose\":[0,0,0,1,0,0,0]}".repeat(8000);
        let packet = encode(&state).unwrap();
        assert!(packet.len() < state.len() / 10);
        assert_eq!(decode(&packet).unwrap().as_ref(), state);
        assert!(decode(&packet[..packet.len() - 2]).is_err());
        let mut small = packet.clone();
        small[5..HEADER].copy_from_slice(&10u32.to_le_bytes());
        assert!(decode(&small).is_err());
        let mut huge = packet.clone();
        huge[5..HEADER].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&huge).is_err());
        let mut trailing = packet;
        trailing.push(0);
        assert!(decode(&trailing).is_err());
        assert_eq!(
            decode(b"{\"type\":\"Hello\"}").unwrap(),
            b"{\"type\":\"Hello\"}".as_slice()
        );
    }
}
