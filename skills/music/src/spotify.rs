//! Spotify ignores "play this" from apps it doesn't recognise: both the
//! MEDIA_PLAY_FROM_SEARCH intent and playFromSearch on its session leave it
//! on a search screen. It does play a track opened by its own URI, so we look
//! the query up on the Web API and hand the frontend that URI to open.
//!
//! Each user signs in through their own Spotify app (Client ID + PKCE, no
//! secret): Spotify caps a shared Client ID at five accounts.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use ari_skill_sdk as ari;

const AUTHORIZE_URL: &str = "https://accounts.spotify.com/authorize";
pub const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
pub const SIGN_IN_TIMEOUT_MS: u64 = 300_000;
// Search needs no scope. These two are for starting playback on the phone
// through the Web API, which unlike the app link works while it's locked.
// Requested now so adding that later doesn't force everyone to sign in again.
const SCOPES: &str = "user-read-playback-state user-modify-playback-state";
#[cfg(target_arch = "wasm32")]
const REFRESH_TOKEN_KEY: &str = "spotify_refresh_token";

#[derive(Debug, PartialEq)]
pub struct Track {
    pub uri: String,
    pub name: String,
    pub artist: String,
}

#[derive(Debug, PartialEq)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum LookupError {
    NotSignedIn,
    /// Spotify refused the stored refresh token: revoked, or the Client ID
    /// changed since the user signed in.
    SignInExpired,
    /// The request never left the device.
    NoConnection,
    /// Spotify answered with something other than a result: quota, outage.
    Unavailable,
    NotFound,
}

pub fn pkce_challenge(verifier: &str) -> String {
    ari::crypto::base64url_nopad(&ari::crypto::sha256(verifier.as_bytes()))
}

pub fn authorize_url(client_id: &str, redirect_uri: &str, state: &str, challenge: &str) -> String {
    format!(
        "{AUTHORIZE_URL}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge_method=S256&code_challenge={}&state={}",
        percent_encode(client_id),
        percent_encode(redirect_uri),
        percent_encode(SCOPES),
        percent_encode(challenge),
        percent_encode(state),
    )
}

pub fn exchange_body(code: &str, client_id: &str, verifier: &str, redirect_uri: &str) -> String {
    format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
        percent_encode(code),
        percent_encode(redirect_uri),
        percent_encode(client_id),
        percent_encode(verifier),
    )
}

pub fn refresh_body(refresh_token: &str, client_id: &str) -> String {
    format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        percent_encode(refresh_token),
        percent_encode(client_id),
    )
}

pub fn search_url(query: &str) -> String {
    format!(
        "https://api.spotify.com/v1/search?type=track&limit=1&q={}",
        percent_encode(query)
    )
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn parse_tokens(body: &str) -> Option<Tokens> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    Some(Tokens {
        access_token: v.get("access_token")?.as_str()?.to_string(),
        refresh_token: v.get("refresh_token").and_then(|r| r.as_str()).map(|s| s.to_string()),
    })
}

pub fn parse_first_track(body: &str) -> Option<Track> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let item = v.get("tracks")?.get("items")?.as_array()?.first()?;
    Some(Track {
        uri: item.get("uri")?.as_str()?.to_string(),
        name: item.get("name")?.as_str()?.to_string(),
        artist: item
            .get("artists")?
            .as_array()?
            .first()?
            .get("name")?
            .as_str()?
            .to_string(),
    })
}

#[cfg(target_arch = "wasm32")]
pub fn store_refresh_token(token: &str) -> bool {
    ari::storage_set(REFRESH_TOKEN_KEY, token)
}

/// Swaps the stored refresh token for an access token. Spotify may rotate
/// the refresh token on any refresh, and the old one stops working, so a new
/// one is persisted before anything else happens.
#[cfg(target_arch = "wasm32")]
fn access_token(client_id: &str) -> Result<String, LookupError> {
    let refresh = ari::storage_get(REFRESH_TOKEN_KEY)
        .filter(|s| !s.is_empty())
        .ok_or(LookupError::NotSignedIn)?;
    let resp = ari::http_request(
        "POST",
        TOKEN_URL,
        &[("Content-Type", "application/x-www-form-urlencoded")],
        Some(&refresh_body(refresh, client_id)),
    );
    match resp.status {
        0 => return Err(LookupError::NoConnection),
        200 => {}
        400 | 401 => return Err(LookupError::SignInExpired),
        _ => return Err(LookupError::Unavailable),
    }
    let tokens = resp
        .body
        .as_deref()
        .and_then(parse_tokens)
        .ok_or(LookupError::Unavailable)?;
    if let Some(rotated) = &tokens.refresh_token {
        store_refresh_token(rotated);
    }
    Ok(tokens.access_token)
}

