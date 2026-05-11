// REQ-O7, §6.5, §10 acceptance bullet "compile-fail test"
// PcShape and PcContent have no cross-type PartialEq — this must NOT compile.

use music::note_collections::pc_set::{PcShape, PcContent};

fn main() {
    let s = PcShape::new(vec![]);
    let c = PcContent::new(vec![]);
    let _ = s == c;   // must NOT compile
}
