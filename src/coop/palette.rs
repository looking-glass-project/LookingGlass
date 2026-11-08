//! Recolor only blue fabric in Alice's known clothing textures at load time.
//! Skin, apron, mouth, hair and user-supplied files remain untouched.
pub fn red_clothing(path: &str, pixels: &mut [u8]) {
    let lower = path.to_ascii_lowercase();
    let stem = lower
        .rsplit_once('.')
        .map_or(lower.as_str(), |(stem, _)| stem);
    if !matches!(
        stem,
        "models/alice/body"
            | "models/alice/body2"
            | "models/alice/bowstraps"
            | "models/alice/r_arm"
    ) {
        return;
    }
    for rgba in pixels.chunks_exact_mut(4) {
        let [r, g, b, _] = <[u8; 4]>::try_from(&*rgba).unwrap();
        if u16::from(b) > u16::from(r) + 12
            && u16::from(b) > u16::from(g) + 8
            && u16::from(b - r) * 5 > u16::from(b)
        {
            rgba[0] = b;
            rgba[1] = r;
            rgba[2] = g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fabric_changes_but_skin_apron_transparency_and_face_do_not() {
        let original = [60, 90, 170, 200, 210, 170, 140, 255, 235, 235, 240, 255];
        let mut cloth = original;
        red_clothing("models/alice/body.tga", &mut cloth);
        assert!(cloth[0] > cloth[2]);
        assert_eq!(cloth[3], 200);
        assert_eq!(&cloth[4..], &original[4..]);
        let mut packed = original;
        red_clothing("models/alice/body.ftx", &mut packed);
        assert_eq!(packed, cloth);
        let mut face = original;
        red_clothing("models/alice/body3.tga", &mut face);
        assert_eq!(face, original);
    }
}
