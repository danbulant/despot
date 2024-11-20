use std::time::Duration;

use cushy::{
    figures::{units::Lp, Size},
    styles::{Dimension, DimensionRange},
    value::{Dynamic, Source},
    widget::MakeWidget,
    widgets::{Button, Image, Label, Slider},
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
        height: Dimension::Lp(Lp::inches_f(1.)).into(),
    })
}

fn meta(player: DynamicPlayer) -> impl MakeWidget {
    Image::new_empty()
        .with_url(player.track.map_each(|track| {
            dbg!(track
                .as_ref()
                .map(|track| track.covers.first().map(|cover| cover.url.clone()))
                .flatten())
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
        .align_left()
        .expand()
        .and(controls(player.clone()).expand())
        .and(vol(player).align_right().expand())
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
        .centered()
        .and(time(player))
        .into_rows()
}

fn time(player: DynamicPlayer) -> impl MakeWidget {
    let duration = player.track.map_each(|track| {
        track
            .as_ref()
            .map(|track| track.duration_ms as f64 / 1000.)
            .unwrap_or(0.)
    });
    let slider = Slider::from_value(player.track_progress.map_each(|progress| {
        progress
            .map(|progress| progress.as_secs_f64())
            .unwrap_or(0.)
    }))
    .minimum(0.)
    .maximum(duration.clone())
    .size(Size::<DimensionRange> {
        height: Dimension::Lp(Lp::inches_f(0.2)).into(),
        width: (..Dimension::Lp(Lp::inches_f(5.))).into(),
    });
    player
        .track_progress
        .map_each(|progress| {
            progress
                .map(|progress| format_time(progress))
                .unwrap_or_else(|| "0:00".to_string())
        })
        .and(slider.expand_horizontally())
        .and(duration.map_each(|duration| format_time(Duration::from_secs_f64(*duration))))
        .into_columns()
        .expand_horizontally()
        .centered()
}

fn format_time(time: Duration) -> String {
    let time = time.as_secs_f64();
    let seconds = time % 60.;
    let minutes = time / 60.;
    format!("{}:{:02}", minutes.round(), seconds.round())
}

fn vol(player: DynamicPlayer) -> impl MakeWidget {
    "vol control here"
}
