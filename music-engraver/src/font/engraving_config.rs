use smufl::StaffSpaces;

/// Resolved engraving constants for a SMuFL font, with all values guaranteed present.
///
/// Values are in **staff spaces** (1 staff space = ¼ of the distance between
/// the outer staff lines in a 5-line staff). To convert to font design units,
/// multiply by `staff_space_in_font_units` (typically `units_per_em / 4`).
///
/// Fallback values come from the SMuFL specification's recommended defaults
/// and match Bravura's metadata where unspecified.
#[derive(Clone, Debug, PartialEq)]
pub struct EngravingConfig {
    pub staff_line_thickness: f64,
    pub stem_thickness: f64,
    pub beam_thickness: f64,
    pub beam_spacing: f64,
    pub leger_line_thickness: f64,
    pub leger_line_extension: f64,
    pub thin_barline_thickness: f64,
    pub thick_barline_thickness: f64,
    pub barline_separation: f64,
    pub slur_endpoint_thickness: f64,
    pub slur_midpoint_thickness: f64,
    pub tie_endpoint_thickness: f64,
    pub tie_midpoint_thickness: f64,
    pub tuplet_bracket_thickness: f64,
    pub hairpin_thickness: f64,
    pub repeat_barline_dot_separation: f64,
    pub bracket_thickness: f64,
    pub sub_bracket_thickness: f64,
    pub dashed_barline_thickness: f64,
    pub dashed_barline_dash_length: f64,
    pub dashed_barline_gap_length: f64,
    pub repeat_ending_line_thickness: f64,
    /// How many font design units equal one staff space.
    /// Typically `units_per_em / 4`.
    pub staff_space: f64,
}

impl EngravingConfig {
    /// Build from the smufl metadata's engraving defaults, filling in
    /// SMuFL-recommended fallbacks for any missing values.
    pub fn from_smufl(defaults: &smufl::EngravingDefaults, units_per_em: u16) -> Self {
        let ss = units_per_em as f64 / 4.0;

        Self {
            staff_line_thickness: unwrap_ss(defaults.staff_line_thickness, 0.13),
            stem_thickness: unwrap_ss(defaults.stem_thickness, 0.12),
            beam_thickness: unwrap_ss(defaults.beam_thickness, 0.5),
            beam_spacing: unwrap_ss(defaults.beam_spacing, 0.25),
            leger_line_thickness: unwrap_ss(defaults.leger_line_thickness, 0.16),
            leger_line_extension: unwrap_ss(defaults.leger_line_extension, 0.4),
            thin_barline_thickness: unwrap_ss(defaults.thin_barline_thickness, 0.16),
            thick_barline_thickness: unwrap_ss(defaults.thick_barline_thickness, 0.5),
            barline_separation: unwrap_ss(defaults.barline_separation, 0.4),
            slur_endpoint_thickness: unwrap_ss(defaults.slur_endpoint_thickness, 0.1),
            slur_midpoint_thickness: unwrap_ss(defaults.slur_midpoint_thickness, 0.22),
            tie_endpoint_thickness: unwrap_ss(defaults.tie_endpoint_thickness, 0.1),
            tie_midpoint_thickness: unwrap_ss(defaults.tie_midpoint_thickness, 0.22),
            tuplet_bracket_thickness: unwrap_ss(defaults.tuplet_bracket_thickness, 0.16),
            hairpin_thickness: unwrap_ss(defaults.hairpin_thickness, 0.16),
            repeat_barline_dot_separation: unwrap_ss(defaults.repeat_barline_dot_separation, 0.16),
            bracket_thickness: unwrap_ss(defaults.bracket_thickness, 0.5),
            sub_bracket_thickness: unwrap_ss(defaults.sub_bracket_thickness, 0.16),
            dashed_barline_thickness: unwrap_ss(defaults.dashed_barline_thickness, 0.16),
            dashed_barline_dash_length: unwrap_ss(defaults.dashed_barline_dash_length, 0.5),
            dashed_barline_gap_length: unwrap_ss(defaults.dashed_barline_gap_length, 0.25),
            repeat_ending_line_thickness: unwrap_ss(defaults.repeat_ending_line_thickness, 0.16),
            staff_space: ss,
        }
    }

    /// Convert a value in staff spaces to font design units.
    pub fn to_font_units(&self, staff_spaces: f64) -> f64 {
        staff_spaces * self.staff_space
    }

    /// Staff line thickness in font design units.
    pub fn staff_line_thickness_fu(&self) -> f64 {
        self.to_font_units(self.staff_line_thickness)
    }

