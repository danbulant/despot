use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use cushy::{
    figures::{units::Lp, Size},
    styles::{Dimension, DimensionRange},
    value::{Destination, Dynamic, Source},
    widget::{MakeWidget, WidgetInstance},
    widgets::{
        label::{Displayable, LabelOverflow},
        Image, Label, Space, VirtualList,
    },
};
use itertools::Itertools;
use rspotify::model::SavedTrack;
use std::sync::Mutex;

use crate::{
    api::SpotifyContextRef, nodebug::NoDebug, rt::tokio_runtime, widgets::image::ImageExt,
};

const PER_PAGE: usize = 50;

#[derive(Debug)]
pub struct LikedSongsPage {
    tracks: Dynamic<HashMap<usize, SavedTrack>>,
    total_tracks: Dynamic<usize>,

    track_images: Arc<Mutex<HashMap<usize, WidgetInstance>>>,
    context: NoDebug<SpotifyContextRef>,
    pages_loading: Arc<RwLock<HashSet<usize>>>,
}

fn get_or_create_track_image(
    track_images: &Arc<Mutex<HashMap<usize, WidgetInstance>>>,
    idx: usize,
    create: impl FnOnce() -> WidgetInstance,
) -> WidgetInstance {
    let mut locked = track_images.lock().unwrap();
    if let Some(image) = locked.get(&idx) {
        image.clone()
    } else {
        let image = create();
        locked.insert(idx, image.clone());
        image
    }
}

impl LikedSongsPage {
    pub fn new(context: SpotifyContextRef) -> Self {
        Self {
            context: context.into(),

            tracks: Default::default(),
            total_tracks: Default::default(),
            pages_loading: Default::default(),
            track_images: Default::default(),
        }
    }

    pub fn into_widget(self) -> impl MakeWidget {
        let tracks = self.tracks;
        let pages_loading = self.pages_loading;
        let context = self.context;
        let total_tracks = self.total_tracks.clone();
        let track_images = self.track_images;

        tracks
            .for_each(|tracks| {
                println!("Tracks: {}", tracks.len());
            })
            .persist();

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
                let track = tracks.map_each(move |tracks| tracks.get(&index).cloned());
                index
                    .to_string()
                    .and({
                        get_or_create_track_image(&track_images, index, || {
                            Image::new_empty()
                                .with_url(track.map_each(|track| {
                                    track
                                        .as_ref()
                                        .map(|track| track.track.album.images[0].url.clone())
                                }))
                                .size(Size::squared(Dimension::Lp(Lp::points(40))))
                                .make_widget()
                        })
                    })
                    .and(track.map_each(|track| {
                        track
                            .as_ref()
                            .map(|track| {
                                Label::new(track.track.name.clone())
                                    .overflow(LabelOverflow::Clip)
                                    .and(
                                        Label::new(
                                            (track.track.artists)
                                                .iter()
                                                .map(|artist| artist.name.clone())
                                                .join(", "),
                                        )
                                        .overflow(LabelOverflow::Clip),
                                    )
                                    .into_rows()
                                    .make_widget()
                            })
                            .unwrap_or(Space::primary().make_widget())
                    }))
                    .and(track.map_each(|track| {
                        track
                            .as_ref()
                            .map(|track| {
                                track
                                    .track
                                    .album
                                    .name
                                    .clone()
                                    .into_label()
                                    .overflow(LabelOverflow::Clip)
                                    .make_widget()
                            })
                            .unwrap_or(Space::primary().make_widget())
                    }))
                    .and(track.map_each(|track| {
                        track
                            .as_ref()
                            .map(|track| {
                                track
                                    .added_at
                                    .to_string()
                                    .into_label()
                                    .overflow(LabelOverflow::Clip)
                                    .make_widget()
                            })
                            .unwrap_or(Space::primary().make_widget())
                    }))
                    .and(track.map_each(|track| {
                        track
                            .as_ref()
                            .map(|track| {
                                track
                                    .track
                                    .duration
                                    .to_string()
                                    .into_label()
                                    .overflow(LabelOverflow::Clip)
                                    .make_widget()
                            })
                            .unwrap_or(Space::primary().make_widget())
                    }))
                    .into_columns()
                    .size(Size {
                        width: DimensionRange::default(),
                        height: Dimension::Lp(Lp::points(60)).into(),
                    })
            },
        )
        .expand_horizontally()
    }
}
