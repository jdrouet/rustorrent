use std::path::PathBuf;

/// This section contains the field which are common to both mode, "single file"
/// and "multiple file".
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentInfo {
    #[serde(flatten)]
    pub base: super::TorrentInfoBase,

    #[serde(flatten)]
    pub fields: TorrentInfoFields,
}

/// This section contains the field which are common to both mode, "single file"
/// and "multiple file".
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentInfoFields {
    /// Concatenated SHA1 hashes of each piece (20 bytes per piece)
    #[serde(with = "serde_bytes")]
    pub pieces: serde_bytes::ByteBuf,

    /// Either a single file or a list of files (multi-file mode)
    #[serde(flatten)]
    pub content: TorrentInfoContent,
}

impl TorrentInfoFields {
    pub fn file_iter(&self, base_name: &str) -> impl Iterator<Item = PathBuf> + '_ {
        match self.content {
            TorrentInfoContent::File { .. } => {
                TorrentIterator::Single(std::iter::once(PathBuf::from(base_name)))
            }
            TorrentInfoContent::Directory { ref files } => {
                TorrentIterator::Multi(files.iter().map(TorrentFileEntry::path))
            }
        }
    }
}
enum TorrentIterator<'a> {
    Single(std::iter::Once<PathBuf>),
    Multi(
        std::iter::Map<std::slice::Iter<'a, TorrentFileEntry>, fn(&'a TorrentFileEntry) -> PathBuf>,
    ),
}

impl Iterator for TorrentIterator<'_> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(inner) => inner.next(),
            Self::Multi(inner) => inner.next(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TorrentInfoContent {
    File {
        /// The length of the file in bytes
        length: u64,

        /// A 32-character hexadecimal string corresponding to the MD5 sum of
        /// the file.
        ///
        /// This is not used by BitTorrent at all, but it is included by some
        /// programs for greater compatibility
        #[serde(default)]
        md5sum: Option<String>,
    },

    Directory {
        /// The content of the directory
        files: Vec<TorrentFileEntry>,
    },
}

impl TorrentInfoContent {
    pub fn file_count(&self) -> usize {
        match self {
            Self::File { .. } => 1,
            Self::Directory { files } => files.len(),
        }
    }
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

impl TorrentFileEntry {
    pub fn path(&self) -> PathBuf {
        self.path
            .iter()
            .fold(PathBuf::default(), |acc, part| acc.join(part))
    }
}
