use cushy::value::Dynamic;
use rspotify::model::{SimplifiedAlbum, SimplifiedPlaylist};

pub mod image;
pub mod library;

#[derive(PartialEq, Debug, Default)]
pub enum ActivePage {
    #[default]
    LikedSongs,
    Playlist(SimplifiedPlaylist),
    Album(SimplifiedAlbum)
}

type SelectedPage = Dynamic<ActivePage>;