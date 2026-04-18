/// Bravura OTF font bytes, bundled at compile time.
pub static BRAVURA_OTF: &[u8] = include_bytes!("../../fonts/Bravura.otf");

/// Bravura font metadata JSON, bundled at compile time.
pub static BRAVURA_METADATA: &[u8] = include_bytes!("../../fonts/bravura_metadata.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bravura_otf_is_valid_opentype() {
        // OTF files start with the "OTTO" magic bytes for CFF-based OpenType
        assert_eq!(&BRAVURA_OTF[..4], b"OTTO", "Expected CFF-based OpenType magic");
        assert!(BRAVURA_OTF.len() > 100_000, "Font file suspiciously small");
    }

    #[test]
    fn bravura_otf_parseable_by_ttf_parser() {
        let face = ttf_parser::Face::parse(BRAVURA_OTF, 0)
            .expect("ttf-parser should parse Bravura.otf");
        assert!(face.units_per_em() > 0);
        assert!(face.number_of_glyphs() > 2000, "Bravura should have thousands of glyphs");
    }

    #[test]
    fn bravura_metadata_is_valid_json() {
        let value: serde_json::Value = serde_json::from_slice(BRAVURA_METADATA)
            .expect("metadata should be valid JSON");
        assert!(value.is_object());
        // Bravura metadata should have engravingDefaults
        assert!(
            value.get("engravingDefaults").is_some(),
            "metadata should contain engravingDefaults"
        );
    }
}
