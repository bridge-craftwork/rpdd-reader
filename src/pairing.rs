//! Tables joined to the deals they belong to.
//!
//! A `.zdd` file — or one fetched chunk of one — is double-dummy results and
//! nothing else: ten bytes a record, no deals, no index, no header. What makes
//! record `i` meaningful is knowing which deal it was solved for, and that is
//! not in the bytes. [`pair`] takes it as an argument.
//!
//! # It knows nothing about chunks
//!
//! `(tables, first_deal_index)` is the whole input. The slice may be the entire
//! 105 MiB `rpdd.zdd`, one 640 KiB chunk a browser fetched, or ten records cut
//! out of the middle; this module cannot tell and does not care. Where a chunk
//! begins, which chunk holds a deal and how a run wraps past the end of the
//! library are all [`crate::chunks`]' business, one layer up.
//!
//! # Why it emits `.zrd` bytes
//!
//! `.zrd` is the published format for a deal with its table, `bridge-encodings`
//! already reads and writes it, and dealer3 already runs from it. Emitting it
//! means the output of this function can be compared byte for byte against the
//! real `rpdd.zrd` — which is the one check that catches an off-by-one in
//! `first_deal_index`, an error that otherwise pairs every deal with its
//! neighbour's table and produces entirely plausible wrong answers.
//!
//! Nothing here reimplements either format. The ten-byte table is decoded by
//! [`bridge_encodings::zrd::read_zdd_table`] and the record written by
//! [`bridge_encodings::zrd::write_record`]; this module supplies the deal and
//! the arithmetic that says which deal.

use bridge_encodings::zrd::{read_record, read_zdd_table, write_record, Record};
use bridge_encodings::Deal;

use crate::deals::{Deals, DEAL_LEN, LIBRARY_DEALS};

/// Bytes in one `.zdd` record: a table with no deal attached.
pub const TABLE_LEN: usize = bridge_encodings::zrd::TABLE_LEN;

/// Bytes in one `.zrd` record: 13 of deal, 10 of table.
pub const RECORD_LEN: usize = bridge_encodings::zrd::RECORD_LEN;

/// Why a run of tables could not be paired with its deals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairError {
    /// The slice is not a whole number of ten-byte records.
    ///
    /// A `.zdd` has no framing, so a truncated download is otherwise
    /// undetectable: the leftover bytes would silently become part of nothing.
    Ragged {
        /// How many bytes were offered.
        bytes: usize,
        /// How long one record is.
        record_bytes: usize,
    },
    /// The run would need deals past the end of the library.
    ///
    /// [`Deals`] is an endless iterator — it keeps unranking beyond
    /// [`LIBRARY_DEALS`] and yields legal deals that are not in anyone's
    /// library — so the bound is checked here rather than noticed downstream.
    PastTheEnd {
        /// Where the run was asked to start.
        first_deal_index: u64,
        /// How many records it holds.
        records: u64,
        /// How many deals the library holds.
        library_deals: u64,
    },
    /// One record could not be decoded or written, with the deal it belonged to.
    Record {
        /// The library index of the deal being paired.
        deal_index: u64,
        /// What `bridge-encodings` said.
        message: String,
    },
}

impl core::fmt::Display for PairError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PairError::Ragged {
                bytes,
                record_bytes,
            } => write!(
                f,
                "{bytes} bytes of tables is not a whole number of {record_bytes}-byte records"
            ),
            PairError::PastTheEnd {
                first_deal_index,
                records,
                library_deals,
            } => write!(
                f,
                "{records} records from deal {first_deal_index} runs past the end of a \
                 {library_deals}-deal library"
            ),
            PairError::Record {
                deal_index,
                message,
            } => write!(f, "deal {deal_index}: {message}"),
        }
    }
}

impl std::error::Error for PairError {}

