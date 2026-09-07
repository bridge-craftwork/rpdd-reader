//! A paired chunk must be the real library's records, byte for byte.
//!
//! This is the only check that catches an off-by-one in `first_deal_index`.
//! Every other test in this crate stays green if the pairing is shifted by one:
//! each record still holds a legal deal and a well-formed table, the lengths
//! are right, the wrapping is right, and the answers are entirely plausible —
//! they are simply every deal joined to its neighbour's double-dummy result.
//! Only the real file says otherwise.
//!
//! `rpdd.zrd` is 241,172,480 bytes and is not committed anywhere; it is built
//! from Pavlicek's `rpdd.zip` by his own `rpdd.bat`. So this test is
//! `#[ignore]`d and, when run, skips with a message if it cannot find one
//! rather than failing on a machine that has no copy.
//!
//! ```text
//! cargo test --release -- --ignored
//! RPDD_ZRD=/path/to/rpdd.zrd cargo test --release -- --ignored
//! ```

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use rpdd_reader::{pair, LIBRARY_DEALS, RECORD_LEN, TABLE_LEN};

const DEAL_LEN: usize = RECORD_LEN - TABLE_LEN;

/// Where a copy of the library might be, in the order worth looking.
///
/// The environment variable first, so a machine that keeps it anywhere can say
/// so; then the repository root, which `.gitignore` already reserves for a
/// dropped copy; then the sibling checkout of rpdd-library, which is where the
/// file is built.
fn library() -> Option<PathBuf> {
    if let Some(from_env) = std::env::var_os("RPDD_ZRD") {
        let path = PathBuf::from(from_env);
        return path.is_file().then_some(path);
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [root.join("rpdd.zrd"), root.join("../rpdd-library/rpdd.zrd")]
        .into_iter()
        .find(|path| path.is_file())
}

/// The library's own records for `count` deals from `first`.
fn records_at(file: &mut File, first: u64, count: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; count * RECORD_LEN];
    file.seek(SeekFrom::Start(first * RECORD_LEN as u64))
        .expect("seeking within a 241MB file");
    file.read_exact(&mut bytes).expect("reading records");
    bytes
}

/// The table halves of those records, which is exactly what a `.zdd` holds.
fn tables_of(records: &[u8]) -> Vec<u8> {
    records
        .chunks_exact(RECORD_LEN)
        .flat_map(|record| record[DEAL_LEN..].iter().copied())
        .collect()
}

/// A whole chunk of the size rpdd-library publishes, from the middle of the
/// file: 65,536 records paired from tables alone and compared with the file.
#[test]
#[ignore = "needs a built rpdd.zrd; run with --ignored"]
fn a_whole_chunk_is_the_librarys_own_records() {
    let Some(path) = library() else {
        eprintln!("skipping: no rpdd.zrd found (set RPDD_ZRD to one)");
        return;
    };
    let mut file = File::open(&path).expect("opening the library");

    const CHUNK: u64 = 65_536;
    let first = 42 * CHUNK;
    let expected = records_at(&mut file, first, CHUNK as usize);
    let paired = pair(&tables_of(&expected), first).expect("a chunk pairs");

    assert_eq!(
        paired.len(),
        expected.len(),
        "a paired chunk is the wrong length"
    );
    // Compared as whole vectors would print 1.5MB of hex on failure, so find
    // the first differing record and say which deal it is: an off-by-one shows
    // up at the very first one, and saying so is the whole point of the test.
    for (i, (got, want)) in paired
        .chunks_exact(RECORD_LEN)
        .zip(expected.chunks_exact(RECORD_LEN))
        .enumerate()
    {
        assert_eq!(
            got,
            want,
            "record {i} of the chunk (deal {}) is not the library's",
            first + i as u64
        );
    }
}

