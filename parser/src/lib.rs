#![doc = include_str!("../readme.md")]

use std::str::FromStr;

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
    pub announce: String,

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

/// Represents errors that may occur while parsing magnet links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MagnetLinkParserError {
    InvalidUrl(url::ParseError),
    InvalidScheme,
    MissingInfoHash,
}

impl From<url::ParseError> for MagnetLinkParserError {
    fn from(value: url::ParseError) -> Self {
        Self::InvalidUrl(value)
    }
}

impl std::fmt::Display for MagnetLinkParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(inner) => inner.fmt(f),
            Self::InvalidScheme => write!(f, "invalid scheme, expected \"magnet\""),
            Self::MissingInfoHash => write!(f, "missing xt parameter for info_hash attribute"),
        }
    }
}

impl std::error::Error for MagnetLinkParserError {}

/// Represents a parsed Magnet URI, which includes the info hash, display name,
/// trackers, and web seeds.
#[derive(Debug)]
pub struct MagnetLink {
    /// The 40-character hexadecimal BitTorrent info hash (unique identifier for
    /// the torrent).
    pub info_hash: String,
    /// A human-readable display name (e.g. for UI display).
    pub display_name: Option<String>,
    /// List of tracker URLs (announces) provided in the URI.
    pub trackers: Vec<String>,
    /// List of web seed URLs from the `ws` parameter (for HTTP-based seeding).
    pub web_seeds: Vec<String>,
    /// Any additional parameters (e.g. web seeds, peer sources, etc).
    pub params: Vec<(String, String)>,
}

impl FromStr for MagnetLink {
    type Err = MagnetLinkParserError;

    fn from_str(uri: &str) -> Result<Self, MagnetLinkParserError> {
        let url = url::Url::parse(uri)?;
        if url.scheme() != "magnet" {
            return Err(MagnetLinkParserError::InvalidScheme);
        }

        let mut info_hash = None;
        let mut display_name = None;
        let mut trackers = Vec::new();
        let mut web_seeds = Vec::new();
        let mut params = Vec::new();

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "xt" if value.starts_with("urn:btih:") => {
                    info_hash = Some(value.trim_start_matches("urn:btih:").into());
                }
                "dn" => {
                    display_name = Some(value.into());
                }
                "tr" => {
                    trackers.push(value.into());
                }
                "ws" => {
                    web_seeds.push(value.into());
                }
                _ => {
                    params.push((key.into(), value.into()));
                }
            }
        }

        let info_hash = info_hash.ok_or(MagnetLinkParserError::MissingInfoHash)?;

        Ok(MagnetLink {
            info_hash,
            display_name,
            trackers,
            web_seeds,
            params,
        })
    }
}

/// Represents errors when processing hash bytes (info hashes).
#[derive(Debug, PartialEq)]
pub enum HashBytesError {
    UnsupportedLength,
    InvalidHex(hex::FromHexError),
    InvalidBase32,
    InvalidLength,
}

impl std::fmt::Display for HashBytesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedLength => write!(f, "unsupported info_hash length"),
            Self::InvalidHex(_) => write!(f, "invalid hex info_hash"),
            Self::InvalidBase32 => write!(f, "invalid base32 info_hash"),
            Self::InvalidLength => write!(f, "invalid SHA1 hash length, expected 20 bytes"),
        }
    }
}

impl std::error::Error for HashBytesError {}

impl MagnetLink {
    /// Converts the `info_hash` (a hexadecimal or Base32 string) to a 20-byte
    /// SHA1 hash.
    ///
    /// This function will take the `info_hash` from the magnet link, which can
    /// be provided in either hexadecimal or Base32 encoding, and convert it
    /// to a fixed-length 20-byte array representing the SHA1 hash.
    ///
    /// # Returns
    ///
    /// Returns a `Result`:
    /// - `Ok([u8; 20])`: A 20-byte array representing the decoded SHA1 hash of
    ///   the torrent info hash.
    /// - `Err(HashBytesError)`: An error if the `info_hash` is of an
    ///   unsupported length, has invalid hexadecimal characters, or is
    ///   improperly formatted in Base32.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    ///
    /// - `HashBytesError::UnsupportedLength`: The `info_hash` is neither 32 nor
    ///   40 characters in length.
    /// - `HashBytesError::InvalidHex`: The `info_hash` contains invalid
    ///   hexadecimal characters when the input is expected to be hexadecimal.
    /// - `HashBytesError::InvalidBase32`: The `info_hash` is in an invalid
    ///   Base32 format when the input is expected to be Base32.
    /// - `HashBytesError::InvalidLength`: The decoded value is not 20 bytes in
    ///   length, which is the expected size for a SHA1 hash.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustorrent_parser::MagnetLink;
    ///
    /// let magnet_link = MagnetLink {
    ///     info_hash: "d6a67b7e10b219d01f84c1c99962f060c18bb658".to_string(),
    ///     display_name: None,
    ///     trackers: Vec::new(),
    ///     web_seeds: Vec::new(),
    ///     params: Vec::new(),
    /// };
    ///
    /// let hash = magnet_link.hash_bytes();
    /// assert!(hash.is_ok());
    /// assert_eq!(hash.unwrap().len(), 20);
    /// ```
    pub fn hash_bytes(&self) -> Result<[u8; 20], HashBytesError> {
        let cleaned = self.info_hash.to_ascii_lowercase();

        if cleaned.len() == 40 {
            let decoded = hex::decode(&cleaned).map_err(HashBytesError::InvalidHex)?;
            decoded
                .try_into()
                .map_err(|_| HashBytesError::InvalidLength)
        } else if cleaned.len() == 32 {
            let decoded = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &cleaned)
                .ok_or(HashBytesError::InvalidBase32)?;
            decoded
                .try_into()
                .map_err(|_| HashBytesError::InvalidLength)
        } else {
            Err(HashBytesError::UnsupportedLength)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_multifile_torrent() {
        let torrent = include_bytes!("../asset/academictorrent-multifile.torrent");
        let file = TorrentFile::from_bytes(torrent).unwrap();
        assert_eq!(file.announce, "https://academictorrents.com/announce.php");
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
    fn should_parse_ubuntu_torrent() {
        let torrent = include_bytes!("../asset/ubuntu-25.04-desktop-amd64.iso.torrent");
        let file = TorrentFile::from_bytes(torrent).unwrap();
        assert_eq!(file.announce, "https://torrent.ubuntu.com/announce");
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
    fn should_parse_academic_link() {
        let url = "magnet:?xt=urn:btih:d984f67af9917b214cd8b6048ab5624c7df6a07a&tr=https%3A%2F%2Facademictorrents.com%2Fannounce.php&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce";
        let magnet = crate::MagnetLink::from_str(url).unwrap();
        assert_eq!(magnet.info_hash, "d984f67af9917b214cd8b6048ab5624c7df6a07a");
        assert_eq!(magnet.display_name, None);
        assert_eq!(magnet.trackers.len(), 3);
        assert!(magnet.web_seeds.is_empty());
        assert!(magnet.params.is_empty());
    }
}
