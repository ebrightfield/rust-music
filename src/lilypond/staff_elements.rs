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