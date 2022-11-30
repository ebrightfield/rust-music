use tera::{Context, Tera};

/// Wrap content in a markup block
pub fn markup(content: String) -> String {
    format!("\\markup {{\n    {}\n}}", content)
}

// TODO Configuring the layout block more hands-on and completely.
/// Layout block which goes at the bottom of a score,
/// and sets the ragged right property to false.
const RAGGED_RIGHT: &str = r#"
    \layout {
        \omit Voice.StringNumber
        ragged-right = ##f
    }
"#;

/// Intentional double indent here.
const G_8_STAFF: &str = r#"
        \new Staff {
            \clef G_8
            {{ time_signature }}
            \omit Score.BarNumber

            {{ content }}
        }}",
"#;

/// Intentional double indent here.
const TAB_STAFF: &str = r#"
        \new TabStaff {
            \clef moderntab
            {{ time_signature }}
            \omit Score.BarNumber

            {{ content }}
        }}",
"#;

/// Wrap content in a score block, optionally with ragged-right set to false.
pub fn score(content: String, ragged_right: bool) -> String {
    let mut score = format!("\\score {{\n    {}\n", content);
    if ragged_right {
        score.push_str(RAGGED_RIGHT);
    }
    score.push_str("}");
    score
}

const OMIT_TIME_SIGNATURE: &str = "\\omit Staff.TimeSignature";
/// Wrap content in a staff block
pub fn staff(content: String, time_signature: Option<String>) -> String {
    let time_sig = if let Some(t) = time_signature {
        format!("\\time {}\n", t)
    } else {
        OMIT_TIME_SIGNATURE.to_string()
    };
    let mut tera = Tera::default();
    tera.add_raw_template("staff", G_8_STAFF);
    let mut ctx = Context::new();
    ctx.insert("time_signature", &time_sig);
    ctx.insert("content", &content);
    tera.render("staff", &ctx).unwrap()
}

pub fn tab_staff(content: String, time_signature: Option<String>) -> String {
    let time_sig = if let Some(t) = time_signature {
        format!("\\time {}\n", t)
    } else {
        OMIT_TIME_SIGNATURE.to_string()
    };
    format!(
        "       \\new TabStaff {{\
            \\clef moderntab\
            {}\
            \\omit Score.BarNumber
            {}\
        }}",
        time_sig, content
    )
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
        let result = score(
            result,
            true
        );
        println!("{}", result);
    }
}