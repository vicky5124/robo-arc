#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigurationData {
    pub old_osu: String,
    pub discord: String,
    pub twitch: String,
    pub twitch_client_id: String,
    pub trace_level: String,
    pub enable_tracing: bool,
    pub webhook_notifications: bool,

    pub presence: PresenceConfig,
    pub sankaku: SankakuConfig,
    pub lavalink: LavalinkConfig,
    pub web_server: WebServerConfig,
    pub ibm: IBMConfig,
    pub osu: OsuConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresenceConfig {
    pub play_or_listen: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SankakuConfig {
    pub idol_login: String,
    pub idol_passhash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LavalinkConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebServerConfig {
    pub server_ip: String,
    pub server_port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IBMConfig {
    pub token: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OsuConfig {
    pub client_id: u16,
    pub client_secret: String,
}
