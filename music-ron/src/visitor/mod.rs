//! Custom serde visitors for shorthand string parsing.
//!
//! - [`pitch`] — `"c4"`, `"bes3"` → [`OwnedPitch`](crate::ast::common::OwnedPitch) (REQ-O12)
//! - [`duration`] — `"4"`, `"8."` → [`OwnedDuration`](crate::ast::common::OwnedDuration) (REQ-O14)
//! - [`bounded_vec`] — length-limited `Vec<T>` deserialization (REQ-O38)

pub(crate) mod bounded_vec;
pub(crate) mod duration;
pub(crate) mod pitch;
