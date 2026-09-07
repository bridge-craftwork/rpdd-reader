//! The chunk layer: which piece holds a deal, and the ask/supply loop.
//!
//! Everything here runs against a small made-up library — three chunks of four
//! deals — described by a manifest of the same shape as rpdd-library's. That is
//! deliberate: the layer must derive the layout from the manifest rather than
//! from our 160-by-65,536 hosting, and a library that looks nothing like ours
//! is the way to show it does.
//!
//! The deals are real ones, though. `deal_at` is the generator, so the checks
//! below are against the actual deals at those indexes and not against
//! whatever the pairing happened to produce.

#![cfg(feature = "chunks")]

use rpdd_reader::{deal_at, Library, LibraryError, RECORD_LEN, TABLE_LEN};

const DEAL_LEN: usize = RECORD_LEN - TABLE_LEN;
const MANIFEST_URL: &str = "https://example.invalid/data/manifest.json";
const PER_CHUNK: u64 = 4;
const CHUNKS: u64 = 3;
const TOTAL: u64 = PER_CHUNK * CHUNKS;

/// A manifest of the same shape as rpdd-library's, for a twelve-deal library.
fn manifest_json(schema: u64) -> Vec<u8> {
    let chunks: Vec<String> = (0..CHUNKS)
        .map(|n| {
            format!(
                r#"{{"file":"zdd/tiny-{n:03}.zdd","first_deal":{},"deals":{PER_CHUNK},
                     "bytes":{},"sha256":"unchecked"}}"#,
                n * PER_CHUNK,
                PER_CHUNK * TABLE_LEN as u64
            )
        })
        .collect();
    format!(
        r#"{{"schema":{schema},"source":"made up","record_bytes":{TABLE_LEN},
             "deals_per_chunk":{PER_CHUNK},"total_deals":{TOTAL},
             "chunks":[{}]}}"#,
        chunks.join(",")
    )
    .into_bytes()
}

/// The ten bytes this made-up library holds for a given deal. Distinct per
/// deal, and every cell a legal trick count, so a misplaced table is visible.
fn table_for(deal: u64) -> [u8; TABLE_LEN] {
    let mut out = [0u8; TABLE_LEN];
    for (b, cell) in out.iter_mut().enumerate() {
        *cell = (((deal as usize + b) % 14) as u8) << 4 | ((deal as usize % 13) as u8);
    }
    out
}

/// The bytes of chunk `n`, as a server would send them.
fn chunk_bytes(n: u64) -> Vec<u8> {
    (n * PER_CHUNK..(n + 1) * PER_CHUNK)
        .flat_map(table_for)
        .collect()
}

fn url_of(n: u64) -> String {
    format!("https://example.invalid/data/zdd/tiny-{n:03}.zdd")
}

/// Run the protocol to completion, recording every URL it asked for.
fn fetch_until_ready(library: &mut Library, first: u64, count: u64) -> (Vec<u8>, Vec<String>) {
    let mut asked = Vec::new();
    loop {
        match library.zrd(first, count) {
            Ok(bytes) => return (bytes, asked),
            Err(LibraryError::Needs(urls)) => {
                for url in urls {
                    asked.push(url.clone());
                    let bytes = if url == MANIFEST_URL {
                        manifest_json(1)
                    } else {
                        let n = chunk_number(&url);
                        chunk_bytes(n)
                    };
                    library
                        .supply(&url, bytes)
                        .expect("supplying what was asked");
                }
            }
            Err(other) => panic!("unexpected error: {other}"),
        }
    }
}

fn chunk_number(url: &str) -> u64 {
    (0..CHUNKS)
        .find(|n| url_of(*n) == url)
        .unwrap_or_else(|| panic!("asked for a URL this library does not publish: {url}"))
}

/// Assert the run holds exactly these deals, each with its own table.
fn assert_run(zrd: &[u8], expected: &[u64]) {
    assert_eq!(
        zrd.len(),
        expected.len() * RECORD_LEN,
        "the run is the wrong length"
    );
    for (i, (record, deal)) in zrd.chunks_exact(RECORD_LEN).zip(expected).enumerate() {
        assert_eq!(
            &record[..DEAL_LEN],
            &deal_at(*deal)[..],
            "record {i} should be deal {deal}"
        );
        assert_eq!(
            &record[DEAL_LEN..],
            &table_for(*deal)[..],
            "record {i} should carry deal {deal}'s table"
        );
    }
}

