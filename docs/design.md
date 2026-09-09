# Why the crate is shaped this way

The reasoning behind decisions that were not obvious, moved out of the README so
that it answers "what is this" first. Nothing here is required reading to use
the crate; all of it is required reading before changing one of these decisions.

## Three layers, and why they are separable

Each layer is useful without the ones above it, and the split is along the lines
of what a consumer actually has.

**The generator** reaches for nothing but the standard library. Someone who
wants deals and no tables — a dealer, a test fixture, a shuffle — should not
acquire a record format or a JSON parser to get them.

**The pairing** needs the `.zdd` and `.zrd` record formats, which
[bridge-encodings] owns. Reimplementing ten bytes of table here would create a
second definition of a published format, and the two would drift.

**The chunk layer** needs to read a manifest, which is JSON. It is feature-gated
because someone with the whole file on disk has nothing to fetch and should not
carry a JSON parser to unrank a deal.

## Why the pairing emits `.zrd` bytes

A `.zdd` file is double-dummy results and nothing else: ten bytes a record, no
deals, no index, no header. What makes record *i* meaningful is knowing which
deal it was solved for, and **that is not in the bytes** — so it is an argument
rather than something recovered.

The output is `.zrd` bytes rather than a structure of our own, deliberately. It
is a published format with an existing reader, so the output can be compared
byte for byte against the real `rpdd.zrd`. That comparison is the one check that
catches an off-by-one in the starting index — an error that otherwise pairs
every deal with its neighbour's table and produces entirely plausible wrong
answers. A consumer also feeds it to code that already reads that format,
instead of growing a second path.

`pair` works on any slice: the whole 100 MiB `rpdd.zdd`, one fetched chunk, or
ten records cut out of the middle. It cannot tell which, and does not care.
Where a chunk begins and how a run wraps past the end of the library are the
chunk layer's business, one level up.

## The fetch protocol

**This crate performs no I/O.** Not a socket, not a file. It says what it needs
and takes bytes; where those come from is the caller's problem.

That is not squeamishness, it is forced. Chunks arrive over the network, `fetch`
is asynchronous, and asynchronous JavaScript cannot be called from synchronous
Rust — so a "give me chunk 42" callback into the crate is not available at all.
What is available is asking and being told:

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

One protocol for both rounds: the caller fetches URLs and hands back bytes, and
never learns which round it is in. The same loop serves a page using `fetch`, a
test using `include_bytes!`, and a command-line tool reading files.

## The layout is data, not code

`deals_per_chunk`, `record_bytes`, `total_deals`, and every chunk's `file` and
`first_deal` come from the manifest. None is a constant here.

Our library happens to be 160 chunks of 65,536 deals, but that is a hosting
decision rather than a property of Pavlicek's library. Point `Library::at` at a
different manifest and it works. `RPDD_MANIFEST` is a constant a caller may
pass, not a default the logic falls back on.

## It carries no data, and that was a move

The tables are 100 MiB that will never change again. They live in
[rpdd-library], and this crate used to live there too. Depending on it meant
cloning 51 MB packed to compile eighty lines of Rust, and the two have nothing
to do with each other's release rhythm — the tables are finished, the code is
not.

So: **rpdd-library** is the tables and how they were made. **rpdd-reader** is
the code that reads them, and the account of where its constants came from. A
consumer wanting deals paired with their results needs both, and gets the data
by fetching chunks rather than by cloning them.

## Where the constants came from

`xxdd.exe` has no source. The generator was recovered by disassembling it, and
the two files recording that work travel with the crate rather than with the
data, because without them it is unmaintainable:

| | |
|---|---|
| [`xxdd-disassembly.asm`](xxdd-disassembly.asm) | the annotated listing every constant in `src/deals.rs` came from |
| [`reference-implementation.py`](reference-implementation.py) | a Python transcription, kept beside it |

Comments in `src/deals.rs` cite addresses into that listing (`@0x4012F9` and the
like). They are evidence, not decoration: **do not "clean up" a constant without
checking it there.**

## What the tests can and cannot catch

Nothing about a wrong constant fails loudly. A mistyped multiplier still yields
four thirteen-card hands, every one a legal deal — just not his, which would
pair every deal with another deal's table and answer every question plausibly
and wrongly.

So the self-consistency tests are not the point. `fixtures/deal-digests.json`
holds SHA-256 digests of the *real* library's deals, one per 16,384-deal seed
group, scattered through the file and including the first group and the last.
Changing one hex digit of one multiplier fails that test and nothing else.

The deals themselves are not committed. Publishing a few hundred thousand of
them to test against would publish the very thing this crate exists to make
unnecessary.

The ignored tests need things not committed anywhere, and skip with a message
rather than failing when they are absent:

| | |
|---|---|
| `matches_the_u128_unranking.rs` | 1.35M deals against a naive `u128` transcription. The guard for anyone optimising the unranking again |
| `pairs_the_real_library.rs` | a paired chunk against the real `rpdd.zrd`, byte for byte, plus the whole ask/supply loop against the published chunks. **The only check that catches an off-by-one in a chunk's starting deal** — every other test stays green, because each record still holds a legal deal and a well-formed table, just its neighbour's |

The second needs an `rpdd.zrd`, found at `$RPDD_ZRD`, at the root of this
checkout, or in a sibling checkout of rpdd-library. With one present —
Pavlicek's `rpdd.bat` produces it from his zip — the digests can be checked or
re-recorded against the file itself:

```bash
scripts/check-against-zrd.py path/to/rpdd.zrd   # the crate against the library
scripts/make-deal-digests.py path/to/rpdd.zrd   # re-record fixtures/
```

## The packed form

Thirteen bytes, two bits per card, holding the seat: `00` West, `01` North,
`10` East, `11` South. Card order is SA, SK, … S2, then the same descending run
for hearts, diamonds and clubs; bits are numbered least significant first.

That is the deal half of a `.zrd` record exactly, so it can be handed straight
to a decoder for that format. A full `.zrd` record is 23 bytes: those 13, then
10 of table.

[bridge-encodings]: https://github.com/bridge-craftwork/bridge-encodings
[rpdd-library]: https://github.com/bridge-craftwork/rpdd-library
