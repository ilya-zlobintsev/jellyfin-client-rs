use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserInfo {
    pub name: String,
    pub server_id: String,
    pub id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthResponse {
    pub user: UserInfo,
    pub access_token: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemsResponse<I> {
    pub items: Vec<I>,
    pub total_record_count: i64,
    pub start_index: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub name: String,
    pub id: String,
    pub collection_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MusicAlbum {
    pub name: String,
    pub id: String,
    pub premiere_date: Option<DateTime<Utc>>,
    pub artists: Vec<String>,
    pub artist_items: Vec<ArtistItem>,
    pub album_artist: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArtistItem {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Audio {
    pub name: String,
    pub id: String,
    pub premiere_date: Option<DateTime<Utc>>,
    pub artists: Vec<String>,
    pub artist_items: Vec<ArtistItem>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub album_artist: Option<String>,
    #[serde(rename = "RunTimeTicks")]
    pub runtime_ticks: Option<u64>,
}

#[derive(Deserialize)]
#[serde(tag = "Type")]
pub enum ItemType {
    MusicArtist(Item),
    Audio(Audio),
    MusicAlbum(MusicAlbum),
    Playlist(Item),
}

impl ItemType {
    pub fn as_str(&self) -> &str {
        match self {
            ItemType::MusicArtist(_) => "MusicArtist",
            ItemType::Audio(_) => "Audio",
            ItemType::MusicAlbum(_) => "MusicAlbum",
            ItemType::Playlist(_) => "Playlist",
        }
    }
}

pub enum ImageType {
    Primary,
}

impl ImageType {
    pub fn as_str(&self) -> &str {
        match self {
            ImageType::Primary => "Primary",
        }
    }
}
