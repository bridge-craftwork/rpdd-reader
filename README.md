# rpdd-reader

Reads Richard Pavlicek's library of 10,485,760 solved bridge deals — from his
single 241 MB file, or from chunks small enough for a browser to fetch one.

The deals are not stored here or anywhere else. They are a pure function of
their index, and this crate is that function. Only the double-dummy tables are
data, and those live in [rpdd-library].

## Attribution

**The library is Richard Pavlicek's.** He created 10,485,760 random deals and
solved each one twenty ways — almost two years of computer time — then
published the results for anyone to use.

| | |
|---|---|
| His site | [rpbridge.net](https://www.rpbridge.net/) |
| Where he serves it | [Bridge Utilities](https://www.rpbridge.net/rput.htm) |
| The download | [`rpdd.zip`](https://www.rpbridge.net/z/rpdd.zip) — 47 MiB, holding the tables and `xxdd.exe`, his Windows program that recreates the deals |
| His documentation | [`rpdd.txt`](https://www.rpbridge.net/d/rpdd.txt) |

> In the early 2000s I created a database of 10,485,760 random deals. That was
> easy. The daunting task was to solve each deal 20 times to determine the
> double-dummy makes for each hand in each strain, which required almost two
> years of computer time.

He published it "as a courtesy to other programmers and data addicts".
**Nothing here replaces his site.** If you want the library itself, get it from
him.

**Licensing.** This crate is our own code, reproducing an algorithm from its
published behaviour — algorithms are not what copyright covers. It carries none
of his data and does not redistribute `xxdd.exe`, which is his program rather
than his data. Released into the public domain under the [Unlicense](LICENSE).

## What it provides

Three layers, separable on purpose. Take the first without the others.

| | |
|---|---|
| **generator** | an index in, a packed 13-byte deal out. No dependencies, no data, no I/O |
| **pairing** | a run of `.zdd` tables plus the index of the first deal, out as `.zrd` records |
| **chunks** | which piece of a published library holds deal *N*, and the loop that gets it. Behind the `chunks` feature |

## Use it as a library

```toml
[dependencies]
rpdd-reader = { git = "https://github.com/bridge-craftwork/rpdd-reader" }

# With the fetch protocol as well:
rpdd-reader = { git = "https://github.com/bridge-craftwork/rpdd-reader", features = ["chunks"] }
```

```rust
use rpdd_reader::{Deals, pair};

// Anywhere in the library, without generating what comes before it.
for packed in Deals::from(4_096_000).take(1000) {
    // 13 bytes: two bits a card, holding the seat.
}

// Tables joined to the deals they belong to, as .zrd records.
let zrd = pair(&tables, 4_096_000)?;
```

About 640ns a deal, and the same in WebAssembly. It re-seeds every 16,384
deals, so starting anywhere costs at most 16,383 deals of catch-up — about
10ms — rather than replaying from the beginning.

## Use it as a command

Prebuilt for Linux, macOS and Windows on every [release]; or
`cargo install --git https://github.com/bridge-craftwork/rpdd-reader`.

```bash
# Packed deals, 13 bytes each.
rpdd deals 4096000 16384 > deals.bin

# Tables from a .zdd on disk, joined to their deals: .zrd records, 23 bytes each.
rpdd zrd 4096000 100 --zdd rpdd.zdd > out.zrd

# A chunk does not start at deal 0, and must say where it does start.
rpdd zrd 4096000 100 --zdd chunk-062.zdd --zdd-first-deal 4063232 > out.zrd
```

Both write binary to standard output, so both want a file or a pipe.

## This crate never fetches anything

It cannot, and that is deliberate. Fetching is asynchronous and this is
synchronous Rust, so there is no "give me chunk 42" callback to be had. Instead
[`Library`] hands back the URLs it needs and the caller supplies the bytes — by
`fetch`, by reading a file, or from a cache. One loop serves a browser, a test
and a command-line tool alike. See [docs/design.md](docs/design.md#the-fetch-protocol).

## Testing

```bash
cargo test --all-features                          # digests, pairing, chunks
cargo test --release --all-features -- --ignored   # and against the real file
```

The generator is checked against SHA-256 digests of the real library's deals,
not against itself: a mistyped multiplier still produces legal deals, just not
his. The ignored tests need Pavlicek's `rpdd.zrd`, which is not committed
anywhere, and skip with a message when it is absent.

## Where the detail lives

| | |
|---|---|
| [docs/design.md](docs/design.md) | why the layers split where they do, why the output is `.zrd` bytes, the fetch protocol, and what the tests can and cannot catch |
| [`docs/xxdd-disassembly.asm`](docs/xxdd-disassembly.asm) | the annotated listing every constant in `src/deals.rs` came from |
| [`docs/reference-implementation.py`](docs/reference-implementation.py) | a Python transcription, kept beside it |

Comments in `src/deals.rs` cite addresses into that listing (`@0x4012F9` and
the like). They are evidence, not decoration: **do not "clean up" a constant
without checking it there.** Nothing about a wrong one fails loudly.

## Related

- [rpdd-library] — the double-dummy tables, published as fetchable chunks with
  a manifest. The data half of this pair
- [bridge-encodings] — reads and writes the `.zrd` and `.zdd` record formats
- [Dealer3] — a worked consumer: it filters deals from a library, so `tricks()`,
  `dds()` and `par()` become lookups rather than searches. Live at
  [bridge-craftwork.com/dealer3](https://bridge-craftwork.com/dealer3/)

[rpdd-library]: https://github.com/bridge-craftwork/rpdd-library
[bridge-encodings]: https://github.com/bridge-craftwork/bridge-encodings
[Dealer3]: https://github.com/bridge-craftwork/Dealer3
[release]: https://github.com/bridge-craftwork/rpdd-reader/releases
[`Library`]: https://github.com/bridge-craftwork/rpdd-reader/blob/main/src/chunks.rs
