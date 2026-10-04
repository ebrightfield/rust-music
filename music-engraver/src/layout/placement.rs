//! Vertical side of the staff on which a mark is engraved.

/// Which side of the staff a mark sits on (LilyPond `^` / `_`,
/// `\dynamicUp` / `\dynamicDown`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Placement {
    /// Above the staff.
    Above,
    /// Below the staff (the default side for dynamics and hairpins).
    #[default]
    Below,
}
