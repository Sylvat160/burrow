#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"

dirs=(
    "crates/shell/src"
     "crates/shell/tests"
     "apps/terminal"
     "apps/web"
     "docs"
)

for dir in "${dirs[@]}"; do
    mkdir -p "$ROOT/$dir"
    echo "created $dir"
done
echo "done."
