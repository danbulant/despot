use std::{sync::Arc, thread};

use api::{SpotifyContext, SpotifyContextRef};
use auth::get_token;
use clap::Parser;
use cli::Args;
use cushy::{
    value::Dynamic, widget::MakeWidget, window::MakeWindow, Application, Open, PendingApp, Run,
    TokioRuntime,
};
use icons::load_fonts;
use librespot_connect::{
    spirc::{Spirc, SpircLoadCommand},
    state::ConnectStateConfig,
};
use librespot_core::{authentication::Credentials, cache::Cache, Session, SessionConfig};
use librespot_playback::{
    audio_backend,
    config::{AudioFormat, PlayerConfig},
    mixer::{softmixer::SoftMixer, Mixer, MixerConfig, NoOpVolume},
    player::{Player, PlayerEvent},
};
use player::{new_dynamic_player, DynamicPlayer, DynamicPlayerInner};
use widgets::{
    library::playlist::playlists_widget, pages::liked::LikedSongsPage, playback::bar::bar,
    ActivePage,
};

mod api;
mod auth;
mod cli;
mod icons;
mod nodebug;
mod player;
mod rt;
mod theme;
mod vibrancy;
mod widgets;

fn main() -> cushy::Result {
    let args = Args::parse();
    let app = PendingApp::new(TokioRuntime::default());
    // doesn't load fonts correctly yet, cushy bug
    // load_fonts(app.cushy().fonts());

    let token = get_token().unwrap();

    let cache = match Cache::new(None, Some("./cache/volume"), Some("./cache/audio"), None) {
        Ok(cache) => Some(cache),
        Err(e) => {
            eprintln!("Failed to create cache: {}", e);
            None
        }
    };
    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();
    let credentials = Credentials::with_access_token(&token.access_token);
    let default_connect_config = ConnectStateConfig::default();
    let connect_config = ConnectStateConfig {
        name: "Despot".to_string(),
        device_type: librespot_core::config::DeviceType::Computer,
        volume_steps: 256,
        initial_volume: cache
            .as_ref()
            .and_then(Cache::volume)
            .map(Into::into)
            .unwrap_or(default_connect_config.initial_volume),
        ..Default::default()
    };
    let backend = audio_backend::find(None).unwrap();

    let session;

    {
        let guard = app.cushy().enter_runtime();
        session = Session::new(session_config, cache);

        dbg!(session.user_data());

        let player = Player::new(
            player_config,
            session.clone(),
            Box::new(NoOpVolume),
            move || backend(None, audio_format),
        );

        let dynplayer = new_dynamic_player(player.clone());
        let context = SpotifyContextRef::new(SpotifyContext::new(
            session.clone(),
            token,
            dynplayer.clone(),
        ));

        let mut app = app.as_app();
        tokio::spawn(async move {
            let (_spirc, spirc_task) = Spirc::new(
                connect_config,
                session.clone(),
                credentials,
                player,
                Arc::new(SoftMixer::open(MixerConfig::default())),
            )
            .await
            .unwrap();
            // this cannot happen in `{}` inside join for some reason
            let dynplayer2 = dynplayer.clone();
            tokio::join!(spirc_task, dynplayer2.run(), async move {
                let user = context.current_user().await.unwrap();
                dbg!(&user);

                let playlists = context.current_user_playlists(None, None).await.unwrap();

                let selected_page = Dynamic::new(ActivePage::default());

                let win = playlists_widget(playlists.items, selected_page)
                    .and(LikedSongsPage::new(context.clone()).into_widget())
                    .into_columns()
                    .expand()
                    .and(bar(dynplayer))
                    .into_rows()
                    .expand()
                    .into_window();
                load_fonts(&win.fonts);
                win.open(&mut app).unwrap();
            });
        });

        drop(guard);
    }

    app.run()
}
