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

**This crate is that function.** An index goes in, thirteen packed bytes come
out. No dependencies, no data, no I/O.

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

## What this is not

**It carries no data.** The tables are 100 MiB that will never change again,
and they live in
[rpdd-library](https://github.com/bridge-craftwork/rpdd-library) as chunks a
browser can fetch one of. This crate used to live there too; depending on it
meant cloning 51MB packed to compile eighty lines of Rust, and the two have
nothing to do with each other's release rhythm.

So: **rpdd-library** is the tables, and how they were made. **rpdd** is the
generator, and the account of where its constants came from. A consumer that
wants deals paired with their double-dummy results needs both, and gets the
data by fetching chunks rather than by cloning them.

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
cargo test                          # against the committed digests
cargo test --release -- --ignored   # 1.35M deals against a naive u128 generator
```

The ignored test is the guard for anyone optimising the unranking again: it
holds the generator as it was before the arithmetic was narrowed and compares
deal for deal from eight scattered starts, including the library's last group.

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
