// REQ-O32
// pc_shape! with an argument >= 12 must NOT compile.
// The const assertion in the macro fires for values >= 12.

use music::pc_shape;

fn main() {
    let _ = pc_shape!(0, 4, 15);   // must NOT compile: 15 >= 12
}
