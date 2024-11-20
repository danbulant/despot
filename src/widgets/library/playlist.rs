use cushy::figures::units::Lp;
use cushy::figures::Size;
use cushy::kludgine::Color;
use cushy::styles::{CornerRadii, Dimension, DimensionRange};
use cushy::widgets::image::ImageCornerRadius;
use cushy::{
    value::{Destination, Dynamic, IntoDynamic, IntoValue, Source, Value},
    widget::{MakeWidget, WidgetList},
    widgets::{
        button::{ButtonBackground, ButtonClick, ButtonHoverBackground},
        grid::Orientation,
        Image, Stack,
    },
};
use rspotify::model::SimplifiedPlaylist;

use crate::{
    theme::{LIBRARY_BG, LIBRARY_BG_HOVER, LIBRARY_BG_SELECTED, LIBRARY_BG_SELECTED_HOVER},
    widgets::{image::ImageExt, ActivePage, SelectedPage},
};

fn playlist_entry(
    playlist: impl IntoValue<SimplifiedPlaylist>,
    selected_page: SelectedPage,
) -> impl MakeWidget {
    let playlist: Value<SimplifiedPlaylist> = playlist.into_value();
    let id = playlist.map(|p| p.id.clone());
    let is_active =
        selected_page.map_each(move |page| matches!(page, ActivePage::Playlist(p) if p.id == id));
    entry(
        playlist.map_each(|p| p.name.clone()).into_dynamic(),
        playlist
            .map_each(|playlist| playlist.images.first().map(|image| image.url.clone()))
            .into_dynamic(),
        is_active,
        move |_| {
            selected_page.set(ActivePage::Playlist(playlist.get()));
        },
    )
}

pub fn playlists_widget(
    playlists: impl IntoValue<Vec<SimplifiedPlaylist>>,
    selected_page: SelectedPage,
) -> impl MakeWidget {
    let playlists: Value<Vec<SimplifiedPlaylist>> = playlists.into_value();
    Stack::new(
        Orientation::Row,
        playlists.map_each(move |t| {
            let mut list = t
                .clone()
                .into_iter()
                .map(|playlist| playlist_entry(playlist, selected_page.clone()))
                .collect::<WidgetList>();
            list.insert(0, liked_songs_entry(selected_page.clone()));
            list
        }),
    )
    .vertical_scroll()
    .size(Size {
        width: Dimension::Lp(Lp::points(200)).into(),
        height: DimensionRange::default(),
    })
}

fn liked_songs_entry(selected_page: SelectedPage) -> impl MakeWidget {
    let is_active = selected_page.map_each(|page| matches!(page, ActivePage::LikedSongs));
    entry(
        "Liked Songs",
        Dynamic::new(Some(
            "https://misc.scdn.co/liked-songs/liked-songs-300.png".to_string(),
        )),
        is_active,
        move |_| {
            selected_page.set(ActivePage::LikedSongs);
        },
    )
}

fn entry<F>(
    text: impl IntoValue<String>,
    url: Dynamic<Option<String>>,
    is_active: Dynamic<bool>,
    callback: F,
) -> impl MakeWidget
where
    F: FnMut(Option<ButtonClick>) + Send + 'static,
{
    let (background, background_hover) = get_colors(is_active);
    Image::new_empty()
        .with_url(url)
        .with(&ImageCornerRadius, Dimension::Lp(Lp::points(4)))
        .size(Size::squared(Dimension::Lp(Lp::points(40))))
        .and(text.into_value().align_left())
        .into_columns()
        .into_button()
        .on_click(callback)
        .with(&ButtonBackground, background)
        .with(&ButtonHoverBackground, background_hover)
        .pad()
}

/// Returns `background` and `background_hover` colors for a library entry.
fn get_colors(is_active: impl IntoValue<bool>) -> (Value<Color>, Value<Color>) {
    let is_active = is_active.into_value();
    let background = is_active.map_each(|active| {
        if *active {
            LIBRARY_BG_SELECTED
        } else {
            LIBRARY_BG
        }
    });
    let background_hover = is_active.map_each(|active| {
        if *active {
            LIBRARY_BG_SELECTED_HOVER
        } else {
            LIBRARY_BG_HOVER
        }
    });
    (background, background_hover)
}
