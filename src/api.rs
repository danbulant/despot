use std::future::Future;
use std::time::Duration;

use chrono::TimeDelta;
use librespot_core::Session;
use librespot_oauth::OAuthToken;
use reqwest::StatusCode;
use rspotify::{AuthCodeSpotify, ClientError, ClientResult, Config, Token};
use rspotify::http::HttpError;

use crate::auth::{rspotify_scopes, SPOTIFY_REDIRECT_URI};


struct SpotifyContext {
    session: Session,
    api: AuthCodeSpotify,
    token: OAuthToken
}

impl SpotifyContext {
    fn new(session: Session, token: OAuthToken) -> SpotifyContext {
        let config = Config {
            token_refreshing: false,
            ..Default::default()
        };
        let api = AuthCodeSpotify::from_token_with_config(
            librespot_token_to_rspotify(&token),
            rspotify::Credentials::default(),
            rspotify::OAuth {
                proxies: None,
                redirect_uri: SPOTIFY_REDIRECT_URI.to_string(),
                scopes: rspotify_scopes(),
                state: String::new()
            },
            config,
        );
        SpotifyContext { session, api, token }
    }

    /// Execute `api_call` and retry once if a rate limit occurs.
    async fn api_with_retry<F, T: Future<Output = ClientResult<R>>, R>(&self, api_call: F) -> Option<R>
    where
        F: Fn(&AuthCodeSpotify) -> T,
    {
        let result = { api_call(&self.api).await };
        match result {
            Ok(v) => Some(v),
            Err(ClientError::Http(error)) => {
                dbg!("http error: {:?}", &error);
                if let HttpError::StatusCode(response) = error.as_ref() {
                    match response.status() {
                        StatusCode::TOO_MANY_REQUESTS => {
                            let waiting_duration = response
                                .headers()
                                .get("Retry-After")
                                .and_then(|v| v.to_str().ok().and_then(|v| v.parse::<u64>().ok()));
                            dbg!("rate limit hit. waiting {:?} seconds", waiting_duration);
                            
                            // sleep with tokio instead
                            tokio::time::sleep(Duration::from_secs(waiting_duration.unwrap_or(1))).await;

                            api_call(&self.api).await.ok()
                        }
                        StatusCode::UNAUTHORIZED => {
                            dbg!("token unauthorized. trying refresh..");
                            // self.update_token()
                            //     .and_then(move |_| api_call(&self.api).await.ok())
                            None
                        }
                        _ => {
                            eprintln!("unhandled api error: {:?}", response);
                            None
                        }
                    }
                } else {
                    None
                }
            }
            Err(e) => {
                eprintln!("unhandled api error: {}", e);
                None
            }
        }
    }
}

fn librespot_token_to_rspotify(token: &OAuthToken) -> Token {
    Token {
        access_token: token.access_token.clone(),
        scopes: rspotify_scopes(),
        refresh_token: None,
        expires_at: None,
        expires_in: TimeDelta::zero()
    }
}