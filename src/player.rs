use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cushy::value::{Destination, Dynamic, Source};
use librespot_metadata::audio::AudioItem;
use librespot_playback::player::{Player, PlayerEvent};
use tokio::time;

pub type DynamicPlayer = Arc<DynamicPlayerInner>;

pub struct DynamicPlayerInner {
    pub player: Arc<Player>,
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
    fn update_position(&self) {
        let state = self.state.get();
        let track_progress = match state {
            PlayerState::Loading {
                loading_at: duration,
            }
            | PlayerState::Paused {
                paused_at: duration,
            } => Some(duration),
            PlayerState::Stopped | PlayerState::Disconnected => None,
            PlayerState::Playing => {
                let started_at = *self.started_at.lock().unwrap();
                let started_at = started_at.unwrap_or_else(Instant::now);
                let position = Instant::now() - started_at;
                Some(position)
            }
        };
        self.track_progress.set(track_progress);
    }
    /// Run the DynamicPlayer event loop
    /// This updates the player state and track progress
    /// Run this only once per player (usually once per app)
    pub async fn run(&self) {
        let mut channel = self.player.get_player_event_channel();
        let mut interval = time::interval(time::Duration::from_millis(100));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
        self.track
            .for_each(|track| {
                dbg!(&track);
            })
            .persist();
        let mut id = 0u64;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.update_position();
                }
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
                                self.update_position();
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
                                self.update_position();
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
                                self.update_position();
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
                                self.update_position();
                            }
                            PlayerEvent::Seeked {
                                play_request_id,
                                position_ms,
                                track_id,
                            } => {
                                println!("Seeked {play_request_id} {position_ms} {track_id}");
                                *self.started_at.lock().unwrap() =
                                    Some(Instant::now() - Duration::from_millis(position_ms as u64));
                                self.update_position();
                            }
                            PlayerEvent::TrackChanged { audio_item } => {
                                println!("TrackChanged {}", audio_item.uri);
                                self.track.map_mut(|mut track| {
                                    *track = Some(audio_item);
                                });
                                self.update_position();
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