/// Pair a run of `.zdd` tables with the deals they were solved for, as `.zrd`
/// records.
///
/// `tables` is any whole number of ten-byte `.zdd` records, and
/// `first_deal_index` is the library index of the deal the *first* of them
/// belongs to. The output is `tables.len() / 10 * 23` bytes: the same records
/// in the same order, each now carrying its deal.
///
/// An all-zero table means "not solved" and is written back as one, which is
/// what a reader of the output will report as no table rather than as twenty
/// zeros. Pavlicek's published library contains none — all 10,485,760 records
/// are solved — but nothing here assumes that.
///
/// # Errors
///
/// [`PairError::Ragged`] if the slice is not a whole number of records,
/// [`PairError::PastTheEnd`] if the run would reach past deal
/// [`LIBRARY_DEALS`], and [`PairError::Record`] if a table's cells are out of
/// range or a deal will not encode.
///
/// ```
/// # use rpdd_reader::{pair, RECORD_LEN, TABLE_LEN};
/// // Two tables, both unsolved, belonging to deals 100 and 101.
/// let zrd = pair(&[0u8; 2 * TABLE_LEN], 100).expect("two records pair");
/// assert_eq!(zrd.len(), 2 * RECORD_LEN);
/// ```
pub fn pair(tables: &[u8], first_deal_index: u64) -> Result<Vec<u8>, PairError> {
    if !tables.len().is_multiple_of(TABLE_LEN) {
        return Err(PairError::Ragged {
            bytes: tables.len(),
            record_bytes: TABLE_LEN,
        });
    }
    let records = (tables.len() / TABLE_LEN) as u64;
    if first_deal_index.saturating_add(records) > LIBRARY_DEALS {
        return Err(PairError::PastTheEnd {
            first_deal_index,
            records,
            library_deals: LIBRARY_DEALS,
        });
    }

    let mut out = Vec::with_capacity(tables.len() / TABLE_LEN * RECORD_LEN);
    // `Deals` never ends, so `zip` is what bounds the loop: exactly one deal
    // per table, and the deal index is `first_deal_index + offset` throughout.
    // `as_chunks` rather than `chunks_exact`: the size is a constant, so the
    // slices become fixed-size arrays and `read_zdd_table`'s length check
    // becomes a compile-time fact. The remainder is empty — that was the
    // `Ragged` check above.
    let (whole, _) = tables.as_chunks::<TABLE_LEN>();
    let paired = Deals::from(first_deal_index).zip(whole);
    for (offset, (packed, table_bytes)) in paired.enumerate() {
        let deal_index = first_deal_index + offset as u64;
        let named = |message: String| PairError::Record {
            deal_index,
            message,
        };

        let table = read_zdd_table(table_bytes).map_err(|e| named(e.to_string()))?;
        let deal = deal_from_packed(&packed).map_err(named)?;
        let record = write_record(&deal, table.as_ref()).map_err(|e| named(e.to_string()))?;
        out.extend_from_slice(&record);
    }
    Ok(out)
}

/// Decode thirteen packed bytes into a deal, through the format's own reader.
///
/// The packed form the generator produces *is* the deal half of a `.zrd`
/// record, so the way to read it without a second implementation of the card
/// bits is to offer it as a record whose table half is absent. An all-zero
/// table half is exactly that — the format's "not solved" — so
/// [`read_record`] returns the deal and `table: None`, and no table is
/// invented in order to read a deal.
///
/// The separator sentinel cannot collide with this. A separator is a record
/// whose first four card bytes are zero, which reads as sixteen cards to West;
/// a hand holds thirteen, so no deal the generator produces can look like one.
/// It is still handled rather than assumed away, because "cannot happen" is
/// how a silent mispairing gets in.
fn deal_from_packed(packed: &[u8; DEAL_LEN]) -> Result<Deal, String> {
    let mut record = [0u8; RECORD_LEN];
    record[..DEAL_LEN].copy_from_slice(packed);
    match read_record(&record).map_err(|e| e.to_string())? {
        Record::Deal { deal, .. } => Ok(deal),
        Record::Separator => Err(
            "the generator produced a deal whose first four bytes are zero, which is the \
             .zrd separator sentinel and not a legal deal"
                .to_string(),
        ),
    }
}
