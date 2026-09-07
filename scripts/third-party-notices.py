#!/usr/bin/env python3
"""Generate THIRD-PARTY-NOTICES from the dependency graph.

Hand-maintained notice files go stale the moment a dependency moves, and a
stale notice is worse than none — it asserts something untrue about what a
binary contains. So this reads what cargo actually resolved, finds each
package's own licence file, and copies the copyright line out of it rather
than guessing from the `authors` field, which is often empty and never
authoritative.

    scripts/third-party-notices.py            # write both notice files
    scripts/third-party-notices.py --check    # fail if either would change
"""

import json
import re
import subprocess
import sys
import textwrap
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The one thing this project distributes. Unlike bridge-solver, which this
# script came from, there is no second graph: no CLI, no WebAssembly wrapper,
# one crate.
TARGETS = [
    (ROOT, "rpdd-reader", ROOT / "THIRD-PARTY-NOTICES", "the rpdd-reader crate"),
]

# Crates of this project, under this project's own licence. Not third party.
OURS = {"rpdd-reader"}

LICENCE_FILE = re.compile(r"^(LICEN[CS]E|COPYING|NOTICE)", re.I)
COPYRIGHT = re.compile(r"^\s*(Copyright\b.*)$", re.M)


def metadata(directory):
    """The resolved dependency graph, without disturbing Cargo.lock.

    Reading the graph can rewrite the lock, and silently: a checkout with
    local `[patch]` overrides resolves its siblings to path dependencies and
    drops their `source` lines, which would break CI, where no such checkouts
    exist. `--locked` refuses rather than rewrites, but it also refuses in
    exactly that situation, so it cannot be used here. Putting the file back
    is what works for both.
    """
    lock = directory / "Cargo.lock"
    before = lock.read_bytes() if lock.exists() else None
    try:
        out = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--all-features"],
            cwd=directory, capture_output=True, text=True, check=True,
        )
    finally:
        if before is not None and lock.exists() and lock.read_bytes() != before:
            lock.write_bytes(before)
        elif before is None and lock.exists():
            lock.unlink()
    return json.loads(out.stdout)


def shipped(meta, root_name):
    """Package ids reachable from `root_name` through normal dependencies.

    Dev-dependencies are excluded: a test harness is not in the binary, and
    claiming otherwise would misstate what a copy contains. Build-dependencies
    and proc-macro crates are kept — their code does not end up in the binary
    either, but they are how it was produced, and being over-inclusive here
    errs towards crediting rather than away from it.
    """
    nodes = {node["id"]: node for node in meta["resolve"]["nodes"]}
    roots = [p["id"] for p in meta["packages"] if p["name"] == root_name]
    seen, stack = set(), list(roots)
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        seen.add(current)
        for dep in nodes.get(current, {}).get("deps", []):
            kinds = {k.get("kind") for k in dep.get("dep_kinds", [])}
            if kinds and kinds == {"dev"}:
                continue
            stack.append(dep["pkg"])
    return seen


def dev_only(meta, root_name):
    """Package names reachable from `root_name` *only* as dev-dependencies.

    They are not in a copy of this crate and this file does not credit them as
    though they were. They are listed by name at the end because a notice
    saying "nothing to credit" invites the question of what the test suite
    pulls in, and answering it is cheaper than being asked.
    """
    nodes = {node["id"]: node for node in meta["resolve"]["nodes"]}
    roots = [p["id"] for p in meta["packages"] if p["name"] == root_name]
    names = {p["id"]: p["name"] for p in meta["packages"]}
    seen, stack = set(), []
    for root in roots:
        for dep in nodes.get(root, {}).get("deps", []):
            kinds = {k.get("kind") for k in dep.get("dep_kinds", [])}
            if kinds == {"dev"}:
                stack.append(dep["pkg"])
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        seen.add(current)
        for dep in nodes.get(current, {}).get("deps", []):
            stack.append(dep["pkg"])
    return sorted({names[i] for i in seen if names.get(i) not in OURS})


