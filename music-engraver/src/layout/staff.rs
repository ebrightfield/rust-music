use crate::font::EngravingConfig;

/// Number of lines in a standard staff.
pub const STANDARD_LINE_COUNT: u8 = 5;

/// A staff position identifies a vertical location on or near a staff.
///
/// Position 0 is the bottom line of a 5-line staff. Each increment moves
/// up by one half-space: position 1 is the first space, position 2 is the
/// second line, and so on up to position 8 (top line).
///
/// Positions outside 0–8 represent ledger-line territory: negative values
/// are below the staff, values above 8 are above it.
pub type StaffPosition = i8;

/// Bottom line of a standard 5-line staff.
pub const BOTTOM_LINE: StaffPosition = 0;
/// Top line of a standard 5-line staff.
pub const TOP_LINE: StaffPosition = 8;

/// Geometric model for a 5-line staff.
///
/// The coordinate system uses SVG conventions: y increases downward.
/// The staff's origin is at its top-left corner (top line, left edge).
/// `y_origin` is the y-coordinate of the top line in the enclosing canvas.
#[derive(Clone, Debug)]
pub struct StaffLayout {
    /// X-coordinate of the left edge.
    pub x: f64,
    /// Y-coordinate of the top staff line in the enclosing canvas.
    pub y_origin: f64,
    /// Total width of the staff lines.
    pub width: f64,
    /// Distance between adjacent staff lines, in font design units.
    pub staff_space: f64,
}

impl StaffLayout {
    /// Create a staff layout with explicit geometry.
    pub fn new(x: f64, y_origin: f64, width: f64, staff_space: f64) -> Self {
        Self {
            x,
            y_origin,
            width,
            staff_space,
        }
    }

    /// Build from an `EngravingConfig`, positioning at a given origin and width.
    pub fn from_config(x: f64, y_origin: f64, width: f64, config: &EngravingConfig) -> Self {
        Self::new(x, y_origin, width, config.staff_space)
    }

    /// Half the distance between adjacent lines.
    pub fn half_space(&self) -> f64 {
        self.staff_space / 2.0
    }

    /// Total height from top line to bottom line (4 staff spaces for 5 lines).
    pub fn height(&self) -> f64 {
        self.staff_space * (STANDARD_LINE_COUNT - 1) as f64
    }

    /// Convert a staff position to a y-coordinate.
    ///
    /// Position 8 (top line) maps to `y_origin`.
    /// Position 0 (bottom line) maps to `y_origin + 4 * staff_space`.
    /// Positions outside 0–8 extrapolate linearly into ledger-line territory.
    pub fn y_of(&self, position: StaffPosition) -> f64 {
        self.y_origin + (TOP_LINE - position) as f64 * self.half_space()
    }

    /// The y-coordinates of the 5 staff lines, from top to bottom.
    pub fn line_ys(&self) -> [f64; STANDARD_LINE_COUNT as usize] {
        let mut ys = [0.0; STANDARD_LINE_COUNT as usize];
        for (i, y) in ys.iter_mut().enumerate() {
            // Line 0 in the array = top line (staff position 8)
            // Line 4 in the array = bottom line (staff position 0)
            *y = self.y_origin + i as f64 * self.staff_space;
        }
        ys
    }

    /// Returns true if the position sits on a staff line (even positions 0–8).
    pub fn is_on_line(position: StaffPosition) -> bool {
        position % 2 == 0 && (BOTTOM_LINE..=TOP_LINE).contains(&position)
    }

    /// Returns true if the position is in a staff space (odd positions 1–7).
    pub fn is_in_space(position: StaffPosition) -> bool {
        position % 2 != 0 && position > BOTTOM_LINE && position < TOP_LINE
    }

    /// Returns true if the position requires ledger lines.
    pub fn needs_ledger_lines(position: StaffPosition) -> bool {
        !(BOTTOM_LINE..=TOP_LINE).contains(&position)
    }

