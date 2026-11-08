//! Invite fragments stay out of HTTP request URLs and ordinary proxy logs.
use anyhow::{ensure, Context, Result};
use tungstenite::http::Uri;

pub const PROTOCOL: &str = "lookingglass-coop-v1";
const PREFIX: &str = "lg-coop=1.";

#[derive(Clone)]
pub struct Invite {
    origin: String,
    token: String,
}
impl std::fmt::Debug for Invite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Invite")
            .field("origin", &self.origin)
            .field("token", &"[redacted]")
            .finish()
    }
}
impl Invite {
    pub fn new(origin: &str) -> Result<Self> {
        let mut random = [0u8; 32];
        getrandom::fill(&mut random).map_err(|e| anyhow::anyhow!("Cannot create invite: {e}"))?;
        Self::parse(&format!(
            "{}/#{PREFIX}{}",
            origin.trim_end_matches('/'),
            random
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        ))
    }

    pub fn parse(link: &str) -> Result<Self> {
        ensure!(link.len() <= 2048, "That invite link is too long.");
        let (origin, fragment) = link
            .trim()
            .split_once('#')
            .context("Paste the complete invite link from the host.")?;
        let token = fragment
            .strip_prefix(PREFIX)
            .context("This invite is not compatible with this co-op build.")?;
        ensure!(
            token.len() == 64
                && token
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "The invite link is incomplete or damaged."
        );
        let uri: Uri = origin.parse().context("The invite address is invalid.")?;
        let host = uri.host().context("The invite has no host address.")?;
        ensure!(
            uri.authority().is_some_and(|a| !a.as_str().contains('@')),
            "Invite addresses cannot contain account details."
        );
        ensure!(
            uri.path() == "/" || uri.path().is_empty(),
            "The invite address has an unexpected path."
        );
        ensure!(
            uri.query().is_none(),
            "The invite address has unexpected parameters."
        );
        let local = matches!(host, "127.0.0.1" | "localhost" | "[::1]");
        ensure!(
            uri.scheme_str() == Some("https") || local && uri.scheme_str() == Some("http"),
            "Internet invites must use a secure HTTPS address."
        );
        Ok(Self {
            origin: origin.trim_end_matches('/').to_owned(),
            token: token.to_owned(),
        })
    }

    pub fn with_origin(&self, origin: &str) -> Result<Self> {
        Self::parse(&format!(
            "{}/#{PREFIX}{}",
            origin.trim_end_matches('/'),
            self.token
        ))
    }

    pub fn link(&self) -> String {
        format!("{}/#{PREFIX}{}", self.origin, self.token)
    }
    pub fn origin(&self) -> &str {
        &self.origin
    }
    pub(crate) fn authorization(&self) -> String {
        format!("Bearer {}", self.token)
    }
    pub(crate) fn websocket_url(&self) -> String {
        format!(
            "{}/coop",
            self.origin
                .replacen("https://", "wss://", 1)
                .replacen("http://", "ws://", 1)
        )
    }
}

pub(crate) fn authorized(expected: &str, received: &[u8]) -> bool {
    let expected = expected.as_bytes();
    if received.len() != expected.len() {
        return false;
    }
    expected
        .iter()
        .zip(received)
        .fold(0u8, |different, (a, b)| different | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invite_preserves_auth_when_relay_changes_origin_and_never_sends_fragment() {
        let invite = Invite::new("http://127.0.0.1:3456").unwrap();
        let public = invite
            .with_origin("https://example.trycloudflare.com")
            .unwrap();
        assert_eq!(invite.authorization(), public.authorization());
        assert_eq!(
            public.websocket_url(),
            "wss://example.trycloudflare.com/coop"
        );
        assert!(!format!("{public:?}").contains(&public.token));
        assert_eq!(
            Invite::parse(&public.link()).unwrap().authorization(),
            public.authorization()
        );
        assert!(authorized(
            &public.authorization(),
            public.authorization().as_bytes()
        ));
        assert!(!authorized(&public.authorization(), b"Bearer wrong"));
    }
    #[test]
    fn damaged_and_unsecured_invites_are_rejected_before_connecting() {
        let token = "0".repeat(64);
        for origin in [
            "http://example.com",
            "https://example.com/path",
            "https://user@example.com",
            "https://example.com/?query=1",
            "file:///private",
        ] {
            assert!(
                Invite::parse(&format!("{origin}#{PREFIX}{token}")).is_err(),
                "{origin}"
            );
        }
        assert!(Invite::parse("https://example.com").is_err());
        assert!(Invite::parse(&format!("https://example.com/#{PREFIX}{token}extra")).is_err());
        assert!(Invite::parse(&format!("https://example.com/#lg-coop=999.{token}")).is_err());
        assert_ne!(
            Invite::new("http://127.0.0.1:1").unwrap().authorization(),
            Invite::new("http://127.0.0.1:1").unwrap().authorization()
        );
    }
}
