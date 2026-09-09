//! The library as fetchable pieces: "deals from index N", and the URLs that
//! would answer it.
//!
//! A browser cannot download 105 MiB of tables to look at five deals, so
//! [rpdd-library] publishes them in pieces with a manifest describing where
//! each begins. This module is the part that knows that: which piece holds
//! deal N, where in it, how a run that crosses a boundary is stitched, and how
//! one that reaches the end of the library wraps round to the start. A caller
//! asks for deals and never computes a piece number, an offset or a wrap.
//!
//! [rpdd-library]: https://github.com/bridge-craftwork/rpdd-library
//!
//! # It asks; it never fetches
//!
//! Fetching is asynchronous and this is synchronous Rust, so a "give me chunk
//! 42" callback is not available — a wasm build cannot await from inside a
//! synchronous call. What is available is asking and being told:
//!
//! ```no_run
//! use rpdd_reader::{Library, LibraryError, RPDD_MANIFEST};
//! # fn fetch(_url: &str) -> Vec<u8> { unimplemented!() }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut library = Library::at(RPDD_MANIFEST);
//! let zrd = loop {
//!     match library.zrd(4_096_000, 500) {
//!         Ok(bytes) => break bytes,
//!         // The manifest on the first round, the chunks it named on the
//!         // second. The caller fetches URLs and never learns which is which.
//!         Err(LibraryError::Needs(urls)) => {
//!             for url in urls {
//!                 library.supply(&url, fetch(&url))?;
//!             }
//!         }
//!         Err(other) => return Err(other.into()),
//!     }
//! };
//! # let _ = zrd; Ok(()) }
//! ```
//!
//! The loop is the same for a page using `fetch`, a test using `include_bytes!`
//! and a command-line tool reading files. Nothing here opens a socket, and
//! nothing here can.
//!
//! # The layout is data, not code
//!
//! `deals_per_chunk`, `record_bytes`, `total_deals` and every chunk's `file`
//! and `first_deal` are read from the manifest. None of them is a constant
//! here. Our library happens to be 160 chunks of 65,536 deals, but that is a
//! hosting decision rather than a property of Pavlicek's library, and baking it
//! in would mean a new release of this crate every time the hosting changed.
//! Point [`Library::at`] at a different manifest and it works.
//!
//! [`RPDD_MANIFEST`] is therefore a constant a caller may pass, not a default
//! the logic falls back on.

use std::collections::HashMap;

use serde::Deserialize;

use crate::pairing::{pair, PairError, TABLE_LEN};

/// The manifest for the library published by [rpdd-library].
///
/// Offered for convenience and used by this crate's tests. It is not a default:
/// [`Library::at`] takes a URL and this module never reaches for this one on
/// its own.
///
/// This is the repository's own Cloudflare Pages deployment rather than
/// `raw.githubusercontent.com`, which is where it pointed first. Raw is a
/// source host: no edge caching, so every reader pays the round trip to
/// GitHub, and rate limits it was never meant to serve under. Measured from a
/// browser, a run reading two 640 KiB pieces spent 2.4 seconds getting them
/// against 0.05 running the script it wanted them for.
///
/// The Pages deployment also sets `Access-Control-Allow-Origin` and
/// `Cross-Origin-Resource-Policy`, which a consumer under COEP `require-corp`
/// — anything running threaded WebAssembly — needs in order to read the
/// pieces at all.
///
/// Raw still serves the same bytes and always will. The manifest's chunk paths
/// are relative, so either base resolves correctly and a mirror needs no
/// change here.
///
/// [rpdd-library]: https://github.com/bridge-craftwork/rpdd-library
pub const RPDD_MANIFEST: &str = "https://rpdd-library.pages.dev/manifest.json";

/// The only manifest schema this version understands.
///
/// Refusing an unknown one is the point of the field. A manifest that grew an
/// incompatible meaning for `first_deal` would otherwise be read confidently
/// and produce deals paired with the wrong tables.
pub const SCHEMA: u64 = 1;

