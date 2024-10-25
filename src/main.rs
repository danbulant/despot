use std::thread;

use api::SpotifyContext;
use auth::get_token;
use clap::Parser;
use cli::Args;
use cushy::{figures::units::Lp, value::Dynamic, widget::MakeWidget, window::MakeWindow, Application, Open, PendingApp, Run, TokioRuntime};
use librespot_core::{authentication::Credentials, Session, SessionConfig};
use librespot_playback::{audio_backend, config::{AudioFormat, PlayerConfig}, mixer::NoOpVolume, player::Player};
use widgets::{library::playlist::playlists_widget, virtual_list::{virtual_list, VirtualList}, ActivePage};

mod vibrancy;
mod theme;
mod cli;
mod auth;
mod widgets;
mod rt;
mod api;

struct TestVirtualList;

impl VirtualList for TestVirtualList {
    fn item_count(&self) -> impl cushy::value::IntoDynamic<usize> {
        Dynamic::new(100)
    }
    fn item_height(&self) -> impl cushy::value::IntoDynamic<cushy::styles::Dimension> {
        Dynamic::new(cushy::styles::Dimension::Lp(Lp::inches_f(0.5)))
    }
    fn widget_at(&self, index: usize) -> impl MakeWidget {
        // println!("Creating item {}", index);
        format!("Item {}", index)
    }
}

fn main() -> cushy::Result {
    let args = Args::parse();
    let mut app = PendingApp::new(TokioRuntime::default());

    let token = get_token().unwrap();

    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let audio_format = AudioFormat::default();
    let credentials = Credentials::with_access_token(&token.access_token);
    let backend = audio_backend::find(None).unwrap();
    
    let session;

    {
        let guard = app.cushy().enter_runtime();
        session = Session::new(session_config, None);
        
        let player = Player::new(player_config, session.clone(), Box::new(NoOpVolume), move || {
            backend(None, audio_format)
        });
    
        tokio::spawn({ let session = session.clone(); async move {
            if let Err(e) = session.connect(credentials, false).await {
                println!("Error connecting: {}", e);
            }
        }});
        
        thread::spawn(move || {
            let mut channel = player.get_player_event_channel();
            loop {
                let event = channel.blocking_recv();
                if let Some(event) = event {
                    dbg!(event);
                } else { break; }
            }
        });

        dbg!(session.user_data());
    
        let context = SpotifyContext::new(session, token);

        let mut app = app.as_app();
        tokio::spawn(async move {
            let user = context.current_user().await.unwrap();
            dbg!(&user);
            let userid = user.id;

            let playlists = context.current_user_playlists(None, None).await.unwrap();

            let selected_page = Dynamic::new(ActivePage::default());

            playlists_widget(playlists.items, selected_page)
            .and(
                virtual_list(TestVirtualList)
            )
                .into_columns()
                .make_window()
                .open(&mut app)
                .unwrap();
        });

        drop(guard);
    }

    app.run()
}