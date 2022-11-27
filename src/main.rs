//#![feature(concat_idents)]

// println!("{}", -1i32 % 4);                // -1
// println!("{}", (-21i32).rem_euclid(4));   // 3

// Note("C") == Pitch("c3") // true

// Problem with this is that it allows invalid state.
// Better to implement From and Into on classes like Note and Pitch for a Pc class
// trait Pc {
//     fn pc(&self) -> u8;
// }

pub mod chord;
mod lilypond;
pub mod note;
mod pitch;
mod rhythm;
pub mod fretboard;

fn main() {
    println!("Hello, world!");
}
