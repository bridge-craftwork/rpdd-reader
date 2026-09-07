#!/usr/bin/env python3
"""Check the generator against a real `rpdd.zrd`, group by group.

`cargo test` checks the crate against `fixtures/deal-digests.json`, which is
enough for CI and needs no 241MB file. This is the check those digests were
made from, and the one to run if you doubt them: it compares the crate's deals
against the library itself, byte for byte.

It needs a built `rpdd.zrd` — Pavlicek's `rpdd.bat` produces one from `rpdd.zip`
— which is his distribution and is not in this repository.

    scripts/check-against-zrd.py [path/to/rpdd.zrd]   # default: ./rpdd.zrd

A `.zrd` record is 23 bytes: 13 of packed deal, then 10 of double-dummy table.
Only the first 13 are the generator's business.
"""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

RECORD_LEN = 23
DEAL_LEN = 13
GROUP = 16_384

# Scattered through the file, and including the very last group. A generator
# wrong in a way that only shows up late would still pass a check of the first
# few thousand deals.
STARTS = (0, 16_384, 4_096_000, 8_192_000, 10_469_376)


def deals_from_crate(start, count):
    """The crate's deals, as raw bytes, via the `dump` example."""
    built = subprocess.run(
        ["cargo", "run", "--release", "--quiet", "--example", "dump",
         "--", str(start), str(count)],
        cwd=ROOT, capture_output=True, check=True,
    )
    return built.stdout


def deals_from_library(handle, start, count):
    """The deal halves of `count` records, starting at record `start`."""
    handle.seek(start * RECORD_LEN)
    raw = handle.read(count * RECORD_LEN)
    if len(raw) < count * RECORD_LEN:
        raise SystemExit(f"the file ends before deal {start + count}")
    return b"".join(
        raw[i * RECORD_LEN:i * RECORD_LEN + DEAL_LEN] for i in range(count)
    )


def main():
    zrd = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "rpdd.zrd"
    if not zrd.exists():
        raise SystemExit(
            f"no such file: {zrd}\n"
            "  build rpdd.zrd from Pavlicek's rpdd.zip, or pass its path"
        )

    failed = False
    with zrd.open("rb") as handle:
        for start in STARTS:
            mine = deals_from_crate(start, GROUP)
            real = deals_from_library(handle, start, GROUP)
            ok = mine == real
            failed |= not ok
            print(f"deals {start:>10}-{start + GROUP - 1:<10} "
                  f"{'MATCH' if ok else 'MISMATCH'}  ({GROUP} deals)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
