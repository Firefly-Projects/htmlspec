# The measurement repeated against HEAD

**Date:** 2026-10-08
**Tasks:** 1.1, 1.2, and 1.3
**Binary:** `htmlspec 0.24.0 (b768860d5b82)`, built in this change's worktree at `b768860` before
any edit. Models come from a copy of `~/projects/html-spec-migration/tools/project-model` whose
path dependency names this worktree's `crates/htmlspec-parse`, built at the same commit. The copy
and both scripts below live in the session's scratch directory, and the scripts are in the
appendices so the run can be repeated. The staged migration was copied into the scratch directory
before anything read it, and the original's 291 files hash the same afterwards.

## 1. A risk with no mitigation, in a scratch project (task 1.1)

`htmlspec init --tools none` and `htmlspec new change settle-a-risk` both exit 0. The design's
placeholder summary is replaced with "This design records one risk the project bears.", and each
variant's units go just before `</main>`. The attribute is spelled `data-response`, as the note of
2026-10-01 in `the-measurement-2026-09-30.md` says. Each variant differs from the one before it in
one way, as there:

- **A:** a risk with a statement and nothing else;
- **B:** A plus `data-response="accepted"` on the risk;
- **C:** B plus `<p data-field="mitigation">Not mitigated, and recorded rather than hidden.</p>`;
- **D:** C without `data-response`;
- **E:** D plus a decision carrying `data-status="accepted"`, `data-state="done"`,
  `data-outcome="passed"`, and, new in this run, `data-response="accepted"`.

| Variant | Response written | Mitigation paragraph | Exit | Rule | `response` in the model |
| --- | --- | --- | --- | --- | --- |
| A | none | none | 1 | `schema/invalid` | absent |
| B | `accepted` | none | 1 | `schema/invalid` | absent |
| C | `accepted` | "Not mitigated, …" | 0 | none | absent |
| D | none | "Not mitigated, …" | 0 | none | absent |
| E | none on the risk, `accepted` on the decision | "Not mitigated, …" | 0 | none | absent |

**Every expected result holds.** A and B each report one diagnostic, the same as on 2026-09-30:

```
htmlspec/changes/settle-a-risk/design.html
  error: schema/invalid at risk-the-inputs-are-prose (/risks/0): "mitigation" is a required property

  1 error, 0 warnings: validation failed
exit 1
```

- **The `--json` payloads of A and B are byte-identical**, SHA-256 `79d563fa…`, and so are their
  projected models, `8bdb93eb…`. The model holds `change`, `id`, `statement`, `title`, and `unit`,
  and no `mitigation` or `response`.
- **The models of C and D are byte-identical**, `58f6c98b…`, with the workaround sentence as the
  risk's `mitigation` and no `response`.
- **E exits 0 with no diagnostics.** Its decision projects `"status": "accepted"` and nothing from
  `data-state`, `data-outcome`, or `data-response`. The risk's model is C's and D's.

So HEAD still ignores `data-response` everywhere, under section 2.2, and the only valid way to write
an accepted risk is still a mitigation that says it is not one.

## 2. Every risk unit in the repository and in the staged migration (task 1.2)

**Tracked HTML.** `git ls-files '*.html'` lists 127 files, five more than on 2026-09-30. Each was
projected, and each projected risk was checked for a `mitigation`. The elements carrying the `risk`
class token were also counted with a parser.

| Where | Files with a risk | Risk units |
| --- | --- | --- |
| `fixtures/conformance/` | 68 | 70 |
| `fixtures/projection/` (`named-fields`, `single-field-repeated`) | 2 | 2 |
| `fixtures/dogfood/foundation-specification.html` | 1 | 2 |
| `assets/template-specification.html` | 1 | 1 |
| `runbooks/release/specification.html` | 1 | 2 |
| `2026-08-10-html-spec/documents/deterministic-formatting.html` | 1 | 1 |
| `2026-08-14-separate-artifact-kinds/documents/design.html` | 1 | 3 |
| `2026-09-11-migrate-an-openspec-project/evidence/a-migrated-design-2026-09-09.html` | 1 | 6 |
| **Total** | **76** | **87** |

