# Torrent Parser

A Rust library for parsing BitTorrent `.torrent` files and magnet URIs. It provides functionality to extract essential metadata from `.torrent` files and magnet links, such as tracker URLs, file names, and SHA1 info hashes.

This library supports:
- Parsing `.torrent` files (bencoded format)
- Parsing and handling magnet URIs
- Extracting file metadata from torrent files and magnet links
- Converting info hashes between hexadecimal and Base32 formats

## Features

- **Torrent File Parsing**: Deserialize `.torrent` files and retrieve metadata such as the main and backup tracker URLs, creation date, file content, and more.
- **Magnet Link Parsing**: Parse magnet URIs to extract the info hash, tracker URLs, display name, web seeds, and other parameters.
- **Info Hash Conversion**: Convert the `info_hash` from a magnet link into a 20-byte SHA1 hash (supporting both hexadecimal and Base32 encoding).
- **Comprehensive Error Handling**: Detailed errors for unsupported formats or invalid input.

## Usage

### Add to `Cargo.toml`

```toml
[dependencies]
rustorrent-parser = "0.1.0"
````

### Example Usage

```rust,no_run
use rustorrent_parser::file::TorrentFile;
use rustorrent_parser::magnet::{MagnetLink, ParseError};
use std::str::FromStr;

fn _main() {
    let torrent_data = std::fs::read("file.torrent").expect("Failed to read torrent file");
    let torrent: TorrentFile = TorrentFile::from_bytes(&torrent_data).expect("Failed to parse torrent");

    println!("Announce URL: {}", torrent.announce);
    println!("File/Directory Name: {}", torrent.info.name);

    let magnet_uri = "magnet:?xt=urn:btih:d6a67b7e10b219d01f84c1c99962f060c18bb658&dn=example";
    let magnet = MagnetLink::from_str(magnet_uri).expect("Failed to parse magnet link");

    println!("Magnet Info Hash: {}", magnet.info_hash);

    let hash_bytes = magnet.hash_bytes().expect("Failed to convert hash");
    println!("Info Hash as Bytes: {:?}", hash_bytes);
}
```

## Functions

### `TorrentFile::from_bytes`

Parses a `.torrent` file from raw bytes (in bencoded format) into a `TorrentFile` structure.

```rust,ignore
pub fn from_bytes(data: &[u8]) -> serde_bencode::Result<Self>
```

### `MagnetLink::from_str`

Parses a magnet URI into a `MagnetLink` structure, extracting the info hash, trackers, web seeds, and additional parameters.

```rust,ignore
pub fn from_str(uri: &str) -> Result<Self, MagnetLinkParserError>
```

### `MagnetLink::hash_bytes`

Converts the `info_hash` from a magnet link (either in hexadecimal or Base32 format) to a 20-byte SHA1 hash.

```rust,ignore
pub fn hash_bytes(&self) -> Result<[u8; 20], HashBytesError>
```

## Error Handling

This library uses custom error types to handle various parsing failures:

* `MagnetLinkParserError`: Errors that occur when parsing magnet URIs.
* `HashBytesError`: Errors related to converting the `info_hash` to a byte array.
* `TorrentFileParseError`: Errors that can occur during the parsing of `.torrent` files.

## Supported Formats

* `.torrent` files: Bencoded format, including fields like `announce`, `announce-list`, `creation date`, `info` (file metadata), and more.
* Magnet URIs: Standard BitTorrent magnet link format, including `xt` (info hash), `dn` (display name), `tr` (trackers), and `ws` (web seeds).

## License

This project is licensed under the MIT License
