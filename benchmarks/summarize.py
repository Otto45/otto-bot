"""Summarizes one benchmark run (benchmarks/run.sh) from what otto kept: the session's usage records, conversations
and plans, the run's wall time, and the checks run on the app afterwards.

    python3 benchmarks/summarize.py <run directory> <stack> <task>
"""

import json
import sys
from collections import Counter
from pathlib import Path


def read_lines(path: Path) -> list[dict[str, object]]:
    if not path.exists():
        return []
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def tool_names(conversation_path: Path) -> Counter[str]:
    names: Counter[str] = Counter()
    for item in read_lines(conversation_path):
        if item.get("type") != "model_output" or item.get("call_kind") is None:
            continue
        output: dict[str, object] = json.loads(str(item.get("item") or "{}"))
        names[str(output.get("name") or output.get("type") or item.get("call_kind"))] += 1
    return names


def main() -> None:
    run_directory: Path = Path(sys.argv[1])
    stack: str = sys.argv[2]
    task: str = sys.argv[3]
    sessions: list[Path] = sorted((run_directory / "home" / "sessions").glob("*/"))
    if not sessions:
        print(f"{stack} {task}: no session")
        return
    session: Path = sessions[-1]

    usage_records: list[dict[str, object]] = read_lines(session / "usage_records.jsonl")
    lanes: dict[str, Counter[str]] = {}
    for record in usage_records:
        lane: Counter[str] = lanes.setdefault(str(record["lane_name"]), Counter())
        lane["requests"] += 1
        for field in ("input_tokens", "cached_input_tokens", "cache_write_tokens", "output_tokens", "reasoning_tokens"):
            value: object = record.get(field) or 0
            lane[field] += int(value) if isinstance(value, int) else 0

    wall_seconds: str = (run_directory / "wall_seconds").read_text().strip()
    checks: list[str] = [
        line for line in (run_directory / "checks.txt").read_text().splitlines() if line.startswith(("passed", "FAILED"))
    ]
    briefs: int = len(read_lines(session / "briefs.jsonl"))
    transcript: str = (run_directory / "transcript.txt").read_text()

    print(f"== {stack} / {task}: {wall_seconds}s wall, {briefs} brief(s)")
    total: Counter[str] = Counter()
    for lane_name, lane in sorted(lanes.items()):
        total.update(lane)
        tools: Counter[str] = tool_names(session / "conversation_items" / f"{lane_name}.jsonl")
        hit: float = lane["cached_input_tokens"] / lane["input_tokens"] if lane["input_tokens"] else 0.0
        print(
            f"  {lane_name:13} {lane['requests']:3} requests  in {lane['input_tokens']:>8}  cached {hit:5.1%}"
            f"  out {lane['output_tokens']:>6} (reasoning {lane['reasoning_tokens']})"
            f"  tools {dict(sorted(tools.items()))}"
        )
    hit_total: float = total["cached_input_tokens"] / total["input_tokens"] if total["input_tokens"] else 0.0
    print(
        f"  {'total':13} {total['requests']:3} requests  in {total['input_tokens']:>8}  cached {hit_total:5.1%}"
        f"  written {total['cache_write_tokens']}  out {total['output_tokens']} (reasoning {total['reasoning_tokens']})"
    )
    # What the implementor and the reviewer reported, and any error, as the CLI printed them.
    for line in transcript.splitlines():
        stripped: str = line.strip()
        if stripped.startswith(("implementor:", "reviewer:", "[high]", "[medium]", "[low]", "needs:", "error:", "(the implementor")):
            print(f"  {stripped[:200]}")
    print("  checks: " + ("all passed" if all(line.startswith("passed") for line in checks) else "; ".join(checks)))


main()
