pub mod v1;
pub mod v2;
pub mod v2hybrid;

/// Represents the structure of a parsed BitTorrent `.torrent` file.
/// The `TorrentFile` structure includes metadata about the torrent file itself,
/// such as the announce URL, creation date, and information about the files in
/// the torrent.
///
/// This struct is deserialized from the bencoded representation of a `.torrent`
/// file.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentFile {
    /// The main tracker URL for the torrent
    ///
    /// In v1, this shouldn't be optional. Keeping this optional anyway for
    /// simplicity.
    pub announce: Option<String>,

    /// List of backup trackers (multi-tiered tracker support)
    #[serde(default, rename = "announce-list")]
    pub announce_list: Vec<Vec<String>>,

    /// Unix timestamp of when the torrent was created
    #[serde(default, rename = "creation date")]
    pub creation_date: Option<i64>,

    /// Comment embedded in the torrent (e.g. website or notes)
    #[serde(default)]
    pub comment: Option<String>,

    /// Creator client or tool name (e.g. "mktorrent")
    #[serde(default, rename = "created by")]
    pub created_by: Option<String>,

    /// Text encoding of strings (usually "UTF-8")
    #[serde(default)]
    pub encoding: Option<String>,

    /// The core metadata dictionary used to identify and download files
    pub info: TorrentInfo,
}

impl TorrentFile {
    /// Parse a `TorrentFile` from the raw bytes of a `.torrent` file (in
    /// bencoded format).
    ///
    /// # Parameters
    ///
    /// * `data`: The raw bytes representing a `.torrent` file in bencoded
    ///   format.
    ///
    /// # Returns
    ///
    /// A `Result` containing either a parsed `TorrentFile` or a
    /// `serde_bencode::Error` if the parsing fails.
    pub fn from_bytes(data: &[u8]) -> serde_bencode::Result<Self> {
        serde_bencode::from_bytes(data)
    }
}

/// Metadata dictionary to identify files
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TorrentInfo {
    V2Hybrid(v2hybrid::TorrentInfo),
    V2(v2::TorrentInfo),
    V1(v1::TorrentInfo),
}

/// Shared fields between all versions.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentInfoBase {
    /// Piece size in bytes (each file is split into pieces of this length)
    #[serde(rename = "piece length")]
    pub piece_length: u64,

    /// 1 if private torrent (disables DHT/PEX)
    #[serde(default)]
    pub private: Option<u8>,

    /// Name of the file or directory
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_v1_multifile() {
        let torrent = std::fs::read("asset/academictorrent-multifile.torrent").unwrap();
        let file = TorrentFile::from_bytes(&torrent).unwrap();
        assert_eq!(
            file.announce.as_deref(),
            Some("https://academictorrents.com/announce.php")
        );

        let TorrentInfo::V1(info) = file.info else {
            panic!("expected v1");
        };
        assert_eq!(info.base.piece_length, 32768);
        assert_eq!(info.base.name, "test_folder");
        let v1::TorrentInfoContent::Directory { files } = info.fields.content else {
            panic!("should be multi files");
        };

        assert_eq!(files[0].length, 17614527);
        assert_eq!(
            files[0].path,
            vec!["images", "LOC_Main_Reading_Room_Highsmith.jpg"]
        );
        assert_eq!(files[1].length, 1682177);
        assert_eq!(files[1].path, vec!["images", "melk-abbey-library.jpg"]);
        assert_eq!(files[2].length, 20);
        assert_eq!(files[2].path, vec!["README"]);
    }

    #[test]
    fn should_parse_v1_singlefile() {
        let torrent = std::fs::read("asset/ubuntu-25.04-desktop-amd64.iso.torrent").unwrap();
        let file = TorrentFile::from_bytes(&torrent).unwrap();
        assert_eq!(
            file.announce.as_deref(),
            Some("https://torrent.ubuntu.com/announce")
        );
        assert_eq!(file.announce_list.len(), 2);
        assert_eq!(file.creation_date, Some(1744895485));
        assert_eq!(
            file.comment.as_deref(),
            Some("Ubuntu CD releases.ubuntu.com")
        );
        assert_eq!(file.created_by.as_deref(), Some("mktorrent 1.1"));
        assert_eq!(file.encoding, None);

        let TorrentInfo::V1(info) = file.info else {
            panic!("expected v1");
        };
        assert_eq!(info.base.piece_length, 262144);
        assert_eq!(info.base.name, "ubuntu-25.04-desktop-amd64.iso");
        let v1::TorrentInfoContent::File { length, md5sum: _ } = info.fields.content else {
            panic!("should be single file");
        };

        assert_eq!(length, 6278520832);
    }

    #[test]
    fn should_parse_v2_hybrid() {
        let torrent = std::fs::read("asset/bittorrent-v2-hybrid-test.torrent").unwrap();
        let file = TorrentFile::from_bytes(&torrent).unwrap();
        assert_eq!(file.announce, None);
        assert!(file.announce_list.is_empty());
        assert_eq!(file.creation_date, Some(1591173906));
        assert_eq!(file.comment, None);
        assert_eq!(file.created_by.as_deref(), Some("libtorrent"));
        assert_eq!(file.encoding, None);

        let info = match file.info {
            TorrentInfo::V1(_) => panic!("expected v2 hybrid, got v1"),
            TorrentInfo::V2Hybrid(inner) => inner,
            TorrentInfo::V2(_) => panic!("expected v2 hybrid, got v2"),
        };
        assert_eq!(info.base.piece_length, 524288);
    }

    #[test]
    fn should_parse_v2() {
        let torrent = std::fs::read("asset/bittorrent-v2-test.torrent").unwrap();
        let file = TorrentFile::from_bytes(&torrent).unwrap();
        assert_eq!(file.announce, None);
        assert!(file.announce_list.is_empty());
        assert_eq!(file.creation_date, Some(1590097257));
        assert_eq!(file.comment, None);
        assert_eq!(file.created_by.as_deref(), Some("libtorrent"));
        assert_eq!(file.encoding, None);

        let info = match file.info {
            TorrentInfo::V1(_) => panic!("expected v2, got v1"),
            TorrentInfo::V2Hybrid(_) => panic!("expected v2, got v2 hybrid"),
            TorrentInfo::V2(inner) => inner,
        };
        assert_eq!(info.base.piece_length, 4194304);
    }
}
