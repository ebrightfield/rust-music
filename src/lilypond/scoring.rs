use tera::Context;
use crate::lilypond::templates::{OMIT_TIME_SIGNATURE, RAGGED_RIGHT, TEMPLATE_ENGINE};

/// Wrap content in a markup block
pub fn markup(content: String) -> String {
    format!("\\markup {{\n    {}\n}}", content)
}

/// Wrap content in a score block, optionally with ragged-right set to false.
pub fn score(content: String, ragged_right: bool) -> String {
    let mut score = format!("\\score {{\n    {}\n", content);
    if ragged_right {
        score.push_str(RAGGED_RIGHT);
    }
    score.push_str("}");
    score
}

/// Wrap content in a staff block
pub fn staff(content: String, time_signature: Option<String>) -> String {
    let time_sig = if let Some(t) = time_signature {
        format!("\\time {}\n", t)
    } else {
        OMIT_TIME_SIGNATURE.to_string()
    };
    let mut ctx = Context::new();
    ctx.insert("time_signature", &time_sig);
    ctx.insert("content", &content);
    (*TEMPLATE_ENGINE).render("staff", &ctx).unwrap()
}

pub fn tab_staff(content: String, time_signature: Option<String>) -> String {
    let time_sig = if let Some(t) = time_signature {
        format!("\\time {}\n", t)
    } else {
        OMIT_TIME_SIGNATURE.to_string()
    };
    let mut ctx = Context::new();
    ctx.insert("time_signature", &time_sig);
    ctx.insert("content", &content);
    (*TEMPLATE_ENGINE).render("tab_staff", &ctx).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ly_tab_staff() {
        let result = staff(
            "c2 d e f g".to_string(),
            Some("3/4".to_string())
        );
        let _result = score(
            result,
            true
        );
        //println!("{}", result);
    }
}