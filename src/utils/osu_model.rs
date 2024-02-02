use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct OsuUser {
    pub id: u64,
    pub username: String,
}

pub type Recent = Vec<RecentElement>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentElement {
    pub id: u64,
    pub user_id: u64,
    pub accuracy: f64,
    pub mods: Vec<String>,
    pub score: u128,
    pub max_combo: u64,
    pub perfect: bool,
    pub statistics: Statistics,
    pub rank: String,
    pub created_at: DateTime<Utc>,
    pub best_id: Option<u64>,
    pub pp: Option<f64>,
    pub mode: String,
    pub mode_int: u16,
    pub replay: bool,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Beatmap {
    pub difficulty_rating: f64,
    pub id: u64,
    pub mode: String,
    pub mode_int: u16,
    pub status: String,
    pub total_length: u64, // seconds
    pub user_id: u64,
    pub version: String,
    pub accuracy: f64,
    pub ar: f64,
    pub beatmapset_id: u64,
    pub bpm: f64,
    pub convert: bool,
    pub count_circles: u64,
    pub count_sliders: u64,
    pub count_spinners: u64,
    pub cs: f64,
    pub deleted_at: Option<DateTime<Utc>>,
    pub drain: f64,
    pub hit_length: u64, // playtime seconds
    pub is_scoreable: bool,
    pub last_updated: DateTime<Utc>,
    pub passcount: u64,
    pub playcount: u64,
    pub ranked: i8,
    pub url: String,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub covers: Covers,
    pub creator: String,
    pub favourite_count: u64,
    pub hype: Option<Hype>,
    pub id: u64,
    pub nsfw: bool,
    pub play_count: u64,
    pub preview_url: String,
    pub source: String,
    pub status: String,
    pub title: String,
    pub title_unicode: String,
    pub user_id: u64,
    pub video: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Covers {
    pub cover: String,
    #[serde(rename = "cover@2x")]
    pub cover_2x: String,
    pub card: String,
    #[serde(rename = "card@2x")]
    pub card_2x: String,
    pub list: String,
    #[serde(rename = "list@2x")]
    pub list_2x: String,
    pub slimcover: String,
    #[serde(rename = "slimcover@2x")]
    pub slimcover_2x: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hype {
    pub current: u32,
    pub required: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub count_50: u64,
    pub count_100: u64,
    pub count_300: u64,
    pub count_geki: Option<u64>,
    pub count_katu: Option<u64>,
    pub count_miss: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub avatar_url: String,
    pub country_code: String,
    pub default_group: String,
    pub id: u64,
    pub is_active: bool,
    pub is_bot: bool,
    pub is_deleted: bool,
    pub is_online: bool,
    pub is_supporter: bool,
    pub last_visit: Option<DateTime<Utc>>,
    pub pm_friends_only: bool,
    pub profile_colour: Option<serde_json::Value>,
    pub username: String,
}
