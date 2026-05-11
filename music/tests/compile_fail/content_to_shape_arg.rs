// REQ-X3, §6.5 acceptance bullet
// Passing &PcContent where &PcShape is expected must NOT compile.
// Caller must call c.to_shape() first.

use music::note_collections::pc_set::{PcShape, PcContent};

fn takes_shape(_s: &PcShape) {}

fn main() {
    let c = PcContent::new(vec![]);
    takes_shape(&c);   // must NOT compile; caller must call c.to_shape() first
}
