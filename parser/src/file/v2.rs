use std::collections::BTreeMap;
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

    pub fn file_iter(&self) -> impl Iterator<Item = (PathBuf, &TorrentFileEntry)> {
        FileTreeIterator::new(PathBuf::new(), &self.file_tree)
    }
}

pub struct FileTreeIterator<'a> {
    stack: Vec<(
        PathBuf,
        std::collections::btree_map::Iter<'a, String, FileTreeNode>,
    )>,
}

impl<'a> FileTreeIterator<'a> {
    fn new(path: PathBuf, node: &'a BTreeMap<String, FileTreeNode>) -> Self {
        Self {
            stack: vec![(path, node.iter())],
        }
    }
}

impl<'a> Iterator for FileTreeIterator<'a> {
    type Item = (PathBuf, &'a TorrentFileEntry);

    fn next(&mut self) -> Option<Self::Item> {
        let (path, mut node) = self.stack.pop()?;
        if let Some((name, entry)) = node.next() {
            match entry {
                FileTreeNode::File(inner) => {
                    self.stack.push((path.clone(), node));
                    return Some((path, inner));
                }
                FileTreeNode::Directory(inner) => {
                    let dir_path = path.join(name);
                    self.stack.push((path, node));
                    self.stack.push((dir_path, inner.iter()));
                }
            }
        }
        self.next()
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
