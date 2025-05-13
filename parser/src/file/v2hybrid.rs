#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentInfo {
    #[serde(flatten)]
    pub base: super::TorrentInfoBase,
    #[serde(flatten)]
    pub v1: super::v1::TorrentInfoFields,
    #[serde(flatten)]
    pub v2: super::v2::TorrentInfoFields,
}
