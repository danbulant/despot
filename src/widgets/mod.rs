use cushy::value::Dynamic;
use rspotify::model::{SimplifiedAlbum, SimplifiedPlaylist};

pub mod image;
pub mod library;
pub mod owned;
pub mod pages;

#[derive(PartialEq, Debug, Default)]
pub enum ActivePage {
    #[default]
    LikedSongs,
    Playlist(SimplifiedPlaylist),
    Album(SimplifiedAlbum)
}

type SelectedPage = Dynamic<ActivePage>;