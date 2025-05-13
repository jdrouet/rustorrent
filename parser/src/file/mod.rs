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

/// Represents the `info` dictionary within a `.torrent` file, which contains
/// metadata about the files being shared, including the file names, sizes,
/// piece length, and hash information.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentInfo {
    /// The name of the file or directory (used as the base path)
    pub name: String,

    /// Piece size in bytes (each file is split into pieces of this length)
    #[serde(rename = "piece length")]
    pub piece_length: u64,

    /// Concatenated SHA1 hashes of each piece (20 bytes per piece)
    #[serde(with = "serde_bytes")]
    pub pieces: serde_bytes::ByteBuf,

    /// 1 if private torrent (disables DHT/PEX)
    #[serde(default)]
    pub private: Option<u8>,

    /// MD5 checksum of file (rarely used; deprecated)
    #[serde(default)]
    pub md5sum: Option<String>,

    /// Either a single file or a list of files (multi-file mode)
    #[serde(flatten)]
    pub content: TorrentInfoContent,
}

/// Enum representing the content of the torrent. It can be either a single file
/// or multiple files.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TorrentInfoContent {
    /// For single-file torrents: includes total length (and optional md5sum)
    Single {
        length: u64,

        #[serde(default)]
        md5sum: Option<String>,
    },

    /// For multi-file torrents: list of files with individual metadata
    Multi { files: Vec<TorrentFileEntry> },
}

/// Represents a single file within a multi-file torrent, including metadata
/// like file length and path.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentFileEntry {
    /// Size of the file in bytes
    pub length: u64,

    /// Path components (e.g. ["folder", "file.txt"])
    pub path: Vec<String>,

    /// MD5 checksum for this file (rarely used)
    #[serde(default)]
    pub md5sum: Option<String>,
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
        assert_eq!(file.info.name, "test_folder");

        assert_eq!(file.info.piece_length, 32768);
        let TorrentInfoContent::Multi { files } = file.info.content else {
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
        assert_eq!(file.info.name, "ubuntu-25.04-desktop-amd64.iso");

        assert_eq!(file.info.piece_length, 262144);
        let TorrentInfoContent::Single { length, md5sum: _ } = file.info.content else {
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
        assert_eq!(file.info.name, "bittorrent-v1-v2-hybrid-test");

        assert_eq!(file.info.piece_length, 524288);
        let TorrentInfoContent::Multi { files } = file.info.content else {
            panic!("should be multiple files");
        };

        assert_eq!(files.len(), 17);
    }
}
