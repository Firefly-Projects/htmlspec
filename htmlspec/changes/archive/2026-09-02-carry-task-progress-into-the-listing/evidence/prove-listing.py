#!/usr/bin/env python3
"""Task 5.1: the listing's fraction against `status`, one change at a time.

Builds a project with changes in deliberately varied task states, then compares what the
enumeration carries against what the completion operation reports for that same change.
Checked by comparing numbers rather than by reading two outputs side by side.
"""
import json
import shutil
import subprocess
import sys
from pathlib import Path

MANIFEST = "Cargo.toml"
ROOT = Path("/tmp/provedemo")

STATES = {
    "alpha": ["done", "done"],
    "beta": ["in-progress", "todo", "done"],
    "gamma": ["dropped", "dropped"],
    "delta-with-a-long-name": ["todo", "todo", "todo", "done"],
    "epsilon-no-tasks": None,
}


def htmlspec(*args, cwd=ROOT):
    return subprocess.run(
        ["cargo", "run", "--quiet", "--manifest-path", MANIFEST, "--bin", "htmlspec", "--", *args],
        capture_output=True, text=True, cwd=cwd,
    )


def rewrite_tasks(path: Path, states: list[str]) -> None:
    source = path.read_text()
    head = source[: source.index('<li class="task"')]
    tail = source[source.rindex("</li>") + len("</li>") :]
    body = ""
    for index, state in enumerate(states):
        para = (
            '<p data-field="reason">Superseded by another task.</p>'
            if state == "dropped"
            else '<p data-field="note">The work this task states.</p>'
        )
        body += (
            f'<li class="task" id="task-{index}" data-change="added" data-state="{state}">\n'
            f'  <h3>Task {index} <span class="hs-id">task-{index}</span></h3>\n  {para}\n</li>\n'
        )
    path.write_text(head + body + tail)


def main() -> int:
    shutil.rmtree(ROOT, ignore_errors=True)
    ROOT.mkdir(parents=True)
    subprocess.run(["git", "init", "-q", "."], cwd=ROOT, check=True)
    htmlspec("init")
    for name, states in STATES.items():
        htmlspec("new", "change", name)
        tasks = ROOT / "htmlspec" / "changes" / name / "tasks.html"
        if states is None:
            tasks.unlink()
        else:
            rewrite_tasks(tasks, states)

    print("=== the human report ===")
    print(htmlspec("list").stdout)

    listing = json.loads(htmlspec("list", "--json").stdout)["changes"]
    print("=== each change's completion, listing against status ===")
    disagreements = []
    for entry in listing:
        name = entry["name"]
        status = json.loads(htmlspec("status", "--change", name, "--json").stdout)["tasks"]
        s_total, s_open = status["total"], status["open"]
        s_resolved = s_total - s_open
        if "total" not in entry:
            # A change with no tasks document carries no completion. `status` reports zeros
            # because it has no way to say "absent"; that asymmetry is the point of the change.
            agree = s_total == 0
            print(f"  {name:24} listing: completion absent          status: total=0  -> "
                  f"{'AGREE' if agree else 'DISAGREE'}")
        else:
            l_total, counts = entry["total"], entry["tasks"]
            l_resolved = counts["done"] + counts["dropped"]
            agree = l_total == s_total and l_resolved == s_resolved
            print(f"  {name:24} listing: {counts['done']}/{l_total} "
                  f"(+{counts['dropped']} dropped, {entry['state']})".ljust(62)
                  + f"status: total={s_total} open={s_open} resolved={s_resolved}  -> "
                  + ("AGREE" if agree else "DISAGREE"))
        if not agree:
            disagreements.append(name)

    print()
    if disagreements:
        print(f"MISMATCH: {', '.join(disagreements)}")
        return 1
    print(f"all {len(listing)} changes agree")
    return 0


if __name__ == "__main__":
    sys.exit(main())
