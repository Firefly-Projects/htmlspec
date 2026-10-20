# The measurement, repeated against the built change

**Date:** 2026-10-02
**Task:** 5.1
**Binary:** `htmlspec 0.20.0`, built from this change's working tree with the comparison, the
payloads, and the instructions in place.
**Script:** `run-all.sh` and the scripts it calls, unchanged from the appendices of
`the-measurement-2026-09-30.md`, run from an empty directory into `run-all-after.log` (657 lines).

## Every step now reports what was and was not merged

| Step | What the change states, and what was merged | Before (`the-measurement-repeated-2026-10-02.md`) | After |
| --- | --- | --- | --- |
| 1 | `req-x` `modified`, one word added. Nothing merged | `req-x` reached, "applied in full", "ready" | `differs req-x (statements from paragraph 1)`, `reached: []`, "has not been synced", `needsSync: true` |
| 2 | as step 1, plus scenario C `added` | the same as step 1 | `differs req-x (statements from paragraph 1; not held: scenario-x-c)`, "has not been synced" |
| 3a | as step 2, plus `req-y` `removed`. Nothing merged | `req-x`, `req-y` reached, "applied in full", "ready" | `req-x` differs as in step 2, `held req-y (this change marks it removed)`, `untakenRemovals: ["req-y"]` |
| 3b | the same change. `req-y` deleted | `req-y` absent, "has not been synced" | **`reached req-y`**, `absent: []`. `req-x` still differs, so the change is still not synced |
| 4 | as step 3a, plus `req-z` `unchanged`. `req-y` deleted | `stated`: `req-x`, `req-y`, `req-z`. `req-y` absent | `reached req-y, req-z`, `req-x` differs. `stated` unchanged |
| 5 | the step 4 change. **Every part merged correctly** | `req-y` absent, **"has not been synced"**, `needsSync: true` | **`reached req-x, req-y, req-z`, "the change's contract has been applied in full", "ready: archival would archive this change", `needsSync: false`** |
| 6 | the step 4 change. **Nothing merged** | all three reached, **"applied in full"**, **"ready"** | **`reached req-z`, `differs req-x`, `held req-y`, "has not been synced", `needsSync: true`** |

**Steps 5 and 6 have traded answers, which is the change.** The correct merge reads as applied in
full and ready, and the untouched accepted specification reads as unsynced, naming each part that did
not arrive.

Every `sync` and `archive --check` run exits 0, 28 of them, as before. The step 3b archival under a
pseudo-terminal still exits 3, and without a terminal or `--yes` still exits 2, and both leave the
project unchanged. At step 3b it still asks, because `req-x`'s content has not been merged, and no
longer because of `req-y`. Every `validate` line is identical to the run before the change.

## Step 6, as printed

```
$ htmlspec sync --change measure
1 of 3 requirement(s) this change states have reached the accepted specifications
  reached  req-z
  differs  req-x (statements from paragraph 1; not held: scenario-x-c)
  held     req-y (this change marks it removed)
standard error:
note: 1 requirement(s) this change adds or modifies hold other content in the accepted specifications. Give each accepted requirement the statements and scenarios this change states, reading from the statement and the scenarios named. This command reports where the two differ and never edits an accepted specification
note: 1 requirement(s) this change withdraws are still held in the accepted specifications. Delete each from the accepted specification that holds it. This command reports the comparison and never edits an accepted specification
exit 0

$ htmlspec sync --change measure --json
{
  "absent": [],
  "change": "measure",
  "reached": ["req-z"],
  "stated": ["req-x", "req-y", "req-z"],
  "untakenContent": [
    { "firstDifferingStatement": 1, "id": "req-x", "scenariosNotHeld": ["scenario-x-c"], "scenariosNotKept": [] }
  ],
  "untakenRemovals": ["req-y"],
  "untakenRenames": []
}
exit 0

$ htmlspec archive measure --check
ready with questions: this change has not been synced, and archival would report the findings above and ask
standard error:
note: 1 requirement(s) this change adds or modifies hold other content in the accepted specifications: req-x (statements from paragraph 1; not held: scenario-x-c). This change has not been synced. Run `htmlspec sync --change measure` to see what remains, and give each accepted requirement the statements and scenarios this change states
note: 1 requirement(s) this change withdraws are still held in the accepted specifications: req-y. This change has not been synced. Run `htmlspec sync --change measure` to see what remains, and delete each from the accepted specification that holds it
exit 0
```

The JSON payload is reflowed. Its members and values are as printed. **No statement or scenario text
from either side appears in any payload or note.** The word "quickly", which is the whole of the
untaken edit, is in neither.