def copyrights(manifest: Path):
    """Copyright lines from a package's own licence files, deduplicated."""
    found, seen = [], set()
    for path in sorted(manifest.parent.iterdir()):
        if not path.is_file() or not LICENCE_FILE.match(path.name):
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line in COPYRIGHT.findall(text):
            line = " ".join(line.split())
            # Apache-2.0's boilerplate carries a specimen line, not a claim.
            if "[yyyy] [name of copyright owner]" in line:
                continue
            if line not in seen:
                seen.add(line)
                found.append(line)
    return found


def blocks(text):
    """Each package's rendered block from an existing notice file, by name."""
    parts = text.split("Packages\n--------\n", 1)
    if len(parts) != 2:
        return {}
    found = {}
    for block in parts[1].split("\nLicence texts")[0].strip().split("\n\n"):
        block = block.strip()
        if block:
            found[block.split("\n")[0]] = block
    return found


def render(packages, what, carried=(), dev=()):
    lines = [
        "THIRD-PARTY NOTICES",
        "===================",
        "",
        f"For {what}.",
        "",
        "rpdd-reader is dedicated to the public domain under the Unlicense; see",
        "LICENSE, and the README for what the generator derives from. Where a",
        "project distributes other people's code as well as its own, their",
        "licences ask that their copyright notices travel with copies, and a",
        "file like this is how they travel.",
        "",
        "GENERATED by scripts/third-party-notices.py from the resolved",
        "dependency graph. Do not edit by hand; run the script.",
        "",
    ]

    if not packages and not carried:
        # An empty notice file asserts nothing and reads like an oversight.
        # Say plainly that there is nothing to credit, and why.
        lines += [
            "There is nothing to credit. rpdd has no dependencies at all: it",
            "is one file of arithmetic over the standard library, and a copy",
            "of it contains no third-party code.",
            "",
            "That is deliberate rather than incidental. The crate turns a deal",
            "index into thirteen bytes; anything it depended on would be",
            "someone else's opinion travelling with those bytes into every",
            "consumer, including a WebAssembly build where each dependency is",
            "download size.",
            "",
        ]
        if dev:
            lines += [
                "Its test suite is another matter, and pulls in "
                f"{len(dev)} packages:",
                "",
                *textwrap.wrap(", ".join(dev), width=68,
                               initial_indent="    ",
                               subsequent_indent="    "),
                "",
                "They are named for information only. Dev-dependencies are not",
                "in a copy of this crate — `cargo add rpdd` fetches none of",
                "them — so this file does not credit them as though they were.",
                "",
            ]
        lines += [
            "If this file ever lists packages, the script found them in the",
            "resolved graph and they are genuinely shipped.",
            "",
        ]
        return "\n".join(lines)

    lines += [
        "No version numbers. This project does not pin its dependencies, so a",
        "version here would be whichever one happened to resolve when the file",
        "was written, not the one in the copy you hold. What does not drift is",
        "the list of packages and what each is licensed under, which is what a",
        "notice is for.",
        "",
        f"{len(packages)} third-party packages, all under permissive licences.",
        "Dev-dependencies are excluded: a test harness is not in a binary.",
        "",
    ]

    packages = sorted(
        list(packages) + list(carried),
        key=lambda p: (p if isinstance(p, str) else p["name"]).split("\n")[0].lower(),
    )
    by_licence = {}
    for pkg in packages:
        if isinstance(pkg, str):
            licence = next(
                (line.split(":", 1)[1].strip() for line in pkg.split("\n")
                 if line.strip().startswith("Licence:")), "see source")
            by_licence.setdefault(licence, []).append(pkg)
            continue
        by_licence.setdefault(pkg["licence"], []).append(pkg)

    lines.append("Summary")
    lines.append("-------")
    for licence in sorted(by_licence):
        lines.append(f"  {len(by_licence[licence]):>3}  {licence}")
    lines.append("")
    lines.append("")
    lines.append("Packages")
    lines.append("--------")
    lines.append("")

    for pkg in packages:
        if isinstance(pkg, str):          # carried through from a past run
            lines.extend(pkg.split("\n"))
            lines.append("")
            continue
        lines.append(pkg["name"])
        lines.append(f"    Licence: {pkg['licence']}")
        if pkg["repository"]:
            lines.append(f"    Source:  {pkg['repository']}")
        for line in pkg["copyrights"]:
            lines.append(f"    {line}")
        if not pkg["copyrights"]:
            if pkg["authors"]:
                lines.append(f"    Authors: {', '.join(pkg['authors'])}")
            elif "Unlicense" in pkg["licence"]:
                lines.append("    Dedicated to the public domain; "
                             "no copyright asserted.")
            else:
                lines.append("    (its licence files assert no copyright line, "
                             "and it names no authors)")
        lines.append("")

    lines.append("")
    lines.append("Licence texts")
    lines.append("-------------")
    lines.append("")
    lines.append("Apache-2.0: https://www.apache.org/licenses/LICENSE-2.0")
    lines.append("MIT:        https://opensource.org/licenses/MIT")
    lines.append("Unlicense:  https://unlicense.org/")
    lines.append("Unicode-3.0: https://www.unicode.org/license.txt")
    lines.append("")
    lines.append("A package offering a choice of licences is used under whichever")
    lines.append("the recipient prefers; the notice above is reproduced either way.")
    lines.append("")
    return "\n".join(lines)


