// REQ-O14
// Modes is implemented for PcShape only, NOT for PcContent.
// Calling .modes() on a PcContent must NOT compile.

use music::note_collections::pc_set::PcContent;
use music::note_collections::geometry::symmetry::transpositional::Modes;

fn main() {
    let c = PcContent::new(vec![]);
    let _ = c.modes();   // must NOT compile: Modes not implemented for PcContent
}