#[cfg(target_arch = "wasm32")]
pub fn find_track(client_id: &str, query: &str) -> Result<Track, LookupError> {
    let bearer = format!("Bearer {}", access_token(client_id)?);
    let resp = ari::http_request("GET", &search_url(query), &[("Authorization", bearer.as_str())], None);
    match resp.status {
        0 => Err(LookupError::NoConnection),
        200 => resp
            .body
            .as_deref()
            .and_then(parse_first_track)
            .ok_or(LookupError::NotFound),
        _ => Err(LookupError::Unavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 7636 appendix B: the verifier/challenge pair the spec publishes.
    #[test]
    fn pkce_challenge_matches_rfc_7636_example() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn authorize_url_carries_pkce_and_encodes_the_redirect() {
        assert_eq!(
            authorize_url("abc123", "https://heyari.dev/oauth/callback", "st8", "chal"),
            "https://accounts.spotify.com/authorize?response_type=code&client_id=abc123\
             &redirect_uri=https%3A%2F%2Fheyari.dev%2Foauth%2Fcallback\
             &scope=user-read-playback-state%20user-modify-playback-state\
             &code_challenge_method=S256&code_challenge=chal&state=st8"
        );
    }

    #[test]
    fn exchange_body_sends_client_id_and_verifier_not_a_secret() {
        assert_eq!(
            exchange_body("c0de", "abc123", "ver", "https://heyari.dev/oauth/callback"),
            "grant_type=authorization_code&code=c0de\
             &redirect_uri=https%3A%2F%2Fheyari.dev%2Foauth%2Fcallback\
             &client_id=abc123&code_verifier=ver"
        );
    }

    #[test]
    fn refresh_body_encodes_the_token() {
        assert_eq!(
            refresh_body("AQ+b/c=", "abc123"),
            "grant_type=refresh_token&refresh_token=AQ%2Bb%2Fc%3D&client_id=abc123"
        );
    }

    #[test]
    fn search_url_percent_encodes_the_query() {
        assert_eq!(
            search_url("guns n' roses & friends"),
            "https://api.spotify.com/v1/search?type=track&limit=1&q=guns%20n%27%20roses%20%26%20friends"
        );
    }

    #[test]
    fn search_url_encodes_non_ascii_as_utf8_bytes() {
        assert_eq!(
            search_url("björk"),
            "https://api.spotify.com/v1/search?type=track&limit=1&q=bj%C3%B6rk"
        );
    }

    #[test]
    fn parses_tokens_with_rotated_refresh() {
        let body = r#"{"access_token":"BQD123","token_type":"Bearer","expires_in":3600,"refresh_token":"AQX9"}"#;
        assert_eq!(
            parse_tokens(body),
            Some(Tokens { access_token: "BQD123".to_string(), refresh_token: Some("AQX9".to_string()) })
        );
    }

    #[test]
    fn parses_tokens_without_refresh() {
        let body = r#"{"access_token":"BQD123","token_type":"Bearer","expires_in":3600}"#;
        assert_eq!(
            parse_tokens(body),
            Some(Tokens { access_token: "BQD123".to_string(), refresh_token: None })
        );
    }

    #[test]
    fn token_parse_fails_on_error_body() {
        assert_eq!(parse_tokens(r#"{"error":"invalid_grant"}"#), None);
        assert_eq!(parse_tokens("not json"), None);
    }

    #[test]
    fn parses_first_track_uri_name_and_lead_artist() {
        let body = r#"{"tracks":{"items":[{
            "uri":"spotify:track:4cOdK2wGLETKBW3PvgPWqT",
            "name":"Never Gonna Give You Up",
            "artists":[{"name":"Rick Astley"},{"name":"Someone Else"}]
        }]}}"#;
        assert_eq!(
            parse_first_track(body),
            Some(Track {
                uri: "spotify:track:4cOdK2wGLETKBW3PvgPWqT".to_string(),
                name: "Never Gonna Give You Up".to_string(),
                artist: "Rick Astley".to_string(),
            })
        );
    }

    #[test]
    fn no_items_is_not_found() {
        assert_eq!(parse_first_track(r#"{"tracks":{"items":[]}}"#), None);
        assert_eq!(parse_first_track(r#"{"tracks":{}}"#), None);
    }
}
