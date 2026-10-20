# The guidance is silent, not wrong

Tasks 1.3 and 1.4, recorded 2026-09-03 against `main` at `80558b5`.

The distinction matters for what the change does. Guidance that stated the rule badly would be
amended. Guidance that never states it needs the rule added, and needs it added everywhere.

## No body draws the line

```console
$ grep -ril "normative" crates/htmlspec-assets/instructions/
$ echo $?
1
```

Nothing. Across all eight installed bodies — `apply.md`, `archive.md`, `continue.md`,
`explore.md`, `ff.md`, `new.md`, `sync.md`, `verify.md` — the word does not appear.

**Eight rather than seven.** The proposal, design, and tasks all say seven, and they were written
before `sync` shipped in 0.7.0. The count is now eight and the obligation lands on all of them.
Recorded here rather than corrected silently in the artifacts, because a task that says "all
seven bodies" is satisfiable while leaving one body without the wording — which is exactly the
failure the pinning test exists to catch.

## What the bodies do say

Each directs an agent at the project's rules and stops short of saying which text is a rule.

| Body | What it says | Line |
| --- | --- | --- |
| `archive.md` | "You apply its contract to the accepted specifications." | `:46` |
| `archive.md` | "### 5. Apply the contract changes to the accepted specifications" | `:192` |
| `new.md` | "Only the rules that apply to every document apply here: parseable markup, resolvable…" | `:117` |
| `explore.md` | "**Read the accepted specifications.** The project's contract states what is already…" | `:48` |

`explore.md` comes closest. It names the accepted specifications as the contract, which is the
right answer to a question it does not ask — it tells an agent where to read and never says what
to do when the reading turns up nothing. An agent that finds no requirement is outside every one
of these sentences.

None of them says a README is not a contract. None says what to do when no requirement exists.

## The contract already states the rule for one case

Task 1.4. This is what makes the change a generalization rather than a new idea, quoted in full
from `openspec/specs/artifact-kinds/spec.md:117`:

> The distinction is between permitted and authoritative. A requirement unit in a proposal SHALL
> be valid, and it SHALL NOT be treated as part of the contract: the contract is what a
> `specification` states. A consumer determining the project's obligations SHALL read
> specifications and SHALL NOT read a proposal's units as binding. This is what "requirements do
> not belong in a proposal" means, and it is a statement about authority rather than about
> validity.

That is the right rule, stated exactly right, for one case. Its subject is a requirement unit
written in the wrong artifact kind — a document the project defines, inside a change the project
holds.

**What it does not reach.** A README is not an artifact kind. A template's example unit is not a
document. A changelog entry is neither. And an agent's assertion in a report is not a document at
all. The statement was never generalized past proposals, so it reaches a requirement in the wrong
artifact and does not reach a layout in a README.

The statement stays exactly as written. This change states the general rule that the specific one
is an instance of, in a new capability, and does not restate the specific one — a rule stated in
two capabilities is the drift 0.8.0 measured and repaired.
