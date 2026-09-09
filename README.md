# rpdd-reader

Reads Richard Pavlicek's public library of 10,485,760 solved bridge deals —
both the single 241 MB file and the chunked form a browser can fetch.

**A reader, not a producer.** Nothing here creates deals: all 10,485,760 of them
already exist as a fixed, public corpus. What this does is recover them from a
compact encoding, which is decompression rather than generation. If anyone ever
wants to *construct* a new corpus in the same format, that is a different crate
and the name is still free.

Richard Pavlicek's 10,485,760 solved bridge deals are two things, and only one
of them is data. The double-dummy tables took him almost two years of computer
time. The deals they belong to are a pure function of their index, and his own
`rpdd.zip` ships a 2,560-byte program that recreates them.

Three layers, separable on purpose.

## 1. The generator: an index in, a deal out

No dependencies, no data, no I/O.

```rust
use rpdd_reader::Deals;

// Anywhere in the library, without generating what comes before it.
for packed in Deals::from(4_096_000).take(1000) {
    // 13 bytes: two bits a card, holding the seat. The deal half of a
    // .zrd record, ready for a decoder of that format.
}
```

About 640ns a deal, and the same in a WebAssembly build — the unranking is
written in arithmetic wasm32 has natively, which is most of why it is that
fast. It re-seeds every 16,384 deals, so an arbitrary starting position costs
at most 16,383 deals of catch-up, about 10ms, rather than replaying from the
beginning of the library.

## 2. Pairing: tables joined to the deals they belong to

A `.zdd` is double-dummy results and nothing else — ten bytes a record, no
deals, no index, no header. What makes record *i* meaningful is knowing which
deal it was solved for, and that is **not in the bytes**. So it is an argument.

```rust
use rpdd_reader::pair;

// Any run of .zdd tables, and the index of the deal the first belongs to.
let zrd = pair(&tables, 42 * 65_536)?;   // .zrd records: deal + its table
```

It works on any slice — the whole 105 MiB `rpdd.zdd`, one fetched chunk, or ten
records cut out of the middle — and knows nothing about chunks.

The output is `.zrd` bytes rather than a structure of our own, deliberately. It
is a published format with an existing reader, so **the output can be compared
byte for byte against the real `rpdd.zrd`** — the one check that catches an
off-by-one in the starting index, an error that otherwise pairs every deal with
its neighbour's table and produces entirely plausible wrong answers. A consumer
also feeds it to code that already reads that format rather than growing a
second path for it.

Nothing here reimplements either format: the table is decoded by
`bridge_encodings::zrd::read_zdd_table` and the record written by
`write_record`.

## 3. Chunks: "deals from index N", behind the `chunks` feature

A browser cannot download 105 MiB of tables to look at five deals, so
[rpdd-library] publishes them as pieces with a manifest. `Library` is the part
that knows which piece holds a deal, where in it, how a run that crosses a
boundary is stitched and how one past the end wraps to the start. A caller asks
for deals and never computes a piece number, an offset or a wrap.

**It never fetches, and cannot.** Fetching is asynchronous and this is
synchronous Rust, so a "give me chunk 42" callback into the crate is not
available. What is available is asking and being told:

```rust
use rpdd_reader::{Library, LibraryError, RPDD_MANIFEST};

let mut library = Library::at(RPDD_MANIFEST);
let zrd = loop {
    match library.zrd(deal_index, count) {
        Ok(bytes) => break bytes,
        // The manifest on the first round, the chunks it names on the second.
        Err(LibraryError::Needs(urls)) => for url in urls {
            library.supply(&url, fetch(&url))?;    // the caller's problem
        },
        Err(other) => return Err(other.into()),
    }
};
```

One protocol for both rounds: the caller fetches URLs and hands back bytes and
never learns which is which. The same loop serves a page using `fetch`, a test
using `include_bytes!` and a command-line tool reading files.

**The layout is data, not code.** `deals_per_chunk`, `record_bytes`,
`total_deals` and every chunk's `file` and `first_deal` come from the manifest;
none is a constant here. Our library happens to be 160 chunks of 65,536 deals,
but that is a hosting decision rather than a property of Pavlicek's library.
Point `Library::at` at a different manifest and it works. `RPDD_MANIFEST` is a
constant a caller may pass, not a default the logic falls back on. It names
rpdd-library's own Cloudflare Pages deployment, which is on a CDN and sets the
CORS and `Cross-Origin-Resource-Policy` headers a browser under COEP
`require-corp` needs; the same bytes are still served from the repository over
`raw.githubusercontent.com`, just slower and without those headers.

```toml
rpdd-reader = "0.1"                                        # 1 and 2
rpdd-reader = { version = "0.1", features = ["chunks"] }   # and 3
```

Feature-gated because the manifest is JSON and the layers below it are not:
someone with the whole file on disk needs the generator and the pairing and has
nothing to fetch.

[rpdd-library]: https://github.com/bridge-craftwork/rpdd-library

## The command line

The same three layers as a binary, `rpdd`. Prebuilt for Linux, macOS and
Windows on every [release]; or `cargo install rpdd-reader`.

```bash
# The generator on its own: packed deals, 13 bytes each.
rpdd deals 4096000 16384 > deals.bin

# Tables from a .zdd on disk, joined to the deals they belong to:
# .zrd records, 23 bytes each.
rpdd zrd 4096000 100 --zdd rpdd.zdd > out.zrd

# A chunk does not start at deal 0, and must say where it does start.
rpdd zrd 4096000 100 --zdd chunk-062.zdd --zdd-first-deal 4063232 > out.zrd
```

