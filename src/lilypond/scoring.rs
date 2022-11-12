/// Wrap content in a markup block
pub fn markup(content: String) -> String {
    format!("\\markup {{\n    {}\n}}", content)
}

/// Layout block which goes at the bottom of a score,
/// and sets the ragged right property to false.
const RAGGED_RIGHT: &str = r#"
    \layout {
        \omit Voice.StringNumber
        ragged-right = ##f
    }
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
    format!(
        "       \\new Staff {{\
            \\clef G_8\
            {}\
            \\omit Score.BarNumber
            {}\
        }}",
        time_sig, content
    )
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
