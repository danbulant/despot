use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cushy::value::{Destination, Dynamic};
use librespot_metadata::audio::AudioItem;
use librespot_playback::player::{Player, PlayerEvent};

pub type DynamicPlayer = Arc<DynamicPlayerInner>;

pub struct DynamicPlayerInner {
    player: Arc<Player>,
    pub state: Dynamic<PlayerState>,
    pub track: Dynamic<Option<Box<AudioItem>>>,
    pub track_progress: Dynamic<Option<Duration>>,
    started_at: Arc<Mutex<Option<Instant>>>,
    pub repeat: Dynamic<RepeatMode>,
    pub shuffle: Dynamic<bool>,
    pub volume: Dynamic<f32>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum RepeatMode {
    #[default]
    None,
    Track,
    Context,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum PlayerState {
    Loading {
        loading_at: Duration,
    },
    Playing,
    Paused {
        paused_at: Duration,
    },
    Stopped,
    #[default]
    Disconnected,
}

pub fn new_dynamic_player(player: Arc<Player>) -> DynamicPlayer {
    Arc::new(DynamicPlayerInner::new(player))
}

impl DynamicPlayerInner {
    pub fn new(player: Arc<Player>) -> Self {
        Self {
            player,
            repeat: Default::default(),
            shuffle: Default::default(),
            started_at: Default::default(),
            state: Default::default(),
            track: Default::default(),
            volume: Default::default(),
            track_progress: Default::default(),
        }
    }
    pub async fn run(&self) {
        let mut channel = self.player.get_player_event_channel();
        let mut id = 0u64;
        loop {
            tokio::select! {
                event = channel.recv() => {
                    if let Some(event) = event {
                        match event {
                            PlayerEvent::Stopped {
                                play_request_id, ..
                            } => {
                                println!("Stopped {play_request_id}");
                                self.state.set(PlayerState::Stopped);
                            }
                            PlayerEvent::Loading {
                                play_request_id,
                                position_ms,
                                track_id,
                            } => {
                                println!("Loading {play_request_id} {position_ms} {track_id}");
                                self.state.set(PlayerState::Loading {
                                    loading_at: Duration::from_millis(position_ms as u64),
                                });
                            }
                            PlayerEvent::Playing {
                                play_request_id,
                                position_ms,
                                track_id,
                            } => {
                                println!("Playing {play_request_id} {position_ms} {track_id}");
                                self.state.set(PlayerState::Playing);
                                *self.started_at.lock().unwrap() =
                                    Some(Instant::now() - Duration::from_millis(position_ms as u64));
                            }
                            PlayerEvent::Paused {
                                play_request_id,
                                position_ms,
                                track_id,
                            } => {
                                println!("Paused {play_request_id} {position_ms} {track_id}");
                                self.state.set(PlayerState::Paused {
                                    paused_at: Duration::from_millis(position_ms as u64),
                                });
                            }
                            PlayerEvent::Unavailable {
                                play_request_id,
                                track_id,
                            } => {
                                println!("Unavailable {play_request_id} {track_id}");
                            }
                            PlayerEvent::VolumeChanged { volume } => {
                                println!("volume {volume}");
                                self.volume.set(volume as f32 / u16::MAX as f32)
                            }
                            PlayerEvent::PositionCorrection {
                                play_request_id,
                                position_ms,
                                track_id,
                            } => {
                                println!("PositionCorrection {play_request_id} {position_ms} {track_id}");
                                *self.started_at.lock().unwrap() =
                                    Some(Instant::now() - Duration::from_millis(position_ms as u64));
                            }
                            PlayerEvent::Seeked {
                                play_request_id,
                                position_ms,
                                track_id,
                            } => {
                                println!("Seeked {play_request_id} {position_ms} {track_id}");
                                *self.started_at.lock().unwrap() =
                                    Some(Instant::now() - Duration::from_millis(position_ms as u64));
                            }
                            PlayerEvent::TrackChanged { audio_item } => {
                                println!("TrackChanged {}", audio_item.uri);
                                dbg!(&audio_item);
                                *self.track.lock() = Some(audio_item);
                            }
                            PlayerEvent::SessionConnected {
                                connection_id,
                                user_name,
                            } => {
                                println!("SessionConnected {connection_id} {user_name}");
                                self.state.set(PlayerState::Stopped);
                            }
                            PlayerEvent::SessionDisconnected {
                                connection_id,
                                user_name,
                            } => {
                                println!("SessionDisconnected {connection_id} {user_name}");
                                self.state.set(PlayerState::Disconnected);
                            }
                            PlayerEvent::SessionClientChanged {
                                client_brand_name,
                                client_id,
                                client_model_name,
                                client_name,
                            } => {
                                println!("SessionClientChanged {client_brand_name} {client_id} {client_model_name} {client_name}");
                            }
                            PlayerEvent::ShuffleChanged { shuffle } => {
                                println!("ShuffleChanged {shuffle}");
                                self.shuffle.set(shuffle);
                            }
                            PlayerEvent::RepeatChanged { context, track } => {
                                let repeat_mode = match (context, track) {
                                    (true, false) => RepeatMode::Context,
                                    (false, true) => RepeatMode::Track,
                                    _ => RepeatMode::None,
                                };
                                println!("RepeatChanged {repeat_mode:?}");
                                self.repeat.set(repeat_mode);
                            }
                            PlayerEvent::AutoPlayChanged { .. } => {
                                println!("AutoPlayChanged")
                            }
                            PlayerEvent::FilterExplicitContentChanged { .. } => {
                                println!("FilterExplicitContentChanged")
                            }
                            PlayerEvent::PlayRequestIdChanged { play_request_id } => {
                                println!("PlayRequestIdChanged {play_request_id}");
                                id = play_request_id;
                            }
                            _ => {}
                        };
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
