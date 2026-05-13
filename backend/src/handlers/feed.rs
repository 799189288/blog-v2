use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use time::format_description::well_known::Rfc2822;

use crate::{
    error::AppResult,
    models::{post::Post, tag::Tag},
    state::AppState,
};

const RSS_LIMIT: i64 = 20;

pub async fn rss(State(state): State<AppState>) -> AppResult<Response> {
    let posts = sqlx::query_as::<_, Post>(
        r#"
        SELECT *
        FROM posts
        WHERE status = 'published'
        ORDER BY published_at DESC NULLS LAST, id DESC
        LIMIT $1
        "#,
    )
    .bind(RSS_LIMIT)
    .fetch_all(&state.db)
    .await?;

    let site = &state.site;
    let mut xml = String::with_capacity(4096);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(
        r#"<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:atom="http://www.w3.org/2005/Atom">"#,
    );
    xml.push_str("<channel>");
    xml.push_str(&format!("<title>{}</title>", esc(&site.title)));
    xml.push_str(&format!("<link>{}</link>", esc(&site.url)));
    xml.push_str(&format!(
        "<description>{}</description>",
        esc(&site.description)
    ));
    xml.push_str(&format!(
        r#"<atom:link href="{}/rss.xml" rel="self" type="application/rss+xml" />"#,
        esc(&site.url)
    ));
    if let Some(latest) = posts.first().and_then(|p| p.published_at) {
        if let Ok(s) = latest.format(&Rfc2822) {
            xml.push_str(&format!("<lastBuildDate>{}</lastBuildDate>", s));
        }
    }

    for p in &posts {
        let link = format!("{}/post/{}", site.url, p.slug);
        xml.push_str("<item>");
        xml.push_str(&format!("<title>{}</title>", esc(&p.title)));
        xml.push_str(&format!("<link>{}</link>", esc(&link)));
        xml.push_str(&format!(
            r#"<guid isPermaLink="true">{}</guid>"#,
            esc(&link)
        ));
        if let Some(pa) = p.published_at {
            if let Ok(s) = pa.format(&Rfc2822) {
                xml.push_str(&format!("<pubDate>{}</pubDate>", s));
            }
        }
        if let Some(ex) = &p.excerpt {
            if !ex.is_empty() {
                xml.push_str(&format!("<description>{}</description>", esc(ex)));
            }
        }
        xml.push_str(&format!(
            "<content:encoded><![CDATA[{}]]></content:encoded>",
            // CDATA can't contain "]]>"; defensively split it if a post ever does.
            p.content_html.replace("]]>", "]]]]><![CDATA[>")
        ));
        xml.push_str("</item>");
    }

    xml.push_str("</channel></rss>");

    Ok(xml_response("application/rss+xml; charset=utf-8", xml))
}

pub async fn sitemap(State(state): State<AppState>) -> AppResult<Response> {
    let posts = sqlx::query_as::<_, Post>(
        r#"
        SELECT *
        FROM posts
        WHERE status = 'published'
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let tags = sqlx::query_as::<_, Tag>("SELECT id, name, slug FROM tags ORDER BY id")
        .fetch_all(&state.db)
        .await?;

    let site = &state.site;
    let mut xml = String::with_capacity(2048);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

    // Home
    xml.push_str("<url>");
    xml.push_str(&format!("<loc>{}/</loc>", esc(&site.url)));
    xml.push_str("</url>");

    for p in &posts {
        xml.push_str("<url>");
        xml.push_str(&format!(
            "<loc>{}/post/{}</loc>",
            esc(&site.url),
            esc(&p.slug)
        ));
        if let Ok(s) = p.updated_at.format(&LASTMOD_FMT) {
            xml.push_str(&format!("<lastmod>{}</lastmod>", s));
        }
        xml.push_str("</url>");
    }

    for t in &tags {
        xml.push_str("<url>");
        xml.push_str(&format!(
            "<loc>{}/tag/{}</loc>",
            esc(&site.url),
            esc(&t.slug)
        ));
        xml.push_str("</url>");
    }

    xml.push_str("</urlset>");

    Ok(xml_response("application/xml; charset=utf-8", xml))
}

fn xml_response(content_type: &'static str, body: String) -> Response {
    (
        [(header::CONTENT_TYPE, HeaderValue::from_static(content_type))],
        body,
    )
        .into_response()
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

// Date-only ISO 8601, sufficient for sitemap <lastmod>.
const LASTMOD_FMT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]");
