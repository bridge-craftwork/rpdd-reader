//! The generator reproduces Pavlicek's deals, or it is worthless.
//!
//! Nothing about a wrong constant fails loudly. A transposed step or a mistyped
//! multiplier still yields four thirteen-card hands, and every one of them is a
//! legal deal — just not his, which would silently pair every deal with another
//! deal's double-dummy table. So the check is against digests taken from the
//! real 241MB library, one per 16,384-deal seed group, scattered through the
//! file and including the first group and the last.
//!
//! The deals themselves are not committed. Publishing a few hundred thousand of
//! them to test against would be publishing the very thing this crate exists to
//! make unnecessary.

use sha2::{Digest, Sha256};

const DIGESTS: &str = include_str!("../fixtures/deal-digests.json");

#[test]
fn every_recorded_group_matches_the_library() {
    let fixture: serde_json::Value = serde_json::from_str(DIGESTS).expect("fixture parses");
    let groups = fixture["groups"].as_array().expect("groups is a list");
    assert!(
        !groups.is_empty(),
        "a fixture with no groups checks nothing"
    );

    for group in groups {
        let first = group["first_deal"].as_u64().expect("first_deal");
        let count = group["deals"].as_u64().expect("deals");
        let expected = group["sha256"].as_str().expect("sha256");

        let mut hasher = Sha256::new();
        for packed in rpdd::Deals::from(first).take(count as usize) {
            hasher.update(packed);
        }
        let got = hex::encode(hasher.finalize());

        assert_eq!(
            got, expected,
            "group starting at deal {first} does not reproduce the library"
        );
    }
}

/// Seeking must not change what a deal is — the whole point of re-seeding every
/// 16,384 deals is that a starting position is free of what came before it.
#[test]
fn a_deal_is_the_same_however_it_was_reached() {
    // A deal partway into a group, reached by running from the group boundary
    // and by seeking straight to it.
    const TARGET: u64 = 16_384 + 9_001;

    let walked = rpdd::Deals::from(16_384)
        .nth(9_001)
        .expect("Deals never ends");
    let sought = rpdd::deal_at(TARGET);

    assert_eq!(walked, sought, "seeking changed the deal");
}

/// And a group boundary really is a boundary: starting there needs no catch-up.
#[test]
fn a_group_boundary_needs_nothing_before_it() {
    let from_boundary = rpdd::deal_at(rpdd::SEED_GROUP * 250);
    let from_further_back = rpdd::Deals::from(rpdd::SEED_GROUP * 249)
        .nth(rpdd::SEED_GROUP as usize)
        .expect("Deals never ends");
    assert_eq!(from_boundary, from_further_back);
}