88 elements carry the token. The 88th is still `risk-both` in
`invalid-schema-projection-two-reserved-classes`, which projects no unit. **All 87 projected risks
carry a mitigation. None lacks one.**

**What moved since 2026-09-30.** The five new files are the four conformance fixtures that
`reject-change-markup-inside-a-unit`, `state-a-modified-requirement-whole`, and
`mark-the-words-a-modification-changed` added (`invalid-schema-change-markup`,
`invalid-schema-change-markup-in-a-modified-unit`, `valid-change-markup-outside-a-unit`, and
`valid-modified-requirement-stated-whole`), each carrying the base's one risk, and the projection
fixture `change-markup`, which carries none. So 83 became 87, 66 became 70, and 64 fixture inputs
carrying the template's risk became 68. No file was removed.

**The staged migration.** 116 HTML files, and the same 11 risk units in the same four files as on
2026-09-30: 1, 3, and 6 in the three carried evidence files, and the one risk the migration drafted,
in `2026-09-06-distinguish-a-requirement-from-an-example/design.html`. **All 11 carry a
mitigation.**

**The default of design decision 1 fits every risk counted.** No risk lacks a mitigation, so every
one of the 98 keeps its validity under a default of `mitigated`. The drafted risk is the one the
2026-09-30 measurement already named: its author calls it accepted, and under the default it
projects `mitigated` until a migration declares otherwise. That is
`migrate-into-the-extended-vocabulary`'s work, and this change leaves it as it stands.

## 3. The contract version at HEAD (task 1.3)

`CONTRACT_VERSION` in `crates/htmlspec-schema/src/lib.rs` is `0.4.0`, and the schema's
`schemaVersion` enumeration is `0.1.0`, `0.2.0`, `0.3.0`, and `0.4.0`. It was `0.2.0` when this
change was written. `reject-change-markup-inside-a-unit` moved it to `0.3.0` and
`mark-the-words-a-modification-changed` to `0.4.0`.

**This change is built third in a stack of four.** `record-a-task-done-differently` moves the
contract to `0.5.0` and `record-a-partial-verification` to `0.6.0`, and both are built in parallel
from the same `main`. This change therefore moves the contract from `0.6.0` to `0.7.0`. Its branch
starts where the contract is `0.4.0`, so the enumeration on this branch reads `0.1.0` to `0.4.0`
and then `0.7.0`: the `0.5.0` and `0.6.0` entries, their changelog entries, and their fixtures
arrive with the changes below this one when the stack is assembled.

## Appendix A: `measure.py`

Run as `measure.py <htmlspec> <project-model> <work dir>`. It builds the scratch project, writes
the five variants into the design one at a time, and records each variant's document, human report,
`--json` payload, exit codes, and projected risks and decisions.

