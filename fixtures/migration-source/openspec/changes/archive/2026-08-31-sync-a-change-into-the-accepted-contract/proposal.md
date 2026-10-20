## Why

Archival tells a contributor to perform an operation this project does not have.

When a change's specification states a requirement that no accepted specification holds,
archival reports it and says: *"Apply the change's contract before archiving, or confirm each
one was withdrawn deliberately"* (`crates/htmlspec-cli/src/commands.rs:1352`). There is no
`apply`. There is no command of any name whose subject is the accepted specifications. The
sentence names a step, the contributor goes looking for it, and the command surface has
nothing to offer — so the merge happens by hand, in whatever way each agent invents, at the
one moment the project's contract changes.

**This is the same fault the project already fixed once, in the other direction.** Version
0.6.0's entry opens: "`htmlspec archive` was named for an operation it did not perform." The
repair was to make archival complete what it started. What that repair could not fix is that
archival's *other* half — the merge into the accepted contract — is correctly excluded from
archival and correctly reserved for an agent, and so belongs to an operation that was never
built. `Archival assesses the merge and never performs it` is an accepted requirement and it
is right. The gap is that nothing else performs it either.

**OpenSpec has this operation and this project promised to carry its name.** `openspec sync`
promotes a change's delta specifications into the main specifications without archiving, and
the workflow for it is installed in this repository right now, at
`.pi/skills/openspec-sync-specs/SKILL.md`. The HTMLSpec skill set — `apply`, `archive`,
`continue`, `explore`, `ff`, `new`, `verify` — has no counterpart. `toolchain-packaging`
already states that "where an operation corresponds to one OpenSpec provides, it SHALL carry
OpenSpec's name for it", and fixes the spellings in a list: `validate`, `archive`, `init`,
`new`, `list`, `status`, `instructions`, `store`. `sync` is an operation OpenSpec provides and
that list does not name, which is not an oversight to be argued about — it is a hole in a
contract that already decided the principle.

**Who is blocked and on what.** Anyone finishing a change. Today the accepted specifications
are only ever updated in the minutes before archiving, by an agent working without a stated
contract for the operation, and the only feedback is archival's identifier comparison
afterwards. Two consequences follow. A contributor who wants the contract updated *now* —
because the next change depends on it, or because review should happen against the accepted
text rather than a delta — has to archive early or hand-edit. And an agent doing the merge has
no requirement telling it what it may not do, so nothing on paper stops it from rewriting an
accepted requirement it was only supposed to add a scenario to.

**What working looks like the first time you see it.** A contributor finishes the
implementation, and before archiving runs the sync operation against their change. The program
reports which requirement identifiers the change states, which the accepted specifications
already hold, and which are absent — the same comparison archival makes, from the same code,
so the two can never disagree. The agent then merges those requirements into the accepted
specifications itself, reading both sides for meaning, because deciding which accepted
requirement a new scenario belongs under is judgment. It re-runs sync and the absent list is
empty. Archival, run later, finds nothing to report about the merge, because the merge already
happened and was checked by the operation named for it.

**The thing to get right, and the tempting thing to build instead.** The tempting build is a
merger: parse the change's specification, parse the accepted ones, splice requirements across.
It would work on the easy cases and it would be wrong. `artifact-kinds` reserves the judgment
of *where a requirement belongs* for an agent, and `Archival assesses the merge and never
performs it` says the program does not propose the text of a merge. A programmatic sync would
contradict both, and would do it silently — producing an accepted specification nobody decided
on, in the file that is the project's contract.

So sync divides exactly where archival divides, and for the reason archival already states:
observing that `req-x` appears in one document set and not another is a set membership test,
and deciding what `req-x` means next to the requirements already accepted is not. The program
does the membership test and reports it. The agent does the merge. What this change adds is
not a merger; it is the operation that tells an agent what remains, refuses to guess, and can
be run again to prove the work landed.

**What this change does not do.** It does not modify archival's behavior, which is already
correct — archival keeps assessing and keeps refusing to write. It does not add an artifact
kind. It does not decide the arrangement of a merged specification, because that is the
agent's and stating it here would be inventing a convention no requirement supports.
