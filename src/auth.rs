use std::{collections::HashSet, io::{Read, Write}, time::{Duration, Instant}};
use librespot_core::SessionConfig;
use librespot_oauth::{get_access_token, OAuthError, OAuthToken};
use oauth2::{basic::BasicClient, reqwest::http_client, AuthUrl, ClientId, RedirectUrl, RefreshToken, TokenResponse, TokenUrl};

pub const SPOTIFY_REDIRECT_URI: &str = "http://127.0.0.1:8898/login";

const SPOTIFY_SCOPES: [&str; 16] = [
    "user-read-playback-state",
    "user-modify-playback-state",
    "user-read-currently-playing",
    "app-remote-control",
    "streaming",
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-private",
    "playlist-modify-public",
    "user-follow-modify",
    "user-follow-read",
    "user-read-playback-position",
    "user-top-read",
    "user-read-recently-played",
    "user-library-modify",
    "user-library-read",
];

pub fn rspotify_scopes() -> HashSet<String> {
    HashSet::from_iter(SPOTIFY_SCOPES.map(|t| t.to_string()))
}

fn get_refresh_token_file_location() -> String {
    "./refresh_token.txt".to_string()
}

fn read_refresh_token() -> Option<String> {
    let file = std::fs::File::open(get_refresh_token_file_location());
    if file.is_err() {
        return None;
    }

    let mut reader = std::io::BufReader::new(file.unwrap());
    let mut token = String::new();
    reader.read_to_string(&mut token).unwrap();
    Some(token)
}

fn write_refresh_token(token: &str) {
    let mut file = std::fs::File::create(get_refresh_token_file_location()).unwrap();
    file.write_all(token.as_bytes()).unwrap();
}

fn oauth2_client() -> Result<BasicClient, OAuthError> {
    let auth_url = AuthUrl::new("https://accounts.spotify.com/authorize".to_string())
        .map_err(|_| OAuthError::InvalidSpotifyUri)?;
    let token_url = TokenUrl::new("https://accounts.spotify.com/api/token".to_string())
        .map_err(|_| OAuthError::InvalidSpotifyUri)?;
    let redirect_url =
        RedirectUrl::new(SPOTIFY_REDIRECT_URI.to_string()).map_err(|e| OAuthError::InvalidRedirectUri {
            uri: SPOTIFY_REDIRECT_URI.to_string(),
            e,
        })?;
    let client = BasicClient::new(
        ClientId::new(SessionConfig::default().client_id),
        None,
        auth_url,
        Some(token_url),
    );
    let client = client.set_redirect_uri(redirect_url);
    Ok(client)
}

pub fn get_access_token_from_refresh_token(refresh_token: &str) -> Result<OAuthToken, OAuthError> {
    let client = oauth2_client()?;
    let token = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
        .request(http_client)
        .map_err(|e| { dbg!(e); OAuthError::ExchangeCode { e: refresh_token.to_string() } })?;

    let refresh_token = match token.refresh_token() {
        Some(t) => t.secret().to_string(),
        _ => "".to_string(), // Spotify always provides a refresh token.
    };

    write_refresh_token(&refresh_token);

    Ok(OAuthToken {
        access_token: token.access_token().secret().to_string(),
        refresh_token,
        expires_at: Instant::now()
            + token
                .expires_in()
                .unwrap_or_else(|| Duration::from_secs(3600)),
        token_type: format!("{:?}", token.token_type()).to_string(), // Urgh!?
        scopes: Vec::from(SPOTIFY_SCOPES.map(|s| s.to_string())),
    })
}

pub fn get_token() -> Result<OAuthToken, OAuthError> {
    let refresh = read_refresh_token();

    match refresh {
        Some(token) => {
            let token = get_access_token_from_refresh_token(&token);
            match token {
                Ok(token) => return Ok(token),
                Err(e) => {
                    eprintln!("Error refreshing token, trying to relogin. Error: {}", e);
                }
            }
        }
        None => {}
    };

    let token = match get_access_token(&SessionConfig::default().client_id, SPOTIFY_REDIRECT_URI, Vec::from(SPOTIFY_SCOPES)) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    write_refresh_token(&token.refresh_token);

    Ok(token)
}