use comrak::{ComrakOptions, markdown_to_html};

pub fn render(md: &str) -> String {
    markdown_to_html(md, &options())
}

fn options() -> ComrakOptions<'static> {
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts.extension.description_lists = true;
    opts.render.unsafe_ = false;
    opts.render.hardbreaks = false;
    opts
}

/// Build a plain-text excerpt by rendering markdown -> HTML, then stripping
/// every tag and collapsing whitespace. Truncates to `max_chars` characters.
pub fn excerpt(md: &str, max_chars: usize) -> String {
    let html = markdown_to_html(md, &options());
    let text = strip_html(&html);
    truncate(&text, max_chars)
}

/// Walk a string and skip everything between `<` and `>`.
/// Decode the handful of HTML entities comrak emits.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut last_was_space = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                if !last_was_space {
                    out.push(' ');
                    last_was_space = true;
                }
            }
            _ if in_tag => {}
            c if c.is_whitespace() => {
                if !last_was_space && !out.is_empty() {
                    out.push(' ');
                    last_was_space = true;
                }
            }
            c => {
                out.push(c);
                last_was_space = false;
            }
        }
    }

    decode_entities(out.trim())
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i >= max_chars {
            out.push('…');
            return out;
        }
        out.push(c);
    }
    out
}
