#!/usr/bin/env python3
"""Section 5: the stated sequence, against real projects and the real binary.

Five scenes, each an assertion rather than an output to read:

  5.1  a three-change sequence, middle change archived first -> named, asked, not refused
  5.2  the same sequence archived in order            -> nothing about ordering, at any step
  5.3  a renamed predecessor                          -> unmatched, not unfinished
  5.4  a two-change cycle                             -> a contradiction, no ordering chosen
  5.5  the motivating four-change sequence            -> the silent archival is reported

Run: uv run openspec/changes/state-a-changes-place-in-a-sequence/evidence/prove-sequence.py
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

HTMLSPEC = Path(__file__).resolve().parents[4] / "target" / "release" / "htmlspec"
ROOT = Path("/tmp/prove-sequence")

failures: list[str] = []


def check(label: str, condition: bool, detail: str = "") -> None:
    print(f"  {'ok  ' if condition else 'FAIL'}  {label}")
    if not condition:
        failures.append(f"{label}{': ' + detail if detail else ''}")
        if detail:
            print(f"        {detail}")


def run(project: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [str(HTMLSPEC), *args], capture_output=True, text=True, cwd=project
    )


def new_project(name: str) -> Path:
    project = ROOT / name
    shutil.rmtree(project, ignore_errors=True)
    project.mkdir(parents=True)
    subprocess.run(["git", "init", "-q", "."], cwd=project, check=True)
    run(project, "init", "--tools", "none")
    return project


def change(project: Path, name: str, follows: str | None = None) -> Path:
    """A change whose tasks are all done, ready to archive, optionally following another."""
    run(project, "new", "change", name)
    directory = project / "htmlspec" / "changes" / name
    # Every task done, so an open task is not a finding competing with the ordering.
    tasks = directory / "tasks.html"
    source = tasks.read_text()
    head = source[: source.index('<li class="task"')]
    tail = source[source.rindex("</li>") + len("</li>") :]
    tasks.write_text(
        head
        + '<li class="task" id="task-the-work" data-change="added" data-state="done">\n'
        '  <h3>The work <span class="hs-id">task-the-work</span></h3>\n'
        '  <p data-field="note">The work this change states.</p>\n</li>\n'
        + tail
    )
    if follows is not None:
        (directory / "FOLLOWS").write_text(f"{follows}\n")
    return directory


def specify(project: Path, directory: Path) -> None:
    """The specification archival gates on, stating a requirement already accepted."""
    run(
        project,
        "new",
        "document",
        "specification",
        str(directory / "specification.html"),
        "--subject",
        directory.name,
    )
    # The same requirement in the accepted specifications, so `needsSync` is not a finding
    # competing with the ordering one.
    accepted = project / "htmlspec" / "specs" / f"{directory.name}.html"
    accepted.write_text((directory / "specification.html").read_text())


def advance(project: Path, version: str) -> None:
    declaration = project / "spec.nix"
    text = declaration.read_text()
    start = text.index('version = "') + len('version = "')
    end = text.index('"', start)
    declaration.write_text(text[:start] + version + text[end:])


def archive(project: Path, name: str, *extra: str) -> subprocess.CompletedProcess:
    return run(project, "archive", name, *extra)


def scene_5_1() -> None:
    print("\n5.1  the middle change of a sequence, archived first")
    project = new_project("out-of-order")
    change(project, "the-contract")
    middle = change(project, "the-graded-work", follows="the-contract")
    change(project, "the-device-work", follows="the-graded-work")
    specify(project, middle)
    advance(project, "0.1.0")

    checked = archive(project, "the-graded-work", "--check", "--json")
    payload = json.loads(checked.stdout)
    check("--check states the condition as a value", payload["outOfOrder"] is True, checked.stdout)
    check(
        "--check names the predecessor in the payload",
        payload["unresolvedPredecessor"]["change"] == "the-contract",
        json.dumps(payload.get("unresolvedPredecessor")),
    )
    check(
        "--check reports it as active rather than unmatched",
        payload["unresolvedPredecessor"]["resolution"] == "active",
    )
    check(
        "--check names the predecessor to a reader",
        "follows the-contract" in checked.stderr,
        checked.stderr.strip(),
    )
    check(
        "--check says the change is ready with questions rather than refusing",
        "ready with questions" in checked.stdout or checked.returncode == 0,
        f"exit {checked.returncode}",
    )

    # No terminal to answer: it stops rather than archiving silently.
    stopped = archive(project, "the-graded-work")
    still_active = (project / "htmlspec" / "changes" / "the-graded-work").is_dir()
    check(
        "with nobody to ask it stops rather than archiving silently",
        stopped.returncode != 0 and still_active,
        f"exit {stopped.returncode}, still active: {still_active}",
    )
    check(
        "it still names the predecessor when it stops",
        "follows the-contract" in stopped.stderr,
        stopped.stderr.strip()[-300:],
    )

    # And --yes archives, because an abandoned sequence must remain archivable.
    forced = archive(project, "the-graded-work", "--yes")
    archived = list((project / "htmlspec" / "changes" / "archive").iterdir())
    check(
        "--yes archives over the finding",
        forced.returncode == 0 and any("the-graded-work" in p.name for p in archived),
        f"exit {forced.returncode}: {[p.name for p in archived]}",
    )
    check(
        "it named the predecessor before doing so",
        "follows the-contract" in forced.stderr,
        forced.stderr.strip()[:300],
    )


def scene_5_2() -> None:
    print("\n5.2  the same sequence, archived in order")
    project = new_project("in-order")
    first = change(project, "the-contract")
    second = change(project, "the-graded-work", follows="the-contract")
    third = change(project, "the-device-work", follows="the-graded-work")

    for index, (name, directory) in enumerate(
        [("the-contract", first), ("the-graded-work", second), ("the-device-work", third)]
    ):
        specify(project, directory)
        advance(project, f"0.{index + 1}.0")
        result = archive(project, name)
        check(
            f"{name} archives with no question asked",
            result.returncode == 0,
            f"exit {result.returncode}: {result.stderr.strip()[-300:]}",
        )
        check(
            f"{name}: nothing about ordering is reported",
            # Not a bare "follows": the version gate reports "version 0.2.0 follows 0.1.0",
            # which is a different fact in the same word.
            "this change follows" not in result.stderr
            and "contradiction" not in result.stderr,
            result.stderr.strip()[-300:],
        )

    listing = run(project, "list")
    check(
        "the project holds no active changes at the end",
        "no active changes" in listing.stdout,
        listing.stdout.strip(),
    )


def scene_5_3() -> None:
    print("\n5.3  a predecessor renamed after its successor recorded it")
    project = new_project("renamed")
    contract = change(project, "the-contract")
    successor = change(project, "the-graded-work", follows="the-contract")
    # Renamed by moving the directory, which is how `artifact-kinds` says a change is renamed.
    contract.rename(contract.with_name("the-contract-renamed"))
    specify(project, successor)
    advance(project, "0.1.0")

    checked = archive(project, "the-graded-work", "--check", "--json")
    payload = json.loads(checked.stdout)
    check(
        "the resolution is unmatched, not active",
        payload["unresolvedPredecessor"]["resolution"] == "unmatched",
        json.dumps(payload.get("unresolvedPredecessor")),
    )
    check(
        "the report calls it a name that matches nothing",
        "no active or archived change carries that name" in checked.stderr,
        checked.stderr.strip(),
    )
    check(
        "the report does not call it unfinished",
        "which is not archived" not in checked.stderr,
        checked.stderr.strip(),
    )
    check(
        "the report says to correct the declaration",
        "correct the declaration" in checked.stderr,
        checked.stderr.strip(),
    )

    listed = json.loads(run(project, "list", "--json").stdout)
    entry = next(c for c in listed["changes"] if c["name"] == "the-graded-work")
    check(
        "the listing separates unmatched from active too",
        entry["follows"]["resolution"] == "unmatched" and entry["follows"]["resolved"] is False,
        json.dumps(entry.get("follows")),
    )


def scene_5_4() -> None:
    print("\n5.4  two changes that each follow the other")
    project = new_project("cycle")
    first = change(project, "first-change", follows="second-change")
    change(project, "second-change", follows="first-change")

    listed = json.loads(run(project, "list", "--json").stdout)
    check(
        "the cycle is reported once, not once per member",
        listed["followsCycles"] == [["first-change", "second-change"]],
        json.dumps(listed.get("followsCycles")),
    )
    human = run(project, "list")
    check(
        "the human listing names it a contradiction",
        "contradiction" in human.stderr,
        human.stderr.strip(),
    )
    check(
        "no ordering is chosen: the table is still in name order",
        [line.split()[0] for line in human.stdout.splitlines() if line.strip()]
        == ["first-change", "second-change"],
        human.stdout.strip(),
    )

    specify(project, first)
    advance(project, "0.1.0")
    checked = archive(project, "first-change", "--check", "--json")
    payload = json.loads(checked.stdout)
    check(
        "archival carries the cycle as a value",
        payload["followsCycle"] == ["first-change", "second-change"],
        json.dumps(payload.get("followsCycle")),
    )
    check(
        "archival calls it a contradiction rather than an ordering violation",
        "contradiction" in checked.stderr,
        checked.stderr.strip(),
    )


def scene_5_5() -> None:
    print("\n5.5  the motivating sequence: the contract, and the three changes it grades")
    project = new_project("missile-launcher")
    change(project, "contract-the-guidance-solver-comparison")
    coarsening = change(
        project,
        "measure-the-cost-of-decimation",
        follows="contract-the-guidance-solver-comparison",
    )
    change(
        project,
        "compare-the-guidance-solver-families",
        follows="contract-the-guidance-solver-comparison",
    )
    change(
        project,
        "settle-sensors-and-precision",
        follows="compare-the-guidance-solver-families",
    )

    # The listing an agent selects from now carries the reason not to select.
    listed = json.loads(run(project, "list", "--json").stdout)
    unresolved = sorted(
        c["name"] for c in listed["changes"] if c.get("follows") and not c["follows"]["resolved"]
    )
    check(
        "all three successors are reported as waiting on their predecessor",
        unresolved
        == [
            "compare-the-guidance-solver-families",
            "measure-the-cost-of-decimation",
            "settle-sensors-and-precision",
        ],
        json.dumps(unresolved),
    )
    check(
        "the contract itself is reported as waiting on nothing",
        next(
            c for c in listed["changes"] if c["name"] == "contract-the-guidance-solver-comparison"
        ).get("follows")
        is None,
    )

    # The archival that is silent today.
    specify(project, coarsening)
    advance(project, "0.4.0")
    checked = archive(project, "measure-the-cost-of-decimation", "--check", "--json")
    payload = json.loads(checked.stdout)
    check(
        "archiving the coarsening change before the contract is reported",
        payload["outOfOrder"] is True
        and payload["unresolvedPredecessor"]["change"]
        == "contract-the-guidance-solver-comparison",
        checked.stdout,
    )
    check(
        "the report names the contract",
        "follows contract-the-guidance-solver-comparison" in checked.stderr,
        checked.stderr.strip(),
    )
    check(
        "and it is reported rather than refused",
        checked.returncode == 0,
        f"exit {checked.returncode}",
    )


def main() -> int:
    if not HTMLSPEC.exists():
        print(f"no binary at {HTMLSPEC}; run `cargo build --release` first")
        return 2
    shutil.rmtree(ROOT, ignore_errors=True)
    ROOT.mkdir(parents=True)

    scene_5_1()
    scene_5_2()
    scene_5_3()
    scene_5_4()
    scene_5_5()

    print()
    if failures:
        print(f"{len(failures)} check(s) failed:")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("every check passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
