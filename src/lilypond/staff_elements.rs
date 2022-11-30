use tera::{Context, Tera};

/// Intentionally triple-indented. Intended to go in a staff (which is inside a score).
const VOICING: &str = r#"
            {% if duration != 0 -%}
                < {% for note in ly_notes %}{{ note }} {% endfor %}>{{ ly_duration }}
            {%- else -%}
                < {% for note in ly_notes %}{{ note }} {% endfor %}>
            {%- endif %}
"#;
pub fn voicing(
    ly_notes: Vec<String>,
    duration: usize,
    ly_duration: Option<String>,
) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("voicing", VOICING).unwrap();
    let mut ctx = Context::new();
    ctx.insert("ly_notes", &ly_notes);
    ctx.insert("duration", &duration);
    ctx.insert("ly_duration", &ly_duration);
    tera.render("voicing", &ctx).unwrap()
}
