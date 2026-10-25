#!/usr/bin/env bash
# Measure a release binary: raw size, gzip size, platform and toolchain.
# Usage: measure-size.sh <binary-path>
set -euo pipefail
bin="$1"
[ -f "$bin" ] || { echo "missing binary: $bin" >&2; exit 2; }
bytes=$(stat -f%z "$bin")
gz=$(gzip -9 -c "$bin" | wc -c | tr -d ' ')
sha=$(shasum -a 256 "$bin" | cut -d' ' -f1)
printf '{"binary_bytes": %s, "gzip_bytes": %s, "sha256": "%s", "platform": "%s", "rustc": "%s"}\n' \
  "$bytes" "$gz" "$sha" "$(uname -s)-$(uname -m)" "$(rustc --version | cut -d' ' -f2)"