    /// Stem thickness in font design units.
    pub fn stem_thickness_fu(&self) -> f64 {
        self.to_font_units(self.stem_thickness)
    }

    /// Beam thickness in font design units.
    pub fn beam_thickness_fu(&self) -> f64 {
        self.to_font_units(self.beam_thickness)
    }

    /// Beam spacing in font design units.
    pub fn beam_spacing_fu(&self) -> f64 {
        self.to_font_units(self.beam_spacing)
    }

    /// Leger line thickness in font design units.
    pub fn leger_line_thickness_fu(&self) -> f64 {
        self.to_font_units(self.leger_line_thickness)
    }

    /// Leger line extension in font design units.
    pub fn leger_line_extension_fu(&self) -> f64 {
        self.to_font_units(self.leger_line_extension)
    }

    /// Thin barline thickness in font design units.
    pub fn thin_barline_thickness_fu(&self) -> f64 {
        self.to_font_units(self.thin_barline_thickness)
    }

    /// Thick barline thickness in font design units.
    pub fn thick_barline_thickness_fu(&self) -> f64 {
        self.to_font_units(self.thick_barline_thickness)
    }
}

/// Extract the inner `f64` from an `Option<StaffSpaces>`, falling back to
/// a default value in staff spaces.
fn unwrap_ss(opt: Option<StaffSpaces>, fallback: f64) -> f64 {
    opt.map(|s| s.0).unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bravura_config() -> EngravingConfig {
        let metadata: smufl::Metadata =
            serde_json::from_slice(crate::font::BRAVURA_METADATA).unwrap();
        EngravingConfig::from_smufl(&metadata.engraving_defaults, 1000)
    }

    #[test]
    fn bravura_staff_space_is_250() {
        let cfg = bravura_config();
        assert!((cfg.staff_space - 250.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bravura_staff_line_thickness() {
        let cfg = bravura_config();
        // Bravura metadata specifies 0.13 staff spaces
        assert!((cfg.staff_line_thickness - 0.13).abs() < 1e-6);
        // In font units: 0.13 * 250 = 32.5
        assert!((cfg.staff_line_thickness_fu() - 32.5).abs() < 1e-6);
    }

    #[test]
    fn bravura_stem_thickness() {
        let cfg = bravura_config();
        assert!((cfg.stem_thickness - 0.12).abs() < 1e-6);
        // 0.12 * 250 = 30
        assert!((cfg.stem_thickness_fu() - 30.0).abs() < 1e-6);
    }

    #[test]
    fn bravura_beam_thickness() {
        let cfg = bravura_config();
        assert!((cfg.beam_thickness - 0.5).abs() < 1e-6);
        // 0.5 * 250 = 125
        assert!((cfg.beam_thickness_fu() - 125.0).abs() < 1e-6);
    }

    #[test]
    fn bravura_beam_spacing() {
        let cfg = bravura_config();
        assert!((cfg.beam_spacing - 0.25).abs() < 1e-6);
        // 0.25 * 250 = 62.5
        assert!((cfg.beam_spacing_fu() - 62.5).abs() < 1e-6);
    }

    #[test]
    fn bravura_leger_line_extension() {
        let cfg = bravura_config();
        assert!((cfg.leger_line_extension - 0.4).abs() < 1e-6);
        assert!((cfg.leger_line_extension_fu() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn bravura_barline_thicknesses() {
        let cfg = bravura_config();
        assert!((cfg.thin_barline_thickness - 0.16).abs() < 1e-6);
        assert!((cfg.thick_barline_thickness - 0.5).abs() < 1e-6);
        assert!((cfg.thin_barline_thickness_fu() - 40.0).abs() < 1e-6);
        assert!((cfg.thick_barline_thickness_fu() - 125.0).abs() < 1e-6);
    }

    #[test]
    fn to_font_units_conversion() {
        let cfg = bravura_config();
        // 1.0 staff space = 250 font units for Bravura
        assert!((cfg.to_font_units(1.0) - 250.0).abs() < f64::EPSILON);
        assert!((cfg.to_font_units(4.0) - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fallback_used_when_metadata_is_none() {
        let empty = smufl::EngravingDefaults::default();
        let cfg = EngravingConfig::from_smufl(&empty, 1000);
        // All values should be the fallback defaults
        assert!((cfg.staff_line_thickness - 0.13).abs() < 1e-6);
        assert!((cfg.stem_thickness - 0.12).abs() < 1e-6);
        assert!((cfg.beam_thickness - 0.5).abs() < 1e-6);
    }
}
