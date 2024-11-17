use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    sync::{Arc, RwLock},
};

use cushy::{
    figures::{units::Lp, Size},
    styles::{Dimension, DimensionRange},
    value::{Destination, Dynamic, Source},
    widget::MakeWidget,
    widgets::VirtualList,
};
use futures_util::lock::Mutex;
use rspotify::model::SavedTrack;

use crate::{api::SpotifyContextRef, nodebug::NoDebug, rt::tokio_runtime};

const PER_PAGE: usize = 50;

#[derive(Debug)]
pub struct LikedSongsPage {
    tracks: Dynamic<HashMap<usize, SavedTrack>>,
    total_tracks: Dynamic<usize>,

    context: NoDebug<SpotifyContextRef>,
    pages_loading: Arc<RwLock<HashSet<usize>>>,
}

impl LikedSongsPage {
    pub fn new(context: SpotifyContextRef) -> Self {
        Self {
            context: context.into(),

            tracks: Default::default(),
            total_tracks: Default::default(),
            pages_loading: Default::default(),
        }
    }

    pub fn into_widget(self) -> impl MakeWidget {
        let tracks = self.tracks;
        let pages_loading = self.pages_loading;
        let context = self.context;
        let total_tracks = self.total_tracks.clone();
        VirtualList::new(
            total_tracks.clone().map_each(|total| (*total).max(1)),
            move |index| {
                let context = context.clone();
                let pages_loading = pages_loading.clone();
                let total_tracks = total_tracks.clone();
                let page = index / PER_PAGE;
                tracks.map_ref({
                    let tracks = tracks.clone();
                    |loaded_tracks| {
                        if !loaded_tracks.contains_key(&index)
                            && !pages_loading.read().unwrap().contains(&page)
                        {
                            pages_loading.write().unwrap().insert(page);
                            tokio_runtime().spawn(async move {
                                println!("Loading page {} idx {}", page, index);
                                let saved_tracks = context
                                    .current_user_saved_tracks(
                                        Some(PER_PAGE as _),
                                        Some((page * PER_PAGE) as _),
                                    )
                                    .await;
                                let Ok(saved_tracks) = saved_tracks else {
                                    eprintln!("Failed to load page {}", page);
                                    // pages_loading.write().unwrap().remove(&page);
                                    return;
                                };
                                println!("Loaded page {} got tracks {}", page, saved_tracks.total);
                                total_tracks.set(saved_tracks.total as usize);
                                tracks.map_mut(|mut tracks| {
                                    for (i, track) in saved_tracks.items.into_iter().enumerate() {
                                        tracks.insert(i + saved_tracks.offset as usize, track);
                                    }
                                });
                            });
                        }
                    }
                });
                tracks
                    .map_each(move |tracks| {
                        if let Some(track) = tracks.get(&index) {
                            format!("{} - {}", track.track.name, track.track.artists[0].name)
                        } else {
                            format!("Loading...")
                        }
                    })
                    .size(Size {
                        width: DimensionRange::default(),
                        height: Dimension::Lp(Lp::points(60)).into(),
                    })
            },
        )
        .expand_horizontally()
    }
}
