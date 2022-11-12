/// {% if duration != 0 -%}
//     < {% for note in ly_notes %}{{ note }} {% endfor %}>{{ ly_duration }}
// {%- else -%}
//     < {% for note in ly_notes %}{{ note }} {% endfor %}>
// {%- endif %}
pub fn voicing(ly_notes: Vec<String>, ly_duration: Option<String>) -> String {
    Default::default()
}
