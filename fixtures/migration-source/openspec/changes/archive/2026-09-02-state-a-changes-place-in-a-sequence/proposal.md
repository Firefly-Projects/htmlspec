## Why

A proposal in another project using this toolchain states the problem in its own words:

> The contract governs three changes that follow it, and one that follows them. They are
> stated here because the ordering is the argument, and because htmlspec expresses no
> dependency between changes — the convention is enforced by contributors reading it and by
> nothing else.

That author is right on the facts. A change's `.openspec.yaml` carries `schema` and `created`.
The enumeration returns names and, since 0.8.0, task counts. Validation checks a change against
its schema and never against a sibling's state. References resolve within one document —
`refs/unresolved` reads the identifiers the same projection carries, so a unit in one change
cannot address a unit in another. Nothing anywhere relates one change to another.

**The ordering was load-bearing and unenforced.** In that project the sequence is the argument:
a normative contract is written first precisely so the three changes it grades cannot be written
to fit models that already exist. Archive the graded changes before the contract and the
contract becomes a description of the work rather than a constraint on it. The whole method
fails, the artifacts all validate, and nothing reports it.

**This project has met the same fault twice, and named it once.** `project-bootstrap` already
says why a convention nobody enforces is not a gate:

> A directory name is a display string that is shortened and renamed, and a gate derived from
> one depends on a convention nothing enforces — which is how this gate came to compare against
> a value no project ever had.

That is the version gate of 0.6.0, which validated its own invented convention across eleven
archives without refusing anything. Version 0.7.0 was the same shape from the other side: an
instruction telling a contributor to apply a change's contract, with no operation that did it.
Ordering between changes is the third instance. A fact the work depends on, recorded where only
a person reads it.

**Who is blocked and on what.** Any project whose changes form a sequence, which is any project
doing more than one thing at a time. An agent asked to archive "the change we were working on"
gets a listing with completion counts and no indication that one of the candidates must not be
archived yet. A contributor returning after a week has the ordering in prose in a proposal they
must remember to re-read. Neither the listing, nor archival, nor validation will say a word.

**What working looks like the first time you see it.** A contributor writes, in the change that
must come first, that two other changes follow it. Archival of one of those two reports what it
found:

```
note: this change follows contract-the-guidance-solver-comparison, which is not archived.
Archive that one first, or state that this change no longer follows it
```

They archive the contract, then the graded change, and the second archival says nothing. The
enumeration carries the same fact, so a workflow selecting a change to work on can see that one
of them is waiting on another.

**A declaration and not an inference.** The program must not derive an ordering from names,
dates, or requirement identifiers. `artifact-kinds` states that a change is renamed by moving
its directory "because nothing depends on the first name", and an ordering inferred from names
would falsify that sentence. So the ordering is something an author writes down, in the change
that has a predecessor, and the program reads exactly what was written.

**What this change does not do.** It does not block archival. An ordering violation is a finding
of the kind archival already reports and asks about — a contributor who knows the sequence
changed is the authority on that, and a gate that refused would be unable to archive the last
change of an abandoned sequence. It does not order the enumeration by dependency, because the
accepted contract requires that the listing's order not follow what it carries. It does not add
a cross-change reference to the projection: a reference resolves to a unit, and a change is not
a unit. And it does not attempt a general dependency graph with parallel branches and
transitive closure. One relation — this change follows that one — is what the observed problem
needs, and a richer model would be built on no evidence.