/// Until the manifest is read there is nothing to say which chunks a range
/// touches, so the manifest is what the first round asks for.
#[test]
fn the_manifest_is_what_it_asks_for_first() {
    let library = Library::at(MANIFEST_URL);
    assert_eq!(
        library.needs(0, 1).expect("needs before a manifest"),
        vec![MANIFEST_URL.to_string()]
    );
    assert_eq!(library.total_deals(), None);
    match library.zrd(0, 1) {
        Err(LibraryError::Needs(urls)) => assert_eq!(urls, vec![MANIFEST_URL.to_string()]),
        other => panic!("expected to be asked for the manifest, got {other:?}"),
    }
}

/// A run inside one chunk asks for the manifest and then that one chunk.
#[test]
fn a_run_inside_one_chunk_fetches_one_chunk() {
    let mut library = Library::at(MANIFEST_URL);
    let (zrd, asked) = fetch_until_ready(&mut library, 5, 2);
    assert_eq!(asked, vec![MANIFEST_URL.to_string(), url_of(1)]);
    assert_run(&zrd, &[5, 6]);
    assert_eq!(library.total_deals(), Some(TOTAL));
}

/// A run crossing a boundary needs both chunks, and the join is not visible in
/// the result — the deals either side must be consecutive.
#[test]
fn a_run_crossing_a_boundary_fetches_both_chunks() {
    let mut library = Library::at(MANIFEST_URL);
    let (zrd, asked) = fetch_until_ready(&mut library, 3, 3);
    assert_eq!(
        asked,
        vec![MANIFEST_URL.to_string(), url_of(0), url_of(1)],
        "a crossing run must ask for both chunks"
    );
    assert_run(&zrd, &[3, 4, 5]);
}

/// Running off the end continues from the beginning, so any starting index
/// answers and a page can pick one from a seed without knowing the size.
#[test]
fn a_run_past_the_last_chunk_wraps_to_the_first() {
    let mut library = Library::at(MANIFEST_URL);
    let (zrd, asked) = fetch_until_ready(&mut library, 10, 4);
    assert_eq!(asked, vec![MANIFEST_URL.to_string(), url_of(2), url_of(0)]);
    assert_run(&zrd, &[10, 11, 0, 1]);
}

/// A starting index past the end of the library is itself a wrap, not an error.
#[test]
fn a_starting_index_past_the_end_wraps() {
    let mut library = Library::at(MANIFEST_URL);
    let (zrd, _) = fetch_until_ready(&mut library, TOTAL * 7 + 2, 2);
    assert_run(&zrd, &[2, 3]);
}