/// What went wrong, or what is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryError {
    /// Not an error: fetch these URLs and hand the bytes back to
    /// [`Library::supply`], then ask again.
    ///
    /// The first round asks for the manifest, since until it is read there is
    /// nothing to say which chunks a range touches. The second asks for those
    /// chunks. A caller treats both the same way.
    Needs(Vec<String>),
    /// The manifest could not be read, or does not describe a usable library.
    Manifest(String),
    /// The manifest announces a schema this version does not understand.
    Schema {
        /// What the manifest said.
        found: u64,
        /// What this version reads.
        understood: u64,
    },
    /// More deals were asked for than the library holds.
    ///
    /// Running off the end wraps to the beginning, so any count up to the
    /// library's size is answerable; beyond that a run would visit a deal twice
    /// and the request is a mistake rather than a wrap.
    TooMany {
        /// How many deals were asked for.
        wanted: u64,
        /// How many the library holds.
        total_deals: u64,
    },
    /// Bytes were supplied for a URL this library did not ask for.
    Unexpected(String),
    /// A chunk arrived at the wrong length for what the manifest says it holds.
    ///
    /// Checked because a truncated or redirected download is otherwise silent:
    /// a `.zdd` has no framing, so short bytes just mean fewer deals than asked
    /// for, with nothing to say so.
    WrongLength {
        /// The chunk's URL.
        url: String,
        /// How many bytes the manifest says it holds.
        expected: usize,
        /// How many arrived.
        found: usize,
    },
    /// Tables and deals would not pair.
    Pairing(PairError),
}