Both write binary to standard output, so both want a file or a pipe. `zrd`
seeks to the records it needs rather than reading the file, so asking a 105 MiB
`rpdd.zdd` for a hundred deals costs a hundred deals.

**There is no `fetch` subcommand.** The crate performs no I/O and the binary
keeps that shape one level out: it pairs tables somebody already has. Getting
them is [rpdd-library]'s business.

[release]: https://github.com/bridge-craftwork/rpdd-reader/releases

## What this is not

**It carries no data.** The tables are 100 MiB that will never change again,
and they live in
[rpdd-library](https://github.com/bridge-craftwork/rpdd-library) as chunks a
browser can fetch one of. This crate used to live there too; depending on it
meant cloning 51MB packed to compile eighty lines of Rust, and the two have
nothing to do with each other's release rhythm.

So: **rpdd-library** is the tables, and how they were made. **rpdd-reader** is
the code that reads them, and the account of where its constants came from. A
consumer that wants deals paired with their double-dummy results needs both,
and gets the data by fetching chunks rather than by cloning them.

**It performs no I/O.** Not a socket, not a file. It says what it needs and
takes bytes; where those come from is the caller's.

## Attribution

**The library is Richard Pavlicek's.** `rpdd.zip` at
[rpbridge.net](https://www.rpbridge.net/) — © 2007 Richard Pavlicek — holds the
deals and the complete twenty-cell double-dummy table for each one. His own
`rpdd.txt` records what that cost:

> In the early 2000s I created a database of 10,485,760 random deals. That was
> easy. The daunting task was to solve each deal 20 times to determine the
> double-dummy makes for each hand in each strain, which required almost two
> years of computer time.

He published it "as a courtesy to other programmers and data addicts". Nothing
here is a replacement for his site; if you want the library itself, get it from
[rpbridge.net](https://www.rpbridge.net/).

This crate is our own code, reproducing an algorithm from its published
behaviour. Algorithms are not what copyright covers, and no part of his
distribution is redistributed here — not the tables, and not `xxdd.exe`, which
is his program rather than his data. It is released into the public domain
under the Unlicense; see [LICENSE](LICENSE).

## Where the constants came from

`xxdd.exe` has no source. The generator was recovered by disassembling it, and
the two files that record that work travel with the crate rather than with the
data, because without them it is unmaintainable:

| | |
|---|---|
| `docs/xxdd-disassembly.asm` | the annotated listing every constant in `src/lib.rs` came from |
| `docs/reference-implementation.py` | a Python transcription, kept beside it |

Comments in `src/lib.rs` cite addresses into that listing (`@0x4012F9` and the
like). They are evidence, not decoration: **do not "clean up" a constant
without checking it there.** Nothing about a wrong one fails loudly.

## It is tested against the real library, not against itself

A mistyped multiplier still yields four thirteen-card hands, every one a legal
deal — just not his, which would pair every deal with another deal's table. So
`fixtures/deal-digests.json` holds SHA-256 digests of the real library's deals,
one per 16,384-deal seed group, scattered through the file and including the
first group and the last, and `cargo test` checks the generator against those.
Changing one hex digit of one multiplier fails that test and nothing else.

The deals themselves are not committed. Publishing a few hundred thousand of
them to test against would publish the very thing this crate exists to make
unnecessary.

```bash
cargo test --all-features                       # digests, pairing, chunks
cargo test --release --all-features -- --ignored  # and against the real file
```

The ignored tests need things not committed anywhere, and skip with a message
rather than failing when they are absent:

| | |
|---|---|
| `matches_the_u128_unranking.rs` | 1.35M deals against a naive `u128` transcription of the generator. The guard for anyone optimising the unranking again. |
| `pairs_the_real_library.rs` | a paired chunk against the real `rpdd.zrd`, byte for byte, plus the whole ask/supply loop against the published chunks. **The only check that catches an off-by-one in a chunk's starting deal** — every other test stays green, because each record still holds a legal deal and a well-formed table, just its neighbour's. |

The second needs an `rpdd.zrd`, found at `$RPDD_ZRD`, at the root of this
checkout, or in a sibling checkout of rpdd-library.

With a built `rpdd.zrd` present — Pavlicek's `rpdd.bat` produces one from his
zip — the digests can be checked or re-recorded against the file itself:

```bash
scripts/check-against-zrd.py path/to/rpdd.zrd   # the crate against the library
scripts/make-deal-digests.py path/to/rpdd.zrd   # re-record fixtures/
```

## The packed form

Thirteen bytes, two bits per card, holding the seat: `00` West, `01` North,
`10` East, `11` South. Card order is SA, SK, … S2, then the same descending run
for hearts, diamonds and clubs; bits are numbered least significant first. That
is the deal half of a `.zrd` record exactly, so it can be handed straight to a
decoder for that format.

## Related

- [rpdd-library](https://github.com/bridge-craftwork/rpdd-library) publishes the
  double-dummy tables as fetchable chunks, and a manifest describing them
- [bridge-encodings](https://github.com/bridge-craftwork/bridge-encodings) reads
  and writes the `.zrd` and `.zdd` record formats
- [dealer3](https://github.com/bridge-craftwork/Dealer3) filters deals from a
  library, so `tricks()`, `dds()` and `par()` become lookups rather than searches