```python
import json, pathlib, shutil, subprocess, sys

binary, project_model, work = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
if work.exists():
    shutil.rmtree(work)
work.mkdir(parents=True)
proj = work / "proj"

def run(*args, cwd=None):
    done = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    return done.returncode, done.stdout, done.stderr

assert run(binary, "init", "--tools", "none", "--root", str(proj))[0] == 0
assert run(binary, "new", "change", "settle-a-risk", cwd=proj)[0] == 0
design = proj / "htmlspec/changes/settle-a-risk/design.html"
template = design.read_text()
start = template.index("<p>Replace this paragraph with what this design settles.")
end = template.index("</p>", start) + len("</p>")
template = template[:start] + "<p>This design records one risk the project bears.</p>" + template[end:]

STATEMENT = ("<p>A run on a machine missing one of the four inputs discovers it partway through, "
             "with the version already advanced.</p>")
WORKAROUND = '<p data-field="mitigation">Not mitigated, and recorded rather than hidden.</p>'

def risk(response, mitigation):
    attribute = f' data-response="{response}"' if response else ""
    lines = [f'        <section class="risk" id="risk-the-inputs-are-prose" data-change="added"{attribute}>',
             '          <h3>The inputs are prose that nothing checks <span class="hs-id">risk-the-inputs-are-prose</span></h3>',
             f"          {STATEMENT}"]
    if mitigation:
        lines.append(f"          {mitigation}")
    lines.append("        </section>")
    return "\n".join(lines)

DECISION = "\n".join([
    '        <section class="decision" id="decision-run-inside-the-shell" data-change="added" '
    'data-status="accepted" data-state="done" data-outcome="passed" data-response="accepted">',
    '          <h3>Run inside the pinned shell <span class="hs-id">decision-run-inside-the-shell</span></h3>',
    "          <p>The procedure is run inside the pinned development shell.</p>",
    '          <p data-field="rationale">The shell supplies all four inputs.</p>',
    "        </section>"])

VARIANTS = {"A": risk(None, None), "B": risk("accepted", None), "C": risk("accepted", WORKAROUND),
            "D": risk(None, WORKAROUND), "E": risk(None, WORKAROUND) + "\n" + DECISION}

results = work / "results"
results.mkdir()
for name, units in VARIANTS.items():
    document = template.replace("      </main>", units + "\n      </main>")
    design.write_text(document)
    relative = "htmlspec/changes/settle-a-risk/design.html"
    code, human, human_err = run(binary, "validate", relative, cwd=proj)
    json_code, payload, _ = run(binary, "validate", "--json", relative, cwd=proj)
    (results / f"{name}.human.txt").write_text(human + human_err + f"exit {code}\n")
    (results / f"{name}.json").write_text(payload)
    model = json.loads(run(project_model, str(design))[1])[str(design)]
    picked = {"risks": model.get("risks", []), "decisions": model.get("decisions", [])}
    (results / f"{name}.model.json").write_text(json.dumps(picked, indent=2, sort_keys=True) + "\n")
```

## Appendix B: `count_risks.py`

Run as `count_risks.py <project-model> <repository root> <staged root> <out json>`. It projects
every tracked `.html` file and every `.html` file under the staged root in batches of 50, lists
each projected risk with whether it has a `mitigation` and what `response` it projects, and counts
the elements whose `class` holds the token `risk` with Python's `html.parser`.

```python
import collections, html.parser, json, pathlib, subprocess, sys

project_model, repo, staged, out = sys.argv[1], pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]), sys.argv[4]

class RiskTokens(html.parser.HTMLParser):
    def __init__(self):
        super().__init__()
        self.count = 0
    def handle_starttag(self, tag, attrs):
        self.count += sum(1 for n, v in attrs if n == "class" and v and "risk" in v.split())

def project(paths):
    models = {}
    for start in range(0, len(paths), 50):
        batch = [str(p) for p in paths[start:start + 50]]
        models.update(json.loads(subprocess.run([project_model, *batch], capture_output=True,
                                                text=True, check=True).stdout))
    return models

def survey(paths):
    models, rows, tokens = project(paths), [], 0
    for path in paths:
        parser = RiskTokens()
        parser.feed(path.read_text())
        tokens += parser.count
        for risk in models[str(path)].get("risks", []):
            rows.append({"path": str(path), "id": risk.get("id"),
                         "has_mitigation": "mitigation" in risk, "response": risk.get("response")})
    return {"files": len(paths), "risk_class_tokens": tokens, "risks": len(rows),
            "with_mitigation": sum(r["has_mitigation"] for r in rows),
            "responses": dict(collections.Counter(str(r["response"]) for r in rows)), "rows": rows}

tracked = subprocess.run(["git", "ls-files", "*.html"], cwd=repo, capture_output=True,
                         text=True, check=True).stdout.split()
report = {"tracked": survey([repo / p for p in tracked]),
          "staged": survey(sorted(staged.rglob("*.html")))}
pathlib.Path(out).write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
```
