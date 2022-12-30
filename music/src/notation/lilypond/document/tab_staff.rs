use crate::notation::rhythm::Meter;

pub struct LilypondTabStaff {
    modern_tab_clef: bool,
    time_signature: Option<Meter>,
    show_bar_numbers: bool,
    show_string_numbers: bool,
}
