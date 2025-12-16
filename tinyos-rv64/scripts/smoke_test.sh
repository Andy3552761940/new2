#!/usr/bin/env bash
set -euo pipefail

make bin

if command -v timeout >/dev/null 2>&1; then
  OUT=$(timeout 3s make run || true)
else
  OUT=$(make run || true)
fi

echo "$OUT" | grep -q "TinyOS-RV64 boot"
echo "$OUT" | grep -q "\[U-A\]"
echo "$OUT" | grep -q "page fault"

echo "[smoke] PASS"
