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

/// Rough word count for mixed CJK / Latin markdown.
///
/// CJK characters (Han, Hiragana, Katakana, Hangul) count 1 each. The
/// remaining non-CJK text is split on whitespace and each non-empty
/// token counts as one word. Markdown punctuation isn't stripped — it's
/// noise either way, and consistent across edits.
pub fn word_count(md: &str) -> usize {
    let mut cjk = 0usize;
    let mut latin = String::with_capacity(md.len());
    for ch in md.chars() {
        if is_cjk(ch) {
            cjk += 1;
        } else {
            latin.push(ch);
        }
    }
    let latin_tokens = latin.split_whitespace().filter(|s| !s.is_empty()).count();
    cjk + latin_tokens
}

/// Estimated reading time in whole minutes, floored at 1.
/// 300 "words" per minute matches the rule of thumb for adult readers on
/// mixed CJK / English content (English-only would be ~250).
pub fn reading_time_min(md: &str) -> i32 {
    let wc = word_count(md);
    ((wc as f64 / 300.0).ceil() as i32).max(1)
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'    // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}'  // CJK Ext-A
        | '\u{20000}'..='\u{2A6DF}' // CJK Ext-B
        | '\u{3040}'..='\u{309F}'  // Hiragana
        | '\u{30A0}'..='\u{30FF}'  // Katakana
        | '\u{AC00}'..='\u{D7AF}'  // Hangul Syllables
    )
}