/// A run may lap the library exactly once, naming a chunk twice — which must
/// be asked for once and served twice.
#[test]
fn a_run_may_lap_the_library_once() {
    let mut library = Library::at(MANIFEST_URL);
    let (zrd, asked) = fetch_until_ready(&mut library, 10, TOTAL);
    assert_eq!(
        asked,
        vec![MANIFEST_URL.to_string(), url_of(2), url_of(0), url_of(1)],
        "chunk 2 is touched twice and must be asked for once"
    );
    assert_run(&zrd, &[10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

/// More than a lap would visit a deal twice, which is a mistake rather than a
/// wrap.
#[test]
fn more_deals_than_the_library_holds_is_refused() {
    let mut library = Library::at(MANIFEST_URL);
    library
        .supply(MANIFEST_URL, manifest_json(1))
        .expect("the manifest reads");
    match library.zrd(0, TOTAL + 1) {
        Err(LibraryError::TooMany {
            wanted,
            total_deals,
        }) => {
            assert_eq!((wanted, total_deals), (TOTAL + 1, TOTAL));
        }
        other => panic!("expected TooMany, got {other:?}"),
    }
}

/// A manifest announcing a schema this reader does not know must be refused,
/// not read hopefully: a changed meaning for `first_deal` would pair every deal
/// with the wrong table and look entirely plausible.
#[test]
fn a_schema_it_does_not_understand_is_refused() {
    let mut library = Library::at(MANIFEST_URL);
    match library.supply(MANIFEST_URL, manifest_json(2)) {
        Err(LibraryError::Schema { found, understood }) => {
            assert_eq!((found, understood), (2, 1));
        }
        other => panic!("expected a schema refusal, got {other:?}"),
    }
    // And nothing was learned from it.
    assert_eq!(library.total_deals(), None);
}

/// Chunk paths are relative to the manifest, so pointing the same code at a
/// manifest somewhere else fetches from there.
#[test]
fn chunk_urls_are_relative_to_the_manifest() {
    let elsewhere = "https://other.invalid/deep/nested/index.json";
    let mut library = Library::at(elsewhere);
    library
        .supply(elsewhere, manifest_json(1))
        .expect("the manifest reads");
    assert_eq!(
        library.needs(0, 1).expect("needs"),
        vec!["https://other.invalid/deep/nested/zdd/tiny-000.zdd".to_string()]
    );
}

/// A `.zdd` has no framing, so a truncated download is otherwise silent: it
/// just means fewer deals, with nothing to say so.
#[test]
fn a_chunk_of_the_wrong_length_is_refused() {
    let mut library = Library::at(MANIFEST_URL);
    library
        .supply(MANIFEST_URL, manifest_json(1))
        .expect("the manifest reads");
    let mut short = chunk_bytes(0);
    short.truncate(short.len() - 1);
    match library.supply(&url_of(0), short) {
        Err(LibraryError::WrongLength {
            expected, found, ..
        }) => assert_eq!(
            (expected, found),
            (
                PER_CHUNK as usize * TABLE_LEN,
                PER_CHUNK as usize * TABLE_LEN - 1
            )
        ),
        other => panic!("expected WrongLength, got {other:?}"),
    }
}

/// Bytes for a URL this library never named are refused rather than filed
/// somewhere plausible.
#[test]
fn bytes_for_an_unasked_for_url_are_refused() {
    let mut library = Library::at(MANIFEST_URL);
    library
        .supply(MANIFEST_URL, manifest_json(1))
        .expect("the manifest reads");
    match library.supply("https://example.invalid/data/zdd/tiny-999.zdd", vec![0; 40]) {
        Err(LibraryError::Unexpected(url)) => {
            assert!(url.ends_with("tiny-999.zdd"), "unexpected url: {url}")
        }
        other => panic!("expected Unexpected, got {other:?}"),
    }
}

/// A manifest whose chunks do not tile the library is refused. A gap makes some
/// deal unreachable and an overlap makes the arithmetic name the wrong piece;
/// either way what comes back is legal, plausible and wrong.
#[test]
fn a_manifest_whose_chunks_do_not_tile_is_refused() {
    let holed = String::from_utf8(manifest_json(1))
        .expect("utf-8")
        .replace(r#""first_deal":4"#, r#""first_deal":5"#);
    let mut library = Library::at(MANIFEST_URL);
    match library.supply(MANIFEST_URL, holed.into_bytes()) {
        Err(LibraryError::Manifest(why)) => {
            assert!(why.contains("starts at deal 5"), "unhelpful: {why}")
        }
        other => panic!("expected a manifest refusal, got {other:?}"),
    }
}

/// Held chunks are reused, so a second run in the same region fetches nothing —
/// and `forget_chunks` really releases them.
#[test]
fn chunks_are_held_and_can_be_released() {
    let mut library = Library::at(MANIFEST_URL);
    fetch_until_ready(&mut library, 0, 2);
    assert!(
        library.needs(1, 2).expect("needs").is_empty(),
        "a held chunk was asked for again"
    );
    library.forget_chunks();
    assert_eq!(library.needs(1, 2).expect("needs"), vec![url_of(0)]);
    // The manifest is kept: only the chunks are released.
    assert_eq!(library.total_deals(), Some(TOTAL));
}
