use once_cell::sync::Lazy;
use tera::Tera;

pub static TEMPLATE_ENGINE: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template("staff", G_8_STAFF).unwrap();
    tera.add_raw_template("tab_staff", TAB_STAFF).unwrap();
    tera
});

// TODO Configuring the layout block more hands-on and completely.
/// Layout block which goes at the bottom of a score,
/// and sets the ragged right property to false.
pub const RAGGED_RIGHT: &str = r#"
    \layout {
        \omit Voice.StringNumber
        ragged-right = ##f
    }
"#;

/// Placed in a staff to hide the time signature.
pub const OMIT_TIME_SIGNATURE: &str = "\\omit Staff.TimeSignature";

/// Intentional double indent here.
const G_8_STAFF: &str = r#"
        \new Staff {
            \clef G_8
            {{ time_signature }}
            \omit Score.BarNumber

            {{ content }}
        }
"#;

/// Intentional double indent here.
const TAB_STAFF: &str = r#"
        \new TabStaff {
            \clef moderntab
            {{ time_signature }}
            \omit Score.BarNumber

            {{ content }}
        }
"#;
