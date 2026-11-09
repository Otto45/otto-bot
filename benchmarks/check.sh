#!/usr/bin/env bash
# Runs a stack's checks on the app a benchmark run left (<run directory>/bench-app), independently of otto, and writes
# what passed and what failed (with the end of a failure's output) to <run directory>/checks.txt.
#
#   benchmarks/check.sh <stack> <run directory>
set -euo pipefail

stack="$1"
run_directory="$(cd "$2" && pwd)"
app="$run_directory/bench-app"
case "$stack" in
    rust) checks=("cargo fmt --check" "cargo clippy --all-targets -- -D warnings" "cargo run --quiet --bin export-openapi -- --check") ;;
    python) checks=("uv run ruff format --check" "uv run ruff check" "uv run pyright" "uv run export-openapi --check") ;;
    typescript) checks=("pnpm exec prettier --check ." "pnpm exec eslint ." "pnpm exec tsc --noEmit" "pnpm run --silent export-openapi --check") ;;
    *) echo "no checks for $stack" >&2; exit 2 ;;
esac
# A Rust app built into its own target folder (see run.sh).
[ "$stack" = "rust" ] && [ -d "$run_directory/target" ] && export CARGO_TARGET_DIR="$run_directory/target"

output_file="$(mktemp)"
: > "$run_directory/checks.txt"
for check in "${checks[@]}"; do
    if (cd "$app" && env -u OPENAI_API_KEY bash -c "$check") > "$output_file" 2>&1; then
        echo "passed  $check" >> "$run_directory/checks.txt"
    else
        echo "FAILED  $check" >> "$run_directory/checks.txt"
        tail -20 "$output_file" | sed 's/^/        /' >> "$run_directory/checks.txt"
    fi
done
rm -f "$output_file"
