// New-comment notifications via SMTP.
//
// The notifier is built once at startup from the optional SmtpCfg. If SMTP
// isn't configured, AppState carries `None` and every call is a no-op.
//
// Sending is *best effort*: the submit handler spawns a fire-and-forget
// tokio task with `notifier.send_new_comment(...)`. A failed send logs at
// `warn` level but never affects the HTTP response — readers should never
// see "comment posted but email failed" 500s.

use std::sync::Arc;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};

use crate::config::SmtpCfg;

#[derive(Clone)]
pub struct Notifier(Arc<Inner>);

struct Inner {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    to: Mailbox,
    site_url: String,
    site_title: String,
}

impl Notifier {
    /// Build the transport once. Bails on bad addresses or bad SMTP
    /// config — surface the error at startup, not at first send.
    pub fn from_cfg(cfg: &SmtpCfg, site_url: &str, site_title: &str) -> anyhow::Result<Self> {
        let from: Mailbox = cfg
            .from
            .parse()
            .map_err(|e| anyhow::anyhow!("SMTP_FROM: {e}"))?;
        let to: Mailbox = cfg
            .to
            .parse()
            .map_err(|e| anyhow::anyhow!("SMTP_TO: {e}"))?;

        let builder = if cfg.starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)?
        };
        let mut builder = builder.port(cfg.port);
        if !cfg.username.is_empty() {
            builder = builder.credentials(Credentials::new(
                cfg.username.clone(),
                cfg.password.clone(),
            ));
        }
        let transport = builder.build();

        Ok(Self(Arc::new(Inner {
            transport,
            from,
            to,
            site_url: site_url.to_string(),
            site_title: site_title.to_string(),
        })))
    }

    /// Send "new comment on $post" to the operator. `status` is the
    /// post-blocklist status of the just-inserted row ('pending' or
    /// 'spam'), included so the operator can ignore spam alerts.
    pub async fn send_new_comment(
        &self,
        post_slug: &str,
        post_title: &str,
        author_name: &str,
        author_email: Option<&str>,
        content: &str,
        status: &str,
    ) {
        let inner = &self.0;
        let post_url = format!("{}/post/{}", inner.site_url, post_slug);
        let subject = format!("[{}] New {} comment on “{}”", inner.site_title, status, post_title);

        let body = format!(
            "From: {author}{email}\n\
             Status: {status}\n\
             Post: {post_title}\n\
             URL:  {post_url}\n\n\
             {content}\n",
            author = author_name,
            email = author_email
                .map(|e| format!(" <{}>", e))
                .unwrap_or_default(),
            status = status,
            post_title = post_title,
            post_url = post_url,
            content = content,
        );

        let msg = match Message::builder()
            .from(inner.from.clone())
            .to(inner.to.clone())
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
        {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "failed to build comment notification email");
                return;
            }
        };

        if let Err(e) = inner.transport.send(msg).await {
            tracing::warn!(error = %e, "failed to send comment notification email");
        }
    }
}
