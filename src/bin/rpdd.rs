//! The crate's three layers at a command line.
//!
//! ```text
//! rpdd deals <first-deal-index> <how-many>
//! rpdd zrd   <first-deal-index> <how-many> --zdd <file> [--zdd-first-deal <n>]
//! ```
//!
//! Both write binary to standard output — packed deals from `deals`, `.zrd`
//! records from `zrd` — so the output goes into a file or a pipe, never a
//! terminal. `deals` is the older `dump` example under a new name, and the
//! bytes it writes are unchanged: `scripts/check-against-zrd.py` still compares
//! them against the deal halves of a real `rpdd.zrd`.
//!
//! There is no `fetch`. The crate performs no I/O and this binary keeps that
//! shape one level out: it reads a `.zdd` a caller already has and pairs it.
//! Getting the tables in the first place is [rpdd-library]'s business.
//!
//! [rpdd-library]: https://github.com/bridge-craftwork/rpdd-library

use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::process::ExitCode;

use rpdd_reader::{pair, LIBRARY_DEALS, TABLE_LEN};

const USAGE: &str = "\
usage:
  rpdd deals <first-deal-index> <how-many>
        packed deals, 13 bytes each, to stdout

  rpdd zrd <first-deal-index> <how-many> --zdd <file> [--zdd-first-deal <n>]
        .zrd records, 23 bytes each (deal + its table), to stdout

  --zdd <file>            a .zdd of double-dummy tables: the whole 105 MiB
                          rpdd.zdd, or one chunk of it
  --zdd-first-deal <n>    the library index of the file's first record
                          (default 0, which is what a whole rpdd.zdd is; a
                          chunk starts somewhere else and must say where)
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();

    let result = match argv.split_first() {
        Some((&"deals", rest)) => deals(rest),
        Some((&"zrd", rest)) => zrd(rest),
        Some((&("-h" | "--help"), _)) | None => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Some((other, _)) => Err(format!("no such command: {other}\n\n{USAGE}")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// `rpdd deals <start> <count>` — the generator on its own, no tables involved.
fn deals(args: &[&str]) -> Result<(), String> {
    let [start, count] = args else {
        return Err(format!(
            "usage: rpdd deals <first-deal-index> <how-many>\n\n{USAGE}"
        ));
    };
    let (start, count) = range(start, count)?;

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for packed in rpdd_reader::Deals::from(start).take(count as usize) {
        out.write_all(&packed).map_err(writing)?;
    }
    out.flush().map_err(writing)
}

/// `rpdd zrd <start> <count> --zdd <file>` — tables from disk, joined to the
/// deals they belong to.
///
/// The records are read out of the middle of the file rather than the file
/// being loaded: a whole `rpdd.zdd` is 105 MiB and a caller asking for a
/// hundred deals should pay for a hundred deals.
fn zrd(args: &[&str]) -> Result<(), String> {
    let mut positional = Vec::new();
    let mut path = None;
    let mut file_first_deal = 0u64;

    let mut rest = args.iter();
    while let Some(&arg) = rest.next() {
        match arg {
            "--zdd" => path = Some(*rest.next().ok_or("--zdd wants a path")?),
            "--zdd-first-deal" => {
                let value = *rest.next().ok_or("--zdd-first-deal wants a number")?;
                file_first_deal = value
                    .parse()
                    .map_err(|_| format!("--zdd-first-deal: not a number: {value}"))?;
            }
            other => positional.push(other),
        }
    }

    let [start, count] = positional.as_slice() else {
        return Err(format!(
            "usage: rpdd zrd <first-deal-index> <how-many> --zdd <file>\n\n{USAGE}"
        ));
    };
    let (start, count) = range(start, count)?;
    let path = path.ok_or_else(|| format!("rpdd zrd needs --zdd <file>\n\n{USAGE}"))?;

    // Which records of *this file* hold deals `start..start + count`. A whole
    // rpdd.zdd begins at deal 0 and the two indices coincide; a chunk does not,
    // and the difference is the whole reason `--zdd-first-deal` exists. Getting
    // it wrong pairs every deal with a neighbour's table and produces entirely
    // plausible wrong answers, so it is checked rather than assumed.
    let skip = start.checked_sub(file_first_deal).ok_or_else(|| {
        format!("{path} starts at deal {file_first_deal}, which is past deal {start}")
    })?;

    let mut file = File::open(path).map_err(|e| format!("opening {path}: {e}"))?;
    file.seek(SeekFrom::Start(skip * TABLE_LEN as u64))
        .map_err(|e| format!("seeking in {path}: {e}"))?;

    let mut tables = vec![0u8; count as usize * TABLE_LEN];
    file.read_exact(&mut tables)
        .map_err(|e| format!("{path} holds fewer than {count} records past deal {start}: {e}"))?;

    let records = pair(&tables, start).map_err(|e| e.to_string())?;

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    out.write_all(&records).map_err(writing)?;
    out.flush().map_err(writing)
}

/// Parse and bounds-check `<first-deal-index> <how-many>`, which both commands
/// take and neither may run past the end of the library with.
///
/// [`rpdd_reader::Deals`] is an endless iterator — it keeps unranking past
/// [`LIBRARY_DEALS`] and yields legal deals that are in nobody's library — so
/// nothing downstream would object.
fn range(start: &str, count: &str) -> Result<(u64, u64), String> {
    let start: u64 = start
        .parse()
        .map_err(|_| format!("not a deal index: {start}"))?;
    let count: u64 = count.parse().map_err(|_| format!("not a count: {count}"))?;
    if start.saturating_add(count) > LIBRARY_DEALS {
        return Err(format!(
            "the library holds {LIBRARY_DEALS} deals; {start}+{count} runs past its end"
        ));
    }
    Ok((start, count))
}

fn writing(e: std::io::Error) -> String {
    format!("writing to stdout: {e}")
}
