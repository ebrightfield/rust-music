use once_cell::sync::Lazy;
use tera::Tera;

pub static TEMPLATE_ENGINE: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template("staff", G_8_STAFF).unwrap();
    tera.add_raw_template("tab_staff", TAB_STAFF).unwrap();
    tera.add_raw_template("score", SCORE).unwrap();
    tera.add_raw_template("voicing", VOICING).unwrap();
    tera.add_raw_template("voicing_tab", VOICING).unwrap();
    tera.add_raw_template("fretboard_diagram", FRET_DIAGRAM).unwrap();
    tera
});

/// The main top-level element of a lilypond document.
const SCORE: &str = r#"
\score {
    {{ content }}
    {{ format_block }}
}
"#;

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

/// Intentionally triple-indented. Intended to go in a staff (which is inside a score).
const VOICING: &str = r#"
            {% if duration != 0 -%}
                < {% for note in ly_notes %}{{ note }} {% endfor %}>{{ ly_duration }}
            {%- else -%}
                < {% for note in ly_notes %}{{ note }} {% endfor %}>
            {%- endif %}
"#;

/// A voicing for tablature, specifying string numbers in addition to pitch / duration.
const VOICING_TAB: &str = r#"
{% if ly_duration != 0 -%}
    < {% for note in ly_notes %}{{ note[0] }}\{{ note[1]  }} {% endfor %}>{{ ly_duration }}
{%- else -%}
    < {% for note in ly_notes %}{{ note[0] }}\{{ note[1] }} {% endfor %}>
{%- endif %}
"#;

/// A staff group where the top has a voicing,
/// and the bottom has some number of fingerings to match it.
const VOICING_AND_TAB_FINGERINGS: &str = r#"
\score {
    \new StaffGroup
  <<
        \new Staff {
            \clef "G_8"
            \omit Staff.TimeSignature
            \omit Score.BarNumber
            {% include "ly/voicing" %}
        }
        \new TabStaff {
            \clef moderntab
            \omit Staff.TimeSignature
            \omit Score.BarNumber
            \tuplet {{ tuplet[0] }}/{{ tuplet[1] }} {
                {% for notes in tab_notes -%}
                    <{% for note in notes[0] -%}
                    {{ note }}\{{ notes[1][loop.index0] }} {% endfor %}>{{ tuplet[1] }}
                {% endfor %}
            }
        }
    >>
}

"#;

const FRET_DIAGRAM: &str = r#"
    \markup {
        \override #'(fret-diagram-details . (
        (finger-code . below-string)
        (number-type . arabic)
        (label-dir . -1)
        (mute-string . "x")
        (orientation . landscape)
        (fret-count . 6)
        (xo-font-magnification . 0.4)
        (xo-padding . 0.3)))
      \fret-diagram-verbose #'(
        {%- for fret in frets %}
            {%- if fret.fret == '255' %}
                (mute {{ fret.string }})
            {%- elif fret.fret == 0 %}
                (open {{ fret.string }})
            {%- else %}
                (place-fret {{ fret.string }} {{ fret.fret }})
            {%- endif %}
        {%- endfor %}
        )
    }
"#;