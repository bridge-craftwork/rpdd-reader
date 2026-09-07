#!/usr/bin/env python3
"""Record digests of the real library's deals, so CI can check the generator.

The generator's job is to reproduce the deal half of Pavlicek's records exactly.
Checking that needs the real deals — and shipping a few hundred thousand of them
to test against would publish the very thing this crate exists to make
unnecessary. They are also not ours to publish.

So what is committed is digests. Each covers one 16,384-deal seed group, taken
from the 13-byte deal half of every record in `rpdd.zrd`. A generator that
reproduces the group byte for byte matches; anything else does not. The groups
are scattered through the file, and include the first and the last.

`rpdd.zrd` is built from Pavlicek's `rpdd.zip` by his own `rpdd.bat`; it is not
in this repository.

    scripts/make-deal-digests.py [path/to/rpdd.zrd]   # writes fixtures/
"""

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ZRD = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "rpdd.zrd"
OUT = ROOT / "fixtures" / "deal-digests.json"

RECORD_LEN = 23
DEAL_LEN = 13
GROUP = 16384
GROUPS = [0, 1, 250, 500, 639]   # first, next, scattered, and the last


def main():
    if not ZRD.exists():
        raise SystemExit(
            f"{ZRD} not found. Build it from rpdd.zdd with xxdd.exe, or with "
            "this crate's own generator, before recording digests."
        )
    entries = []
    with ZRD.open("rb") as handle:
        for group in GROUPS:
            start = group * GROUP
            handle.seek(start * RECORD_LEN)
            raw = handle.read(GROUP * RECORD_LEN)
            deals = b"".join(
                raw[i * RECORD_LEN:i * RECORD_LEN + DEAL_LEN] for i in range(GROUP)
            )
            entries.append({
                "group": group,
                "first_deal": start,
                "deals": GROUP,
                "sha256": hashlib.sha256(deals).hexdigest(),
            })
    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text(json.dumps({
        "source": "rpdd.zrd, built from rpdd.zip, rpbridge.net, (c) 2007 Richard Pavlicek",
        "whole_file_sha1": "a5e35bdbed32d2a1faa8d450216e529701284994",
        "note": ("SHA-256 of the concatenated 13-byte deal halves of one "
                 "16,384-deal seed group. The deals themselves are not "
                 "published; the generator reproduces them."),
        "groups": entries,
    }, indent=2) + "\n")
    print(f"wrote {OUT.relative_to(ROOT)} ({len(entries)} groups)")


if __name__ == "__main__":
    main()
