use tera::Context;
use serde::Serialize;
use crate::lilypond::templates::TEMPLATE_ENGINE;

#[derive(Debug, Serialize)]
pub struct DiagramFret {
    string: u8,
    fret: u8,
}

impl From<(u8, u8)> for DiagramFret {
    fn from(value: (u8, u8)) -> Self {
        Self {
            string: value.0,
            fret: value.1,
        }
    }
}

pub fn fretboard_diagram(frets: Vec<DiagramFret>) -> String {
    let mut ctx = Context::new();
    ctx.insert("frets", &frets);
    (*TEMPLATE_ENGINE).render("fretboard_diagram", &ctx).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::lilypond::fretboard_diagram::fretboard_diagram;
    use crate::lilypond::staff_elements::*;

    #[test]
    fn fretboard_diagram_works() {
        let frets = vec![
            (6, 5).into(),
            (5, 7).into(),
            (4, 7).into(),
            (3, 6).into(),
            (2, 5).into(),
            (1, 5).into(),
        ];

        let result = fretboard_diagram(frets);
        println!("{}", result);
    }
}
