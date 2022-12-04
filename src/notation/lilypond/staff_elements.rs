use tera::Context;
use crate::lilypond::templates::TEMPLATE_ENGINE;

pub fn voicing(
    ly_notes: Vec<String>,
    duration: usize,
    ly_duration: Option<String>,
) -> String {
    let mut ctx = Context::new();
    ctx.insert("ly_notes", &ly_notes);
    ctx.insert("duration", &duration);
    ctx.insert("ly_duration", &ly_duration);
    (*TEMPLATE_ENGINE).render("voicing", &ctx).unwrap()
}

pub trait ToLilypondString {
    fn to_lilypond_string(&self) -> String;
}

// Does this work? Some collections are not tab. Oh but we can disambiguate? No,
// need to specify it for concrete vec types.
// impl ToLilypondString for Vec<T: ToLilypondString> {
//     fn to_lilypond_string(&self) -> String {
//         todo!()
//     }
// }