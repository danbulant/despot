use std::future::Future;
use std::time::{Duration, Instant};

use chrono::TimeDelta;
use futures_util::lock::Mutex;
use librespot_core::Session;
use librespot_oauth::OAuthToken;
use reqwest::StatusCode;
use rspotify::model::PrivateUser;
use rspotify::prelude::*;
use rspotify::{AuthCodeSpotify, ClientError, ClientResult, Config, Token};
use rspotify::http::HttpError;

use crate::auth::{get_access_token_from_refresh_token, rspotify_scopes, SPOTIFY_REDIRECT_URI};


pub struct SpotifyContext {
    session: Session,
    api: AuthCodeSpotify,
    token: Mutex<OAuthToken>
}

impl SpotifyContext {
    pub fn new(session: Session, token: OAuthToken) -> SpotifyContext {
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
        SpotifyContext { session, api, token: Mutex::new(token) }
    }

    pub async fn update_token(&self) -> bool {
        let expires_soon = Instant::now() + Duration::from_secs(10);
        let token = self.token.lock().await;
        if token.expires_at < expires_soon {
            let refresh_token = token.refresh_token.clone();
            drop(token);
            let token = get_access_token_from_refresh_token(&refresh_token).unwrap();;
            *self.api.token.lock().await.unwrap() = Some(librespot_token_to_rspotify(&token));
            *self.token.lock().await = token;
            true
        } else {
            false
        }
    }

    /// Execute `api_call` and retry once if a rate limit occurs.
    async fn api_with_retry<'a, F, T: Future<Output = ClientResult<R>>, R>(&'a self, api_call: F) -> Option<R>
    where
        F: Fn(&'a AuthCodeSpotify) -> T,
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
                            let updated = self.update_token().await;
                            if updated {
                                api_call(&self.api).await.ok()
                            } else {
                                None
                            }
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
    
    pub async fn current_user(&self) -> Result<PrivateUser, ()> {
        self.api_with_retry(|api| api.current_user()).await.ok_or(())
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