use std::{sync::Arc, thread};

use api::{SpotifyContext, SpotifyContextRef};
use auth::get_token;
use clap::Parser;
use cli::Args;
use cushy::{
    value::Dynamic, widget::MakeWidget, window::MakeWindow, Application, Open, PendingApp, Run,
    TokioRuntime,
};
use librespot_connect::{
    spirc::{Spirc, SpircLoadCommand},
    state::ConnectStateConfig,
};
use librespot_core::{authentication::Credentials, Session, SessionConfig};
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
mod nodebug;
mod player;
mod rt;
mod theme;
mod vibrancy;
mod widgets;

fn main() -> cushy::Result {
    let args = Args::parse();
    let app = PendingApp::new(TokioRuntime::default());

    let token = get_token().unwrap();

    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();
    let credentials = Credentials::with_access_token(&token.access_token);
    let connect_config = ConnectStateConfig::default();
    let backend = audio_backend::find(None).unwrap();

    let session;

    {
        let guard = app.cushy().enter_runtime();
        session = Session::new(session_config, None);

        dbg!(session.user_data());

        let player = Player::new(
            player_config,
            session.clone(),
            Box::new(NoOpVolume),
            move || backend(None, audio_format),
        );

        let context = SpotifyContextRef::new(SpotifyContext::new(session.clone(), token));

        let mut app = app.as_app();
        tokio::spawn(async move {
            let (_spirc, spirc_task) = Spirc::new(
                connect_config,
                session.clone(),
                credentials,
                player.clone(),
                Arc::new(SoftMixer::open(MixerConfig::default())),
            )
            .await
            .unwrap();
            let dynplayer = new_dynamic_player(player);
            // this cannot happen in `{}` inside join for some reason
            let dynplayer2 = dynplayer.clone();
            tokio::join!(spirc_task, dynplayer2.run(), async move {
                let user = context.current_user().await.unwrap();
                dbg!(&user);
                // let userid = user.id;

                let playlists = context.current_user_playlists(None, None).await.unwrap();

                let selected_page = Dynamic::new(ActivePage::default());

                playlists_widget(playlists.items, selected_page)
                    .and(LikedSongsPage::new(context.clone()).into_widget())
                    .into_columns()
                    .expand()
                    .and(bar(dynplayer))
                    .into_rows()
                    .expand()
                    .make_window()
                    .open(&mut app)
                    .unwrap();
            });
        });

        drop(guard);
    }

    app.run()
}
