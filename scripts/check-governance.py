#!/usr/bin/env python3
"""Lint the cross-agent coordination ledger.

Enforces the machine-checkable parts of the multi-agent review protocol
(docs/governance/review-protocol.md):

  1. Every `merged` PR names a Merger agent.
  2. The Merger agent differs from the Author agent (the "no self-merge" rule,
     enforced at the agent-identity layer since all agents share one GitHub
     account).
  3. Every non-draft PR names a Reviewer agent that differs from the Author.

This is intentionally dependency-free (stdlib only) so it can run in any CI
image without a toolchain. Exit code 0 = clean, 1 = violations found.

Usage:
    python3 scripts/check-governance.py [path-to-ledger.md]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

DEFAULT_LEDGER = "docs/governance/coordination-ledger.md"

# Statuses in which the row represents a real, in-flight or completed PR.
NONDRAFT_STATUSES = {"in-review", "changes-requested", "approved", "merged"}
PLACEHOLDER = re.compile(r"^(|_.*_|n/?a|tbd|pending.*|-)$", re.IGNORECASE)


def is_placeholder(cell: str) -> bool:
    """A cell that names no concrete agent (empty, dash, italic note, pending)."""
    return bool(PLACEHOLDER.match(cell.strip().strip("*_")))


def parse_ledger_table(text: str) -> list[dict[str, str]]:
    """Extract the slice/PR table rows as dicts keyed by header name."""
    rows: list[dict[str, str]] = []
    header: list[str] | None = None
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            header = None
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if header is None:
            header = [c.lower() for c in cells]
            continue
        if all(set(c) <= {"-", ":", " "} for c in cells):  # separator row
            continue
        # Only parse the table that has the columns we care about.
        if "author agent" not in header:
            continue
        rows.append(dict(zip(header, cells)))
    return rows


def check(rows: list[dict[str, str]]) -> list[str]:
    problems: list[str] = []
    for row in rows:
        pr = row.get("pr", "?")
        status = row.get("status", "").strip().lower()
        author = row.get("author agent", "").strip()
        reviewer = row.get("reviewer agent", "").strip()
        merger = row.get("merger agent", "").strip()

        if status == "merged":
            if is_placeholder(merger):
                problems.append(f"{pr}: status 'merged' but no Merger agent named.")
            elif not is_placeholder(author) and merger.strip("*_") == author.strip("*_"):
                problems.append(
                    f"{pr}: Merger '{merger}' is the Author — a PR must be "
                    f"merged by a different agent."
                )

        if status in NONDRAFT_STATUSES:
            if is_placeholder(reviewer):
                problems.append(
                    f"{pr}: status '{status}' but no Reviewer agent named."
                )
            elif not is_placeholder(author) and reviewer.strip("*_") == author.strip("*_"):
                problems.append(
                    f"{pr}: Reviewer '{reviewer}' is the Author — a PR must be "
                    f"reviewed by a different agent."
                )
    return problems


def main(argv: list[str]) -> int:
    ledger_path = Path(argv[1]) if len(argv) > 1 else Path(DEFAULT_LEDGER)
    if not ledger_path.exists():
        print(f"error: ledger not found at {ledger_path}", file=sys.stderr)
        return 1
    rows = parse_ledger_table(ledger_path.read_text(encoding="utf-8"))
    if not rows:
        print(f"error: no PR table rows parsed from {ledger_path}", file=sys.stderr)
        return 1
    problems = check(rows)
    if problems:
        print("Governance check FAILED:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        return 1
    print(f"Governance check passed: {len(rows)} PR row(s) satisfy the "
          f"author != reviewer != merger rules.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
