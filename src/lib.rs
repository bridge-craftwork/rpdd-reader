//! Richard Pavlicek's library of 10,485,760 solved bridge deals, read from
//! wherever it happens to be.
//!
//! His `rpdd.zip` ships the double-dummy results (`rpdd.zdd`) together with
//! `xxdd.exe`, a 2,560-byte program that recreates the deals those results
//! belong to. Only the results are data: the deals are a pure function of their
//! index, and his own documentation says as much — making them "was easy", and
//! the two years of computer time went on solving them.
//!
//! The crate is three layers, and they are separable on purpose.
//!
//! # 1. [`deals`] — the generator
//!
//! An index goes in, a packed deal comes out. No dependencies, no data, no
//! I/O; ported from `xxdd.exe` and tested against digests of the real library.
//!
//! ```
//! use rpdd_reader::Deals;
//!
//! // Anywhere in the library, without generating what comes before it.
//! let mut deals = Deals::from(4_096_000);
//! let packed = deals.next().expect("the library is 10,485,760 deals long");
//! assert_eq!(packed.len(), 13);
//! ```
//!
//! # 2. [`pairing`] — tables joined to their deals
//!
//! [`pair`] takes a run of `.zdd` tables and the index of the deal the first of
//! them belongs to, and emits `.zrd` records: each deal with its own table.
//! It works on any slice — the whole 105 MiB file, one fetched chunk, or ten
//! records — and knows nothing about how the tables were obtained.
//!
//! `.zrd` bytes rather than a structure of our own, deliberately: it is a
//! published format with an existing reader, so the output can be compared
//! byte for byte against the real `rpdd.zrd`, and a consumer feeds it to code
//! that already reads that format instead of growing a second path.
//!
//! # 3. [`chunks`] — the library as fetchable pieces, behind the `chunks` feature
//!
//! [`Library`] turns "deals from index N" into the pieces that hold them, from
//! a manifest it is pointed at. It is feature-gated because it wants a JSON
//! parser and the two layers below it do not: someone with the whole file on
//! disk needs the generator and the pairing and has nothing to fetch.
//!
//! ```toml
//! rpdd-reader = "0.1"                                # generator + pairing
//! rpdd-reader = { version = "0.1", features = ["chunks"] }  # and the fetch protocol
//! ```
//!
//! # This crate never fetches anything
//!
//! It cannot. Chunks arrive over the network, `fetch` is async, and async
//! JavaScript cannot be called from synchronous Rust — so there is no
//! "give me chunk 42" callback available. What there is instead is asking and
//! being told: [`Library`] hands back the URLs it needs and the caller supplies
//! the bytes, by `fetch`, by reading a file, or from a cache. See [`chunks`]
//! for the loop.
//!
//! # The packed form
//!
//! Thirteen bytes, two bits per card, holding the seat: `00` West, `01` North,
//! `10` East, `11` South. Card order is SA, SK, ... S2, then the same
//! descending run for hearts, diamonds and clubs; bits are numbered least
//! significant first. That is the deal half of a `.zrd` record exactly.

#![forbid(unsafe_code)]

pub mod deals;
pub mod pairing;

#[cfg(feature = "chunks")]
pub mod chunks;

pub use deals::{deal_at, Deals, DEAL_LEN, LIBRARY_DEALS, SEED_GROUP};
pub use pairing::{pair, PairError, RECORD_LEN, TABLE_LEN};

#[cfg(feature = "chunks")]
pub use chunks::{Library, LibraryError, RPDD_MANIFEST};
