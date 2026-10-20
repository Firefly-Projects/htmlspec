# OpenSpec provides the operation, and this project promised to carry its name

Task 1.2, recorded 2026-08-31.

## The workflow is installed in this repository right now

```console
$ ls .pi/skills/ | grep openspec-sync
openspec-sync-specs
```

`.pi/skills/openspec-sync-specs/SKILL.md` describes the operation in its own first lines:

> Sync delta specs from a change to main specs.
>
> This is an **agent-driven** operation - you will read delta specs and directly edit main
> specs to apply the changes. This allows intelligent merging (e.g., adding a scenario without
> copying the entire requirement).

Two facts follow. The operation exists in the toolchain this project is a spinoff of, and
**OpenSpec's own version is agent-driven too** — so carrying the name does not import a
programmatic merger, because OpenSpec does not have one either.

## The HTMLSpec skill set has no counterpart

```console
$ ls .pi/skills/ | grep '^htmlspec-'
htmlspec-apply-change
htmlspec-archive-change
htmlspec-continue-change
htmlspec-explore
htmlspec-ff-change
htmlspec-new-change
htmlspec-verify-change
```

Seven, matching the seven workflow stages. No sync.

## Why the name is a contract question rather than a preference

`toolchain-packaging` states it:

> **Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for
> it.** [...] The spellings are therefore fixed rather than left to an implementer: `validate`,
> `archive`, `init`, `new`, `list`, `status`, `instructions`, and `store` name the operations
> OpenSpec also provides.

Eight spellings fixed. `sync` is an operation OpenSpec provides and the list does not name it —
because at the time the list was written the operation did not exist here to name. So this is a
hole in a contract that already settled the principle, and not a decision to relitigate.

The same requirement says why fixing the spelling matters: "The workflow instructions this
project installs invoke these commands by name, so the name is part of the contract they depend
on."
