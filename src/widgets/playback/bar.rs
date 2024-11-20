use cushy::{
    figures::{units::Lp, Size},
    styles::{Dimension, DimensionRange},
    value::Source,
    widget::MakeWidget,
    widgets::{Button, Image, Label},
};
use itertools::Itertools;
use librespot_metadata::audio::UniqueFields;

use crate::{
    player::{DynamicPlayer, PlayerState},
    widgets::image::ImageExt,
};

pub fn bar(player: DynamicPlayer) -> impl MakeWidget {
    meta(player).size(Size {
        width: DimensionRange::default(),
        height: Dimension::Lp(Lp::inches_f(1.5)).into(),
    })
}

fn meta(player: DynamicPlayer) -> impl MakeWidget {
    Image::new_empty()
        .with_url(player.track.map_each(|track| {
            track
                .as_ref()
                .map(|track| track.covers.first().map(|cover| cover.url.clone()))
                .flatten()
        }))
        .size(Size::squared(Dimension::Lp(Lp::inches_f(1.))))
        .and(
            player
                .track
                .map_each(|track| {
                    track
                        .as_ref()
                        .map(|track| {
                            track
                                .name
                                .clone()
                                .and(match &track.unique_fields {
                                    UniqueFields::Track {
                                        artists,
                                        album,
                                        album_artists,
                                        popularity,
                                        number,
                                        disc_number,
                                    } => {
                                        artists.iter().map(|artist| artist.name.clone()).join(", ")
                                    }
                                    UniqueFields::Episode {
                                        description,
                                        publish_time,
                                        show_name,
                                    } => show_name.clone(),
                                })
                                .into_rows()
                                .make_widget()
                        })
                        .unwrap_or(Label::<String>::new("No track found").make_widget())
                })
                .expand(),
        )
        .into_columns()
}

fn controls(player: DynamicPlayer) -> impl MakeWidget {
    "shuffle"
        .into_button()
        .and("previous".into_button())
        .and(player.state.map_each(|state| {
            match state {
                PlayerState::Playing => "pause",
                PlayerState::Paused { .. } => "play",
                _ => "play",
            }
            .into_button()
            .make_widget()
        }))
        .and("skip".into_button())
        .and("repeat".into_button())
        .into_columns()
        .and(progress(player))
        .into_rows()
}

fn progress(player: DynamicPlayer) -> impl MakeWidget {
    "progress bar here"
}

fn vol(player: DynamicPlayer) -> impl MakeWidget {
    "vol control here"
}
