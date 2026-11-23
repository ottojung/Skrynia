#!/usr/bin/env python3
"""Source-only F1 reconciliation check (file reads/diff only; no build, no network). Run from repo root."""
import re
from pathlib import Path

root = Path(__file__).resolve().parents[2]
recs = root / "docs/intent-records"
spec = (root / "docs/v3-vm-spec.md").read_text()
readme = (root / "README.md").read_text()
doc = (root / "docs/v3/intent-supersession.md").read_text()
live = "\n".join(p.read_text() for p in sorted(recs.glob("*.md")))

def section(start, end):
    return doc.split(start, 1)[1].split(end, 1)[0]

retired = re.findall(r"\| `(\$id-\d+)` \|", section("## Retired CURRENT records", "## Retained records"))
trimmed = re.findall(r"\| `(\$id-\d+)` \|", section("## Retained records", "## Non-intent doc"))
trim_sec = section("## Retained records", "## Non-intent doc")
fails = []

def check(ok, msg):
    print(("PASS " if ok else "FAIL ") + msg)
    if not ok:
        fails.append(msg)

check(len(retired) == 17, f"17 retired IDs parsed (got {len(retired)})")
check(len(trimmed) == 5, f"5 trimmed IDs parsed (got {len(trimmed)})")
for i in retired:
    check(i not in live, f"{i} absent from live intent records")
    check(doc.count(i) == 1, f"{i} mapped exactly once")
for i in trimmed:
    check(i in live and i in trim_sec, f"{i} retained and mapped")

banned = ["Web Push", "WebPush", "root grant", "issuer credential", "issuerToken",
          "X-Skrynia-If-SHA256", "GET /store", "PUT /store", "SKRYNIA_URL",
          "STORE_DIR", "/grants/", "GET /version", "GET /health", "quota",
          "grant", "subscribe", "notification"]
for t in banned:
    check(t not in live, f"live records free of v2 token {t!r}")

for f, h in set(re.findall(r"`(docs/v3-vm-spec\.md|README\.md)` \"([^\"]+)\"", doc)):
    text = spec if f.startswith("docs/v3") else readme
    check(any(line.strip().lstrip("#").strip() == h for line in text.splitlines()),
          f"heading {h!r} exists in {f}")

for phrase in ["MUST NOT implement grants", "The only endpoint is `POST /`",
               "SKRYNIA_ROOT_WASM", "ERR_PERM"]:
    check(phrase in spec, f"kernel contract intact: {phrase!r}")

auth = (root / "docs/intent-location-authority.md").read_text().splitlines()
check(any(l.startswith("Status:") and "v2 compatibility target" in l for l in auth),
      "v2 target doc relabeled")
raise SystemExit(1 if fails else 0)
