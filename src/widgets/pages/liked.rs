use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use chrono::TimeDelta;
use cushy::{
    figures::{units::Lp, Size},
    styles::{CornerRadii, Dimension, DimensionRange, Edges},
    value::{Destination, Dynamic, Source},
    widget::{MakeWidget, WidgetInstance},
    widgets::{
        button::ButtonKind,
        image::ImageCornerRadius,
        label::{Displayable, LabelOverflow},
        Image, Label, Space, VirtualList,
    },
};
use itertools::Itertools;
use librespot_core::SpotifyId;
use rspotify::model::{Id, SavedTrack};
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
    create: impl FnOnce(usize) -> WidgetInstance,
) -> WidgetInstance {
    let mut locked = track_images.lock().unwrap();
    if let Some(image) = locked.get(&idx) {
        image.clone()
    } else {
        let image = create(idx);
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
                    let context = context.clone();
                    |loaded_tracks| {
                        if !loaded_tracks.contains_key(&index)
                            && !pages_loading.read().unwrap().contains(&page)
                        {
                            pages_loading.write().unwrap().insert(page);
                            tokio_runtime().spawn(async move {
                                // println!("Loading page {} idx {}", page, index);
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
                                // println!("Loaded page {} got tracks {}", page, saved_tracks.total);
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
                    .size(Size {
                        width: Dimension::Lp(Lp::points(40)).into(),
                        height: DimensionRange::default(),
                    })
                    .fit_horizontally()
                    .and({
                        get_or_create_track_image(&track_images, index, |index| {
                            Image::new_empty()
                                .with_url(tracks.map_each(move |tracks| {
                                    tracks
                                        .get(&index)
                                        .map(|track| (&track.track.album.images)[0].url.clone())
                                }))
                                .size(Size::squared(Dimension::Lp(Lp::points(40))))
                                .with(&ImageCornerRadius, Dimension::Lp(Lp::points(4)))
                                .make_widget()
                        })
                        .size(Size::squared(Dimension::Lp(Lp::points(40))))
                    })
                    .and(
                        track
                            .map_each(|track| {
                                track
                                    .as_ref()
                                    .map(|track| {
                                        Label::new(track.track.name.clone())
                                            .overflow(LabelOverflow::Clip)
                                            .align_left()
                                            .and(
                                                Label::new(
                                                    (track.track.artists)
                                                        .iter()
                                                        .map(|artist| artist.name.clone())
                                                        .join(", "),
                                                )
                                                .overflow(LabelOverflow::Clip)
                                                .align_left(),
                                            )
                                            .into_rows()
                                            .make_widget()
                                    })
                                    .unwrap_or(Space::primary().make_widget())
                            })
                            .align_left()
                            .expand_weighted(2),
                    )
                    .and(
                        track
                            .map_each(|track| {
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
                            })
                            .align_left()
                            .expand_weighted(1),
                    )
                    .and(
                        track
                            .map_each(|track| {
                                track
                                    .as_ref()
                                    .map(|track| {
                                        track
                                            .added_at
                                            .format("%B %-e, %Y")
                                            .to_string()
                                            .into_label()
                                            .overflow(LabelOverflow::Clip)
                                            .make_widget()
                                    })
                                    .unwrap_or(Space::primary().make_widget())
                            })
                            .align_left()
                            .expand_weighted(1),
                    )
                    .and(
                        track
                            .map_each(|track| {
                                track
                                    .as_ref()
                                    .map(|track| {
                                        format_delta(track.track.duration)
                                            .into_label()
                                            .overflow(LabelOverflow::Clip)
                                            .make_widget()
                                    })
                                    .unwrap_or(Space::primary().make_widget())
                            })
                            .pad_by(Edges::default().with_horizontal(Dimension::Lp(Lp::points(5)))),
                    )
                    .into_columns()
                    .centered()
                    .size(Size {
                        width: DimensionRange::default(),
                        height: Dimension::Lp(Lp::points(60)).into(),
                    })
                    .expand_horizontally()
                    .into_button()
                    .kind(ButtonKind::Transparent)
                    .on_click({
                        let player = context.player.clone();
                        move |_| {
                            dbg!("Clicked", index);
                            let id = track.map_ref(|track| {
                                track.as_ref().map(|track| track.track.id.clone()).flatten()
                            });
                            dbg!(&id);
                            match id {
                                Some(id) => player.player.load(
                                    SpotifyId::from_uri(&id.uri()).unwrap(),
                                    true,
                                    0,
                                ),
                                None => println!("No track id :("),
                            }
                        }
                    })
            },
        )
        .expand_horizontally()
    }
}

fn format_delta(delta: TimeDelta) -> String {
    format!("{}:{:02}", delta.num_minutes(), delta.num_seconds() % 60)
}
