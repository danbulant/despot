use cushy::{styles::components::WidgetBackground, value::{Destination, Dynamic, IntoDynamic, IntoValue, Source, Value}, widget::{MakeWidget, WidgetList}, widgets::{grid::Orientation, Image, Stack}};
use rspotify::model::SimplifiedPlaylist;
use cushy::kludgine::Color;

use crate::widgets::{image::ImageExt, ActivePage, SelectedPage};

fn playlist_entry(playlist: impl IntoValue<SimplifiedPlaylist>, selected_page: SelectedPage) -> impl MakeWidget {
    let playlist: Value<SimplifiedPlaylist> = playlist.into_value();
    let id = playlist.map(|p| p.id.clone());
    let background = selected_page.map_each(move |page| {
        match page {
            ActivePage::Playlist(p) if p.id == id => {
                Color(0xFFFFFF10)
            }
            _ => Color::CLEAR_WHITE
        }
    });
    Image::new_empty()
        .with_url(
            playlist
                .map_each(|playlist| playlist.images.first().map(|image| image.url.clone()))
                .into_dynamic()
        )

    .and(
        playlist
            .map_each(|p| p.name.clone())
            .align_left()
            .expand()
    )
    .into_columns()
    .into_button()
    .on_click(move |_| {
        selected_page.set(ActivePage::Playlist(playlist.get()));
    })
    .with(&WidgetBackground, background)
}

pub fn playlists_widget(playlists: impl IntoValue<Vec<SimplifiedPlaylist>>, selected_page: SelectedPage) -> impl MakeWidget {
    let playlists: Value<Vec<SimplifiedPlaylist>> = playlists.into_value();
    Stack::new(
        Orientation::Row,
        playlists
            .map_each(move |t| {
                let mut list = t.clone().into_iter().map(|playlist| playlist_entry(playlist, selected_page.clone())).collect::<WidgetList>();
                list.insert(0, liked_songs_entry(selected_page.clone()));
                list
            })
    )
    .vertical_scroll()
    .expand()
}

pub fn liked_songs_entry(selected_page: SelectedPage) -> impl MakeWidget {
    let background = selected_page.map_each(move |page| {
        match page {
            ActivePage::LikedSongs => {
                Color(0xFFFFFF10)
            }
            _ => Color::CLEAR_WHITE
        }
    });
    Image::new_empty()
        .with_url(
            Dynamic::new(Some("https://misc.scdn.co/liked-songs/liked-songs-300.png".to_string()))
        )
    .and(
        "Liked Songs"
            .align_left()
            .expand()
    )
    .into_columns()
    .into_button()
    .on_click(move |_| {
        selected_page.set(ActivePage::LikedSongs);
    })
    .with(&WidgetBackground, background)
}