impl core::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LibraryError::Needs(urls) => {
                write!(
                    f,
                    "needs {} URL(s), starting with {:?}",
                    urls.len(),
                    urls.first()
                )
            }
            LibraryError::Manifest(why) => write!(f, "manifest: {why}"),
            LibraryError::Schema { found, understood } => write!(
                f,
                "manifest schema {found}, but this reader understands schema {understood}"
            ),
            LibraryError::TooMany {
                wanted,
                total_deals,
            } => write!(
                f,
                "asked for {wanted} deals from a library of {total_deals}"
            ),
            LibraryError::Unexpected(url) => {
                write!(f, "bytes supplied for an unasked-for URL: {url}")
            }
            LibraryError::WrongLength {
                url,
                expected,
                found,
            } => write!(f, "{url} should be {expected} bytes, got {found}"),
            LibraryError::Pairing(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LibraryError {}

impl From<PairError> for LibraryError {
    fn from(e: PairError) -> Self {
        LibraryError::Pairing(e)
    }
}

/// One piece of the library, as the manifest describes it.
#[derive(Debug, Deserialize)]
struct ChunkEntry {
    /// Path to the chunk, relative to the manifest's own URL.
    file: String,
    /// The library index of the first deal whose table this chunk holds.
    first_deal: u64,
    /// How many tables it holds.
    deals: u64,
}

/// The manifest, as far as this reader is concerned.
///
/// Unknown fields — `sha256`, `note`, `source` — are ignored rather than
/// refused, so the manifest can carry more than a reader needs.
#[derive(Debug, Deserialize)]
struct Manifest {
    schema: u64,
    record_bytes: usize,
    deals_per_chunk: u64,
    total_deals: u64,
    chunks: Vec<ChunkEntry>,
}

/// A library described by a manifest, holding whatever pieces it has been given.
///
/// See the [module documentation](self) for the ask/supply loop.
pub struct Library {
    manifest_url: String,
    manifest: Option<Manifest>,
    /// Chunk ordinal to its bytes. Kept between calls, so a second run in the
    /// same region needs no fetching; [`Library::forget_chunks`] releases them.
    held: HashMap<usize, Vec<u8>>,
}

impl Library {
    /// A library described by the manifest at `manifest_url`.
    ///
    /// Nothing is read yet — the first call to [`zrd`](Self::zrd) or
    /// [`needs`](Self::needs) asks for the manifest.
    pub fn at(manifest_url: impl Into<String>) -> Self {
        Library {
            manifest_url: manifest_url.into(),
            manifest: None,
            held: HashMap::new(),
        }
    }

    /// The manifest URL this library was pointed at.
    pub fn manifest_url(&self) -> &str {
        &self.manifest_url
    }

    /// How many deals the library holds, once the manifest has been read.
    pub fn total_deals(&self) -> Option<u64> {
        self.manifest.as_ref().map(|m| m.total_deals)
    }

    /// Drop every chunk held, keeping the manifest.
    ///
    /// A page that has walked a long way through the library would otherwise
    /// accumulate every chunk it passed through.
    pub fn forget_chunks(&mut self) {
        self.held.clear();
    }

    /// The URLs still needed to answer `zrd(first_deal, count)`, in the order
    /// worth fetching them.
    ///
    /// Empty means [`zrd`](Self::zrd) will succeed. The list is the manifest
    /// on the first round and the chunks it names on the second — a caller
    /// fetches whatever it is given and does not distinguish.
    ///
    /// # Errors
    ///
    /// The manifest errors, and [`LibraryError::TooMany`]. Never
    /// [`LibraryError::Needs`]: what is needed is the return value.
    pub fn needs(&self, first_deal: u64, count: u64) -> Result<Vec<String>, LibraryError> {
        let Some(manifest) = self.manifest.as_ref() else {
            return Ok(vec![self.manifest_url.clone()]);
        };
        let mut urls = Vec::new();
        for (ordinal, _, _) in self.span(manifest, first_deal, count)? {
            if self.held.contains_key(&ordinal) {
                continue;
            }
            let Some(entry) = manifest.chunks.get(ordinal) else {
                return Err(missing_chunk(ordinal));
            };
            let url = resolve(&self.manifest_url, &entry.file);
            // A run may lap a small library and touch a chunk twice; it is
            // still one fetch.
            if !urls.contains(&url) {
                urls.push(url);
            }
        }
        Ok(urls)
    }

    /// `count` deals from `first_deal`, each with its double-dummy table, as
    /// `.zrd` records.
    ///
    /// A run that reaches the end of the library continues from its beginning,
    /// so any starting index answers — `first_deal` is taken modulo the
    /// library's size. A run that crosses a chunk boundary is stitched from
    /// both chunks and is not visible in the result.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Needs`] when something must be fetched first; see the
    /// [module documentation](self) for the loop.
    pub fn zrd(&self, first_deal: u64, count: u64) -> Result<Vec<u8>, LibraryError> {
        let needed = self.needs(first_deal, count)?;
        if !needed.is_empty() {
            return Err(LibraryError::Needs(needed));
        }
        // `needs` returning empty means the manifest is read and every chunk in
        // the span is held, so neither lookup below can miss.
        let Some(manifest) = self.manifest.as_ref() else {
            return Err(LibraryError::Manifest("no manifest".into()));
        };

        let mut out = Vec::new();
        for (ordinal, offset, take) in self.span(manifest, first_deal, count)? {
            let Some(bytes) = self.held.get(&ordinal) else {
                return Err(LibraryError::Manifest(format!(
                    "chunk {ordinal} was reported held and is not"
                )));
            };
            let from = offset as usize * TABLE_LEN;
            let to = from + take as usize * TABLE_LEN;
            // `supply` checked this chunk against the length the manifest
            // declares and `read_manifest` checked the declaration, so the
            // range fits. Sliced fallibly all the same: a panic out of a
            // library is a worse answer than an error, whatever the reasoning
            // that says it cannot happen.
            let (Some(tables), Some(entry)) = (bytes.get(from..to), manifest.chunks.get(ordinal))
            else {
                return Err(LibraryError::Manifest(format!(
                    "chunk {ordinal} does not hold deals {offset}..{}",
                    offset + take
                )));
            };
            out.extend_from_slice(&pair(tables, entry.first_deal + offset)?);
        }
        Ok(out)
    }

    /// Hand back bytes fetched for a URL this library asked for.
    ///
    /// The manifest and a chunk arrive by the same call; which one it is comes
    /// from the URL, not from the caller.
    ///
    /// # Errors
    ///
    /// [`LibraryError::Unexpected`] for a URL this library did not name,
    /// [`LibraryError::WrongLength`] for a chunk that is not the size the
    /// manifest says, and the manifest errors for a manifest that will not
    /// parse or does not describe a usable library.
    pub fn supply(&mut self, url: &str, bytes: Vec<u8>) -> Result<(), LibraryError> {
        if url == self.manifest_url {
            self.manifest = Some(read_manifest(&bytes)?);
            // A new manifest may describe different pieces, so what is held for
            // the old one is no longer addressable by ordinal.
            self.held.clear();
            return Ok(());
        }
        let Some(manifest) = self.manifest.as_ref() else {
            return Err(LibraryError::Unexpected(url.to_string()));
        };
        let found = manifest
            .chunks
            .iter()
            .position(|entry| resolve(&self.manifest_url, &entry.file) == url);
        let Some(ordinal) = found else {
            return Err(LibraryError::Unexpected(url.to_string()));
        };
        let expected = manifest.chunks[ordinal].deals as usize * manifest.record_bytes;
        if bytes.len() != expected {
            return Err(LibraryError::WrongLength {
                url: url.to_string(),
                expected,
                found: bytes.len(),
            });
        }
        self.held.insert(ordinal, bytes);
        Ok(())
    }

    /// The pieces a run touches, as `(chunk ordinal, deals into it, how many)`.
    ///
    /// One entry per contiguous piece of the run, in order. A run crossing a
    /// boundary yields two; a run wrapping past the last chunk yields the tail
    /// of the library and then the head, and can name the same chunk twice if
    /// the library is small enough for a run to lap it.
    fn span(
        &self,
        manifest: &Manifest,
        first_deal: u64,
        count: u64,
    ) -> Result<Vec<(usize, u64, u64)>, LibraryError> {
        if count > manifest.total_deals {
            return Err(LibraryError::TooMany {
                wanted: count,
                total_deals: manifest.total_deals,
            });
        }
        let mut spans = Vec::new();
        // Wrapping is the reason for the modulo: an index past the end names a
        // deal from the beginning rather than being an error, so a page can
        // pick a start from a seed without knowing how big the library is.
        let mut deal = first_deal % manifest.total_deals;
        let mut left = count;
        while left > 0 {
            let ordinal = manifest.chunk_of(deal);
            let Some(entry) = manifest.chunks.get(ordinal) else {
                return Err(missing_chunk(ordinal));
            };
            let offset = deal - entry.first_deal;
            let take = (entry.deals.saturating_sub(offset)).min(left);
            // `chunk_of` names a chunk that contains `deal`, so `offset` is
            // inside it and `take` is at least one. Checked rather than
            // assumed: a zero would leave `left` unchanged and spin this loop
            // forever, which is the one failure mode here that never returns to
            // say what went wrong.
            if take == 0 {
                return Err(LibraryError::Manifest(format!(
                    "chunk {ordinal} does not contain deal {deal}"
                )));
            }
            spans.push((ordinal, offset, take));
            left -= take;
            deal = (deal + take) % manifest.total_deals;
        }
        Ok(spans)
    }
}

impl Manifest {
    /// Which chunk holds `deal`. Infallible: [`read_manifest`] has already
    /// checked that the chunks tile the library with none missing.
    fn chunk_of(&self, deal: u64) -> usize {
        let guess = (deal / self.deals_per_chunk) as usize;
        // `deals_per_chunk` makes this a division rather than a search, and the
        // validation makes the guess right; the search is the fallback for a
        // manifest whose last chunk is short, which shifts nothing but is
        // cheap to allow for.
        if let Some(entry) = self.chunks.get(guess) {
            if deal >= entry.first_deal && deal < entry.first_deal + entry.deals {
                return guess;
            }
        }
        match self
            .chunks
            .binary_search_by(|entry| entry.first_deal.cmp(&deal))
        {
            Ok(exact) => exact,
            // `deal` is inside the library and the chunks tile it from zero, so
            // the insertion point is never zero and the chunk before it holds
            // the deal.
            Err(after) => after.saturating_sub(1),
        }
    }
}

/// A chunk ordinal that `read_manifest`'s checks say cannot exist.
///
/// Reported rather than indexed into, so a manifest that slipped through the
/// validation is an error and not a panic out of a library.
fn missing_chunk(ordinal: usize) -> LibraryError {
    LibraryError::Manifest(format!("no chunk {ordinal}"))
}

/// Parse a manifest and check it describes a library that can be read.
///
/// The checks are what makes [`Manifest::chunk_of`] and the slicing in
/// [`Library::zrd`] infallible, and each one is a way a plausible manifest
/// would otherwise produce plausible wrong deals rather than an error.
fn read_manifest(bytes: &[u8]) -> Result<Manifest, LibraryError> {
    let manifest: Manifest =
        serde_json::from_slice(bytes).map_err(|e| LibraryError::Manifest(e.to_string()))?;

    if manifest.schema != SCHEMA {
        return Err(LibraryError::Schema {
            found: manifest.schema,
            understood: SCHEMA,
        });
    }
    // The pairing decodes ten-byte `.zdd` records. A manifest describing
    // anything else describes a format this crate cannot read, and reading it
    // as ten bytes anyway would silently misalign every table.
    if manifest.record_bytes != TABLE_LEN {
        return Err(LibraryError::Manifest(format!(
            "record_bytes is {}, but a .zdd record is {TABLE_LEN} bytes",
            manifest.record_bytes
        )));
    }
    if manifest.deals_per_chunk == 0 {
        return Err(LibraryError::Manifest("deals_per_chunk is zero".into()));
    }
    if manifest.chunks.is_empty() {
        return Err(LibraryError::Manifest("no chunks".into()));
    }

    // The chunks must tile the library from deal zero with no gap and no
    // overlap. A gap would make some deal unreachable; an overlap or a
    // misordered entry would make `chunk_of` name the wrong piece, and the
    // deals that came back would be legal, plausible and wrong.
    let mut next = 0u64;
    let last = manifest.chunks.len() - 1;
    for (ordinal, entry) in manifest.chunks.iter().enumerate() {
        if entry.first_deal != next {
            return Err(LibraryError::Manifest(format!(
                "chunk {ordinal} starts at deal {} where {next} was expected",
                entry.first_deal
            )));
        }
        if entry.deals == 0 {
            return Err(LibraryError::Manifest(format!("chunk {ordinal} is empty")));
        }
        // Every chunk but the last is full, or the division in `chunk_of` does
        // not name the chunk holding a deal.
        let full = entry.deals == manifest.deals_per_chunk;
        if !(full || (ordinal == last && entry.deals < manifest.deals_per_chunk)) {
            return Err(LibraryError::Manifest(format!(
                "chunk {ordinal} holds {} deals, not the {} this manifest declares",
                entry.deals, manifest.deals_per_chunk
            )));
        }
        next += entry.deals;
    }
    if next != manifest.total_deals {
        return Err(LibraryError::Manifest(format!(
            "the chunks hold {next} deals but total_deals is {}",
            manifest.total_deals
        )));
    }
    Ok(manifest)
}

/// Resolve a manifest-relative chunk path against the manifest's own URL.
///
/// A `file` carrying a scheme is taken as it stands, one beginning with `/` is
/// resolved against the origin, and anything else against the directory the
/// manifest sits in — so `zdd/rpdd-000.zdd` beside `.../data/manifest.json`
/// becomes `.../data/zdd/rpdd-000.zdd`. Any query or fragment on the manifest
/// URL is dropped first, so it cannot be mistaken for a path.
fn resolve(manifest_url: &str, file: &str) -> String {
    if file.contains("://") {
        return file.to_string();
    }
    let base = manifest_url
        .split_once(['?', '#'])
        .map_or(manifest_url, |(before, _)| before);

    if let Some(path) = file.strip_prefix('/') {
        let origin = match base.find("://") {
            Some(scheme) => match base[scheme + 3..].find('/') {
                Some(slash) => &base[..scheme + 3 + slash],
                None => base,
            },
            None => "",
        };
        return format!("{origin}/{path}");
    }
    match base.rfind('/') {
        Some(slash) => format!("{}{file}", &base[..=slash]),
        None => file.to_string(),
    }
}
