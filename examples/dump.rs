//! Write packed deals to standard output, for checking against the real library.
//!
//! Thirteen bytes a deal, nothing between them — the same bytes
//! `scripts/check-against-zrd.py` pulls out of the deal half of an `rpdd.zrd`
//! record, so the two can be compared with no parsing on either side.
//!
//! ```text
//! cargo run --release --example dump -- 4096000 16384 > deals.bin
//! ```

use std::io::{BufWriter, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let parsed: Option<(u64, usize)> = match args.as_slice() {
        [_, start, count] => start.parse().ok().zip(count.parse().ok()),
        _ => None,
    };
    let Some((start, count)) = parsed else {
        eprintln!("usage: dump <first-deal-index> <how-many>");
        return ExitCode::FAILURE;
    };
    if start.saturating_add(count as u64) > rpdd_reader::LIBRARY_DEALS {
        eprintln!(
            "the library holds {} deals; {start}+{count} runs past its end",
            rpdd_reader::LIBRARY_DEALS
        );
        return ExitCode::FAILURE;
    }

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for packed in rpdd_reader::Deals::from(start).take(count) {
        if let Err(e) = out.write_all(&packed) {
            eprintln!("writing deals: {e}");
            return ExitCode::FAILURE;
        }
    }
    if let Err(e) = out.flush() {
        eprintln!("writing deals: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