/// Windows chosen to catch a starting index that is right only sometimes: the
/// very first deal, a seed boundary, an unaligned start inside a group, and the
/// last records in the library.
#[test]
#[ignore = "needs a built rpdd.zrd; run with --ignored"]
fn scattered_windows_are_the_librarys_own_records() {
    let Some(path) = library() else {
        eprintln!("skipping: no rpdd.zrd found (set RPDD_ZRD to one)");
        return;
    };
    let mut file = File::open(&path).expect("opening the library");

    let windows: [(u64, usize); 6] = [
        (0, 1),
        (0, 5_000),
        (16_384, 5_000),
        (16_383, 5_000),
        (4_096_000 + 7, 5_000),
        (LIBRARY_DEALS - 5_000, 5_000),
    ];
    for (first, count) in windows {
        let expected = records_at(&mut file, first, count);
        let paired = pair(&tables_of(&expected), first).expect("a window pairs");
        assert_eq!(
            paired, expected,
            "{count} records from deal {first} are not the library's"
        );
    }
}

/// The whole stack against the library as it is actually published: the real
/// manifest, the real 640 KiB chunks, and the ask/supply loop between them.
///
/// The tests above prove the pairing; the ones in `chunks_serve_a_range.rs`
/// prove the chunk arithmetic against a made-up three-chunk library. Only this
/// one proves the two agree about the library we ship — that chunk 42 really
/// does start at deal 42 x 65,536, and that a run stitched from two files is
/// the file's own records.
#[cfg(feature = "chunks")]
mod published {
    use super::{library, records_at};
    use rpdd_reader::{Library, LibraryError, LIBRARY_DEALS, RPDD_MANIFEST};
    use std::fs::File;
    use std::path::PathBuf;

    /// A checkout of rpdd-library, which holds the manifest and the chunks.
    fn checkout() -> Option<PathBuf> {
        let from_env = std::env::var_os("RPDD_LIBRARY").map(PathBuf::from);
        let path = from_env
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rpdd-library"));
        path.join("data/manifest.json").is_file().then_some(path)
    }

    /// The published URLs, served from the checkout instead of the network.
    ///
    /// The crate hands back the URLs it would fetch; this maps them back to
    /// files. That mapping is the whole of what a page's `fetch` does
    /// differently, which is why the loop below is the page's loop.
    fn serve(checkout: &std::path::Path, url: &str) -> Option<Vec<u8>> {
        let prefix = RPDD_MANIFEST.rsplit_once('/').map(|(dir, _)| dir)?;
        let relative = url.strip_prefix(prefix)?.strip_prefix('/')?;
        std::fs::read(checkout.join("data").join(relative)).ok()
    }

    fn run(checkout: &std::path::Path, first: u64, count: u64) -> Vec<u8> {
        let mut lib = Library::at(RPDD_MANIFEST);
        loop {
            match lib.zrd(first, count) {
                Ok(bytes) => return bytes,
                Err(LibraryError::Needs(urls)) => {
                    for url in urls {
                        let bytes =
                            serve(checkout, &url).unwrap_or_else(|| panic!("no local {url}"));
                        lib.supply(&url, bytes).expect("supplying what was asked");
                    }
                }
                Err(other) => panic!("{other}"),
            }
        }
    }

    #[test]
    #[ignore = "needs a built rpdd.zrd and a checkout of rpdd-library; run with --ignored"]
    fn the_published_chunks_serve_the_librarys_own_records() {
        let (Some(checkout), Some(zrd_path)) = (checkout(), library()) else {
            eprintln!("skipping: need a checkout of rpdd-library and an rpdd.zrd");
            return;
        };
        let mut zrd = File::open(&zrd_path).expect("opening the library");

        // Inside one chunk; across the boundary between two; and a run that
        // starts two deals from the end of the library and wraps to its start,
        // which no single chunk can answer.
        for (first, count) in [(42 * 65_536 + 11, 50u64), (65_536 - 3, 6), (0, 3)] {
            let served = run(&checkout, first, count);
            let expected = records_at(&mut zrd, first, count as usize);
            assert_eq!(
                served, expected,
                "{count} deals from {first} are not the library's records"
            );
        }

        let wrapped = run(&checkout, LIBRARY_DEALS - 2, 4);
        let expected = [
            records_at(&mut zrd, LIBRARY_DEALS - 2, 2),
            records_at(&mut zrd, 0, 2),
        ]
        .concat();
        assert_eq!(wrapped, expected, "the wrap is not the library's records");
    }
}