def notices(directory, root_name, what, existing=None):
    existing = existing or {}
    meta = metadata(directory)
    keep = shipped(meta, root_name)
    packages = []
    for pkg in meta["packages"]:
        if pkg["name"] in OURS or pkg["id"] not in keep:
            continue
        manifest = Path(pkg["manifest_path"])
        packages.append({
            "name": pkg["name"],
            "version": pkg["version"],
            "licence": pkg.get("license") or "see source",
            "repository": pkg.get("repository") or "",
            "copyrights": copyrights(manifest),
            "authors": [a for a in pkg.get("authors", []) if a],
        })
    # Two majors of the same package are one entry: they say the same thing
    # about licence and copyright, which is all this file asserts.
    seen, unique = set(), []
    for pkg in packages:
        key = (pkg["name"], pkg["licence"], tuple(pkg["copyrights"]))
        if key not in seen:
            seen.add(key)
            unique.append(pkg)
    packages = unique
    packages.sort(key=lambda p: p["name"].lower())
    # Anything a previous run credited that this resolution did not produce is
    # carried through rather than dropped. Nothing pins these dependencies, so
    # a machine with an older lock, a fresh CI checkout and a release build can
    # each resolve a different set — and a notice that shrank on one machine
    # would stop crediting a package another machine ships. The file only ever
    # grows; over-crediting is harmless, omitting is the failure.
    resolved = {pkg["name"] for pkg in packages}
    carried = [block for name, block in existing.items() if name not in resolved]
    dev = dev_only(meta, root_name)
    return render(packages, what, carried, dev), len(packages) + len(carried)


def credited(text):
    """Package names a rendered notice file credits."""
    body = text.split("Packages\n--------\n", 1)
    if len(body) != 2:
        return set()
    names = set()
    for block in body[1].split("\nLicence texts")[0].strip().split("\n\n"):
        first = block.strip().split("\n")[0]
        if first:
            names.add(first)
    return names


def uncredited(out, fresh):
    """Packages that resolved but the committed file does not name.

    Deliberately one-directional. Nothing pins this project's dependencies, so
    a resolution months later legitimately differs — a transitive package
    appears, another is dropped, a crate restates its licence. Failing on any
    difference would break unrelated pull requests for no benefit.

    What must never happen is shipping a package the notice does not credit.
    An extra name is harmless over-crediting; a missing one is the failure this
    exists to prevent.
    """
    if not out.exists():
        return credited(fresh)
    return credited(fresh) - credited(out.read_text())


def main():
    check = "--check" in sys.argv
    failed = False
    for directory, root_name, out, what in TARGETS:
        existing = blocks(out.read_text()) if out.exists() else {}
        text, count = notices(directory, root_name, what, existing)
        where = out.relative_to(ROOT)
        if check:
            missing = uncredited(out, text)
            if missing:
                print(f"{where} does not credit: {', '.join(sorted(missing))}\n"
                      "  run scripts/third-party-notices.py", file=sys.stderr)
                failed = True
            else:
                print(f"{where} credits everything that resolved "
                      f"({count} packages)")
        else:
            out.write_text(text)
            print(f"wrote {where} ({count} packages)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
