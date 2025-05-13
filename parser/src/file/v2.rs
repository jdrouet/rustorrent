use std::collections::BTreeMap;

/// This section contains the field which are common to both mode, "single file"
/// and "multiple file".
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentInfo {
    #[serde(flatten)]
    pub base: super::TorrentInfoBase,

    #[serde(flatten)]
    pub fields: TorrentInfoFields,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TorrentInfoFields {
    /// Must be set to 2 to indicate v2 format
    #[serde(rename = "meta version")]
    pub meta_version: u8,

    /// The file tree structure
    #[serde(rename = "file tree")]
    pub file_tree: BTreeMap<String, FileTreeNode>,
}

impl TorrentInfoFields {
    pub fn file_count(&self) -> usize {
        self.file_tree.values().map(|node| node.file_count()).sum()
    }
}

/// Represents a file node or a directory node within the file tree
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum FileTreeNode {
    File(TorrentFileEntry),
    Directory(BTreeMap<String, FileTreeNode>),
}

impl FileTreeNode {
    pub fn file_count(&self) -> usize {
        match self {
            Self::File(_) => 1,
            Self::Directory(inner) => inner.values().map(|node| node.file_count()).sum(),
        }
    }
}

/// File metadata for each file entry in the file tree
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TorrentFileEntry {
    /// Length in bytes of the file
    pub length: u64,

    /// Optional file-specific SHA256 root (Merkle root hash)
    #[serde(rename = "pieces root", with = "serde_bytes")]
    pub pieces_root: serde_bytes::ByteBuf,

    /// Optional MD5 checksum for compatibility (not used in v2, but may appear)
    #[serde(default)]
    pub md5sum: Option<String>,
}