    /// How many ledger lines are needed for a given position.
    /// Returns 0 if the position is on or within the staff, or in the
    /// first space immediately outside it (positions -1, 9).
    pub fn ledger_line_count(position: StaffPosition) -> u8 {
        if position <= -2 {
            // Below: -2 → 1, -3 → 1, -4 → 2, -5 → 2, …
            ((-position) / 2) as u8
        } else if position >= 10 {
            // Above: 10 → 1, 11 → 1, 12 → 2, …
            ((position - 8) / 2) as u8
        } else {
            0
        }
    }

    /// Y-coordinates of ledger lines needed for a given staff position.
    /// Empty if no ledger lines are needed.
    pub fn ledger_line_ys(&self, position: StaffPosition) -> Vec<f64> {
        let mut ys = Vec::new();
        if position <= -2 {
            // Ledger lines below: at positions -2, -4, -6, ...
            let mut p = -2;
            while p >= position {
                ys.push(self.y_of(p));
                p -= 2;
            }
        } else if position >= 10 {
            // Ledger lines above: at positions 10, 12, 14, ...
            let mut p = 10;
            while p <= position {
                ys.push(self.y_of(p));
                p += 2;
            }
        }
        ys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staff() -> StaffLayout {
        // Bravura: 1000 UPM, staff_space = 250
        StaffLayout::new(0.0, 0.0, 5000.0, 250.0)
    }

    #[test]
    fn height_is_four_staff_spaces() {
        let s = test_staff();
        assert!((s.height() - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn half_space_is_125() {
        let s = test_staff();
        assert!((s.half_space() - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn top_line_y_is_origin() {
        let s = test_staff();
        assert!((s.y_of(TOP_LINE) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bottom_line_y_is_four_spaces_below_origin() {
        let s = test_staff();
        assert!((s.y_of(BOTTOM_LINE) - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn middle_line_y_is_two_spaces_below_origin() {
        let s = test_staff();
        // Middle line = staff position 4, y = (8-4) * 125 = 500
        assert!((s.y_of(4) - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn line_ys_top_to_bottom() {
        let s = test_staff();
        let ys = s.line_ys();
        assert_eq!(ys.len(), 5);
        assert!((ys[0] - 0.0).abs() < f64::EPSILON);   // top
        assert!((ys[1] - 250.0).abs() < f64::EPSILON);
        assert!((ys[2] - 500.0).abs() < f64::EPSILON);  // middle
        assert!((ys[3] - 750.0).abs() < f64::EPSILON);
        assert!((ys[4] - 1000.0).abs() < f64::EPSILON); // bottom
    }

    #[test]
    fn y_with_nonzero_origin() {
        let s = StaffLayout::new(0.0, 200.0, 5000.0, 250.0);
        assert!((s.y_of(TOP_LINE) - 200.0).abs() < f64::EPSILON);
        assert!((s.y_of(BOTTOM_LINE) - 1200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn is_on_line_checks() {
        assert!(StaffLayout::is_on_line(0));  // bottom
        assert!(StaffLayout::is_on_line(2));
        assert!(StaffLayout::is_on_line(4));  // middle
        assert!(StaffLayout::is_on_line(6));
        assert!(StaffLayout::is_on_line(8));  // top
        assert!(!StaffLayout::is_on_line(1)); // space
        assert!(!StaffLayout::is_on_line(3));
        assert!(!StaffLayout::is_on_line(-2)); // ledger line — not ON staff
        assert!(!StaffLayout::is_on_line(10));
    }

    #[test]
    fn is_in_space_checks() {
        assert!(StaffLayout::is_in_space(1));
        assert!(StaffLayout::is_in_space(3));
        assert!(StaffLayout::is_in_space(5));
        assert!(StaffLayout::is_in_space(7));
        assert!(!StaffLayout::is_in_space(0));
        assert!(!StaffLayout::is_in_space(8));
        assert!(!StaffLayout::is_in_space(9)); // above staff
    }

    #[test]
    fn needs_ledger_lines_checks() {
        assert!(!StaffLayout::needs_ledger_lines(0));
        assert!(!StaffLayout::needs_ledger_lines(4));
        assert!(!StaffLayout::needs_ledger_lines(8));
        assert!(StaffLayout::needs_ledger_lines(-1));
        assert!(StaffLayout::needs_ledger_lines(-2));
        assert!(StaffLayout::needs_ledger_lines(9));
        assert!(StaffLayout::needs_ledger_lines(10));
    }

    #[test]
    fn ledger_line_count_within_staff() {
        assert_eq!(StaffLayout::ledger_line_count(0), 0);
        assert_eq!(StaffLayout::ledger_line_count(4), 0);
        assert_eq!(StaffLayout::ledger_line_count(8), 0);
    }

    #[test]
    fn ledger_line_count_below_staff() {
        // Position -1 is the space just below — no ledger line needed
        assert_eq!(StaffLayout::ledger_line_count(-1), 0);
        // Middle C in treble clef = position -2: one ledger line
        assert_eq!(StaffLayout::ledger_line_count(-2), 1);
        // Position -3: space below first ledger line, still just 1 ledger line (at -2)
        assert_eq!(StaffLayout::ledger_line_count(-3), 1);
        // Position -4: two ledger lines (at -2 and -4)
        assert_eq!(StaffLayout::ledger_line_count(-4), 2);
        // Position -5: two ledger lines
        assert_eq!(StaffLayout::ledger_line_count(-5), 2);
        // Position -6: three ledger lines
        assert_eq!(StaffLayout::ledger_line_count(-6), 3);
    }

    #[test]
    fn ledger_line_count_above_staff() {
        // Position 9 is the space just above — no ledger line needed
        assert_eq!(StaffLayout::ledger_line_count(9), 0);
        assert_eq!(StaffLayout::ledger_line_count(10), 1);
        assert_eq!(StaffLayout::ledger_line_count(11), 1);
        assert_eq!(StaffLayout::ledger_line_count(12), 2);
        assert_eq!(StaffLayout::ledger_line_count(14), 3);
    }

    #[test]
    fn ledger_line_ys_below() {
        let s = test_staff();
        // Position -2: one ledger line at y_of(-2)
        let ys = s.ledger_line_ys(-2);
        assert_eq!(ys.len(), 1);
        // y_of(-2) = (8 - (-2)) * 125 = 10 * 125 = 1250
        assert!((ys[0] - 1250.0).abs() < f64::EPSILON);

        // Position -4: two ledger lines
        let ys = s.ledger_line_ys(-4);
        assert_eq!(ys.len(), 2);
        assert!((ys[0] - 1250.0).abs() < f64::EPSILON); // at -2
        assert!((ys[1] - 1500.0).abs() < f64::EPSILON); // at -4
    }

    #[test]
    fn ledger_line_ys_above() {
        let s = test_staff();
        let ys = s.ledger_line_ys(10);
        assert_eq!(ys.len(), 1);
        // y_of(10) = (8 - 10) * 125 = -250
        assert!((ys[0] - -250.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ledger_line_ys_within_staff_is_empty() {
        let s = test_staff();
        assert!(s.ledger_line_ys(0).is_empty());
        assert!(s.ledger_line_ys(4).is_empty());
        assert!(s.ledger_line_ys(8).is_empty());
        assert!(s.ledger_line_ys(5).is_empty());
    }

    #[test]
    fn positions_below_staff_extrapolate() {
        let s = test_staff();
        // Position -1: just below bottom line
        // y = (8 - (-1)) * 125 = 9 * 125 = 1125
        assert!((s.y_of(-1) - 1125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn positions_above_staff_extrapolate() {
        let s = test_staff();
        // Position 9: just above top line
        // y = (8 - 9) * 125 = -125
        assert!((s.y_of(9) - -125.0).abs() < f64::EPSILON);
    }
}
