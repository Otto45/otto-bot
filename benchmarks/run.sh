#!/usr/bin/env bash
# Runs one benchmark task on a new app of one stack: `otto new` starts the app from the stack's template, the task is
# its first (and only) request, then the stack's checks run on what otto left, independently of otto (benchmarks/check.sh),
# and the run is summarized from the session's files (benchmarks/summarize.py).
#
#   benchmarks/run.sh <stack> <task> [<run directory>]
#
# <task> names a file in benchmarks/tasks/ (e.g. orders). Runs are independent (each has its own app, otto home and
# log), so several can run at once; their times are then measured under load. OPENAI_API_KEY must be set.
set -euo pipefail

stack="$1"
task="$2"
repository="$(cd "$(dirname "$0")/.." && pwd)"
run_directory="${3:-$repository/target/benchmarks/$(date +%Y%m%d-%H%M%S)-$stack-$task}"
task_file="$repository/benchmarks/tasks/$task.md"
otto="$repository/target/debug/otto"

[ -f "$task_file" ] || { echo "no task $task_file" >&2; exit 2; }
[ -n "${OPENAI_API_KEY:-}" ] || { echo "OPENAI_API_KEY is not set" >&2; exit 2; }
[ -x "$otto" ] || (cd "$repository" && cargo build --quiet)
mkdir -p "$run_directory"
run_directory="$(cd "$run_directory" && pwd)"
export OTTO_BOT_HOME="$run_directory/home"

# A Rust app builds into its own target folder outside the workspace, started as a copy-on-write clone of the
# template's, so its dependencies are already built (on APFS the clone takes no space).
if [ "$stack" = "rust" ] && [ -d "$repository/stack_packs/rust/template/target" ]; then
    export CARGO_TARGET_DIR="$run_directory/target"
    [ -d "$CARGO_TARGET_DIR" ] || cp -c -R "$repository/stack_packs/rust/template/target" "$CARGO_TARGET_DIR" 2>/dev/null \
        || cp -R "$repository/stack_packs/rust/template/target" "$CARGO_TARGET_DIR"
fi

# Every app of a stack has the same name, so their project maps (and cached prompt prefixes) are the same.
started_at=$(date +%s)
tr '\n' ' ' < "$task_file" | "$otto" new bench-app --stack "$stack" --in "$run_directory" > "$run_directory/transcript.txt" 2>&1 \
    || echo "otto exited with $?" >> "$run_directory/transcript.txt"
finished_at=$(date +%s)
echo $((finished_at - started_at)) > "$run_directory/wall_seconds"

"$repository/benchmarks/check.sh" "$stack" "$run_directory"
python3 "$repository/benchmarks/summarize.py" "$run_directory" "$stack" "$task" | tee "$run_directory/summary.txt"
