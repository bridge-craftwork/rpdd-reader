//! What pairing must get right without the 241MB library to hand.
//!
//! The byte-for-byte comparison in `pairs_the_real_library.rs` is the check
//! that matters, and it needs a file most machines do not have. These are the
//! cases that can be checked anywhere: the shape of the output, the deals it
//! chose, and the one table value that means something other than a table.

use bridge_encodings::zrd::{read_record, write_zdd_table, Record};
use rpdd_reader::{deal_at, pair, PairError, LIBRARY_DEALS, RECORD_LEN, TABLE_LEN};

const DEAL_LEN: usize = RECORD_LEN - TABLE_LEN;

/// A run of distinct, well-formed tables — every cell in range, and no two
/// records alike, so a misplaced one is visible.
fn tables(count: usize) -> Vec<u8> {
    (0..count)
        .flat_map(|i| (0..TABLE_LEN).map(move |b| (((i + b) % 14) as u8) << 4 | ((i % 13) as u8)))
        .collect()
}

/// Each record must hold the deal at its own index, and the table it was given.
#[test]
fn every_record_is_its_own_deal_and_its_own_table() {
    const FIRST: u64 = 4_096_000 + 7;
    const COUNT: usize = 300;

    let given = tables(COUNT);
    let zrd = pair(&given, FIRST).expect("a run pairs");
    assert_eq!(zrd.len(), COUNT * RECORD_LEN);

    for (i, record) in zrd.chunks_exact(RECORD_LEN).enumerate() {
        assert_eq!(
            &record[..DEAL_LEN],
            &deal_at(FIRST + i as u64)[..],
            "record {i} does not hold deal {}",
            FIRST + i as u64
        );
        assert_eq!(
            &record[DEAL_LEN..],
            &given[i * TABLE_LEN..(i + 1) * TABLE_LEN],
            "record {i} does not hold the table it was given"
        );
    }
}

/// An all-zero table means "not solved", and must come back out as no table
/// rather than as twenty zeros — which is a legal-looking table saying every
/// contract makes nothing.
#[test]
fn an_unsolved_table_reads_back_as_no_table() {
    let zrd = pair(&[0u8; TABLE_LEN], 1234).expect("an unsolved record pairs");

    match read_record(&zrd).expect("the record is well formed") {
        Record::Deal { table, .. } => assert!(
            table.is_none(),
            "an all-zero table came back as a table, not as unsolved"
        ),
        Record::Separator => panic!("a paired deal read back as a separator"),
    }
    assert_eq!(
        &zrd[DEAL_LEN..],
        &[0u8; TABLE_LEN],
        "the zeros were not kept"
    );
}

/// And a solved table survives the round trip unchanged, so the previous test
/// is about zero meaning "unsolved" rather than about zeros passing through.
#[test]
fn a_solved_table_survives_the_pairing() {
    let mut solved = [0u8; TABLE_LEN];
    solved.copy_from_slice(&tables(1));
    let zrd = pair(&solved, 55).expect("a solved record pairs");

    let Record::Deal { table, .. } = read_record(&zrd).expect("well formed") else {
        panic!("a paired deal read back as a separator");
    };
    let table = table.expect("a non-zero table is a table");
    assert_eq!(
        write_zdd_table(Some(&table)).expect("re-encodes"),
        solved,
        "the table changed on the way through"
    );
}

/// A `.zdd` has no framing, so a truncated download is otherwise undetectable:
/// the leftover bytes would silently become part of nothing.
#[test]
fn a_ragged_slice_is_refused() {
    let err = pair(&[0u8; TABLE_LEN + 3], 0).expect_err("ragged bytes are refused");
    assert_eq!(
        err,
        PairError::Ragged {
            bytes: TABLE_LEN + 3,
            record_bytes: TABLE_LEN
        }
    );
}

/// `Deals` never ends — it keeps unranking past the library and yields legal
/// deals nobody has a table for — so the bound is this function's to enforce.
#[test]
fn a_run_past_the_end_of_the_library_is_refused() {
    let first = LIBRARY_DEALS - 2;
    let err = pair(&tables(3), first).expect_err("running off the end is refused");
    assert_eq!(
        err,
        PairError::PastTheEnd {
            first_deal_index: first,
            records: 3,
            library_deals: LIBRARY_DEALS,
        }
    );
    // And the last deal in the library still pairs.
    pair(&tables(2), first).expect("the final two records pair");
}

/// A cell above thirteen is not a trick count, and must be named rather than
/// truncated into a plausible one.
#[test]
fn an_impossible_trick_count_is_refused_by_name() {
    let mut bad = [0u8; TABLE_LEN];
    bad[0] = 0x0E; // fourteen tricks
    let err = pair(&bad, 77).expect_err("fourteen tricks is refused");
    match err {
        PairError::Record { deal_index, .. } => assert_eq!(deal_index, 77),
        other => panic!("expected a record error, got {other}"),
    }
}

/// Pairing is a pure function of the slice and its first index: the same
/// records however they are cut up.
#[test]
fn a_slice_pairs_the_same_however_it_is_cut() {
    let whole = pair(&tables(40), 900).expect("forty records pair");
    let given = tables(40);
    let head = pair(&given[..17 * TABLE_LEN], 900).expect("head pairs");
    let tail = pair(&given[17 * TABLE_LEN..], 917).expect("tail pairs");
    assert_eq!(whole, [head, tail].concat());
}
