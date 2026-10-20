## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **One function pairs a change's requirement with an accepted one.** `merge_state` in
  `crates/htmlspec-cli/src/commands.rs` (3919 as built) reads every accepted specification, builds
  `held`, a map from identifier to every accepted requirement carrying it, as projected and in the
  order the specifications are discovered (3935), and looks each stated identifier up in it (3961).
  `sync` and archival both call it (evidence, section 5). It keeps no path.
  `what-it-builds-on-2026-10-07.md` reads it as built.
- **`see-a-modification-the-merge-did-not-take` turns that into a comparison of content.** As built,
  `untaken_content(requirement, id, accepted)` compares one stated requirement with every accepted
  requirement carrying its identifier, through `content_difference`, and returns
  `Option<UntakenContent>`: `None` where any of them holds the content, and otherwise the difference
  from the first. The entry's fields are private: `first_differing_statement`, a position counted from
  one, and `scenarios_not_held` and `scenarios_not_kept`, each scenario named by its identifier, by its
  heading name where it has none, and by its position where it has neither. Content is the statements
  in order and the scenarios, less those marked `removed`, matched as a multiset of texts: heading
  name, then given, when, and then entries. Every text is compared after whitespace normalization.
  Notes and the `renamed-from` paragraph are not content. It reports positions and names and never
  text, on purpose, and its design's decision 5 leaves "every differing paragraph, found by aligning
  the two sequences" to this change.
- **Which requirements the comparison compares by content is one closed type.** `ContractEffect` is
  `Establishes` for a requirement marked `added` or `modified`, `Withdraws` for one marked `removed`,
  and `CarriesForward` for one marked `unchanged`. `ContractEffect::of_requirement` reads a requirement
  marked `unchanged` that holds a scenario the change adds, modifies, or withdraws as `Establishes`,
  which `see-a-modification-the-merge-did-not-take`'s review added. The two states that establish text
  are found by their stems.
- **`mark-the-words-a-modification-changed` gives `ins` and `del` a meaning, but no positions in the
  model.** A `del`'s content supplies no field, so every statement is current text. Each requirement
  and scenario gains `deletions`, a list of deleted text. Its decision 2 says: "Nothing pairs the two.
  The markup keeps the positions, and `show-what-a-modification-changed` can show them against the
  accepted text." The markup is permitted only in a requirement or scenario marked `modified` in a
  change's specification. The projection's walk is `own_nodes`, which tags each node `Current` or
  `Deleted`, and `own_descendants`, which every field reads, keeps the current ones.
- **`state-a-modified-requirement-whole` fixes what a modified requirement contains.** Every statement
  and scenario is stated, each scenario carries its own change state, and a scenario carried forward,
  altered, or withdrawn keeps its accepted identifier. A statement paragraph has no state and no
  identifier.
- **A change is resolved by `resolve_change` (1103) and `resolve_change_in` (1174).** A directory path
  is taken as the change wherever it is, and a name is looked up under `changes/` only. So an archived
  change is reachable by its path (evidence, section 5). With `--store`, the name is composed under the
  store's `changes/`, so `archive/<dated name>` reaches an archived change there too.
- **The program orders the archive only for runbooks.** `runbook_release::history` sorts runbook
  snapshots by recorded project version. For changes, `last_archived_version` takes the greatest
  version (evidence, section 5).
- **The exit-code contract is one type.** `exit::Outcome` has `Success` (0), `DocumentViolation` (1),
  `OperationalFailure` (2), and `Cancelled` (3). `main.rs` already reports an invalid pairing of
  options by name for `run`, rather than with clap's generic message.
- **Instruction bodies are checked against the command tree.** `invocations.rs`'s
  `no_shipped_body_names_an_absent_command` fails when an instruction names `htmlspec <word>` and clap
  has no such subcommand.
- **OpenSpec's view, measured** (evidence, sections 1 and 2): the proposal, then per capability each
  ADDED requirement whole and each MODIFIED one as a unified-diff hunk, with two paragraphs of context
  on each side.
  `(no textual changes)` once the accepted text holds the edit. `Unknown item`, exit 1, once the change
  is archived.

## Goals / Non-Goals

**Goals:**

- A reviewer finds the words a modification changed without reading both texts side by side.
- The view and `sync` cannot disagree about whether a modification reached the accepted text.
- An author's marks are shown where the accepted text confirms them, and never where it does not.
- The output cannot be mistaken for merge text, or fed to a tool as one.
- The view reads and never writes.

**Non-Goals:**

- **An archived change.** See decision 8.
- **A default view for `show <change>`, and `show` for an accepted specification.** See decision 2.
- **The text of removed or carried-forward requirements.** See decision 7.
- **Presentational markup, notes, and `renamed-from`.** The view shows what the comparison compares.
- **Colour, paging, and wrapping to the terminal's width.** Each paragraph is one line, so a text can
  be searched and copied whole, and the terminal wraps it.

## Decisions

### 1. The command is `htmlspec show <change> --diff`

The change is a positional argument, and `--diff` is a flag. `openspec show --help` gives
`Usage: openspec show [options] [item-name]` and `--diff  Show per-requirement diffs for delta specs
(change)` (evidence, section 3). toolchain-packaging's "The command surface names its operations as
OpenSpec names them" carries OpenSpec's name for the subcommand "and for the options that select what
it acts on", so both spellings are OpenSpec's.

**Considered: `--change <name>`, as `status` and `sync` take it.** Rejected. `status` takes
`--change` because OpenSpec's `status` does (`openspec status --help`: `--change <id>`). OpenSpec's
`show` takes its item positionally, and a second spelling for the same argument is what the naming
requirement forbids.

**Considered: `sync --diff`.** Rejected. `sync` is the operation a merging agent runs, and its
report carries no text by design. Putting both texts under the command whose output is a worklist
would make them read as instructions.

**Considered: a subcommand `diff`.** Rejected. OpenSpec provides this operation under `show`, and an
operation it provides takes its name.

### 2. `show <change>` without `--diff` refuses

`Command::Show` has `diff: bool`. Where it is false, `main.rs` reports, as it does for `run` without
`--as`:

```
htmlspec: show has one view today: `htmlspec show <change> --diff`, which shows each requirement the
change modifies beside the accepted one. Pass --diff
```

and exits 2. Standard output is empty, with `--json` or without it.

**Considered: print a short summary of the change.** Rejected. `status` already reports what a change
holds, and a second report of the same facts is a second answer to one question.

**Considered: print the proposal, as OpenSpec does** (evidence, section 1). Rejected. An html-spec
proposal is a page a browser renders. Printed to a terminal it is markup, and printed as projected
text it is a page with its structure removed.

**Considered: make the diff the default, with `--diff` accepted and changing nothing.** Rejected.
A flag that changes nothing is a name that breaks its promise. If a later change gave `show` a default
view, every caller that left `--diff` out would start receiving something else.

Refusing costs the contributor who types the bare form one line that names the fix, and keeps the
bare form free for a later change to define.

### 3. One pairing and one comparison, shared with `sync` and archival

The view does not compare texts on its own. It calls the two things
`see-a-modification-the-merge-did-not-take` builds for `sync` and archival:

- **The accepted index.** The map `merge_state` builds is taken out into one function,
  `accepted_requirements(project)`, that `merge_state` and the view both call. Each entry keeps every
  accepted requirement carrying the identifier, as projected and in discovery order, and beside each
  the path of the specification that holds it, which the view reports. Which of two holders a
  requirement is compared with is the comparison's choice, made once: one that holds the content where
  there is one, and otherwise the first.
- **The content comparison.** The function that returns `Option<UntakenContent>` for one stated
  requirement. `None` means the accepted text holds the requirement as stated, and the view shows it
  as held as stated. `Some` means it differs. `firstDifferingStatement` is where the statements
  begin to differ, and `scenariosNotHeld` and `scenariosNotKept` are the scenarios decision 6 pairs.
  As built it names those scenarios as strings, so task 3.2 has it also give the view the scenarios
  themselves, and builds `UntakenContent` from that same result.
- **Which requirements are compared.** Those `ContractEffect::of_requirement` reads as establishing
  text, as decision 7 states.

The view then adds the part that comparison left out: which paragraphs differ, how they pair, and
which words differ inside a pair.

A test, `the_view_and_sync_agree_about_what_differs`, runs both over one fixture. The fixture holds a
modification taken, one not taken in a statement, one not taken in a scenario, and a scenario
withdrawn but still held. The test asserts three things:

- the view shows a requirement's content as differing exactly where `sync` puts it in
  `untakenContent`;
- the first paragraph the view does not show as held by both is `firstDifferingStatement`;
- the scenarios the view shows on each side are `scenariosNotHeld` and `scenariosNotKept`.

**Considered: a comparison of the view's own in `show.rs`.** Rejected. It would be a second
definition of "differs", and the two would drift where they are hardest to see: in normalization, in
what counts as content, and in scenario order. Then the view would show a difference `sync` calls
reached, or the reverse, and a contributor would be left with two answers.

**Considered: diffing the documents' source.** Rejected. The source differs where nothing normative
does: `data-change` differs between a change's specification and an accepted one by design, the
formatter lays out both, and `ins` and `del` sit only in the change's.

### 4. Statements: two alignments, then words

One generic function aligns two sequences:

```rust
enum Step { Both(usize, usize), Left(usize), Right(usize) }

fn align<T>(left: &[T], right: &[T], same: impl Fn(&T, &T) -> bool) -> Vec<Step>;
```

It removes the common prefix and suffix, then finds a longest common subsequence of the rest under
`same`, breaking ties toward the left side so the same inputs always give the same steps. It is pure
and total, and it is generic over `T`, so it cannot look at what it aligns except through `same`. It
is used three times:

1. **Paragraphs, by equality** of normalized text. A `Both` step is a paragraph held by both. Because
   the common prefix is removed first, the first step that is not `Both` falls at
   `firstDifferingStatement`, which is what the parity test in decision 3 checks.
2. **Within each run of paragraphs between two held by both, by likeness.** Two paragraphs are alike
   when their words align with at least half of each one's words in common. A `Both` step here is a
   pair. A paragraph left over is shown whole, as stated only by the change or held only by the
   accepted specification.
3. **Words, by equality, inside a pair.** A word is a run of the projected text between whitespace.
   The words of a `Left` step are marked on the accepted side and those of a `Right` step on the
   change's.

The prototype in the evidence, section 6, runs this on the one-word case and finds one pair and one
marked word, `sync,`. That matches OpenSpec's hunk.

**Why half.** A pair that shares fewer than half its words is more marks than text, and two whole
paragraphs read better. `mark-the-words-a-modification-changed` gives an author the same advice for a
paragraph rewritten throughout: delete it and add it whole. The threshold is a judgment rather than a
measurement, and task 9.1 asks the contributor to confirm it.

**Considered: pair leftover paragraphs by position**, as the prototype does. Rejected. Where a change
adds a paragraph before one it edits, position pairs the new paragraph with the old one, and the
edited one is left over. The reader then sees three whole paragraphs and no marks, where the likeness
alignment shows one added paragraph and one pair with its word marked. The spec's scenario "A
paragraph added beside an edited one" holds that case.

**Considered: one word diff across the whole run**, as `git diff --word-diff` does over a hunk.
Rejected. A statement is a sequence of paragraphs, and the comparison compares them in order, one
paragraph at a time. A word diff across paragraph breaks loses the paragraph, and it can mark half of
one paragraph as belonging to the next.

**Considered: whole paragraphs, old then new, with no word marks.** This is OpenSpec's view.
Rejected. The evidence shows its cost: the reader finds `` `sync`, `` by comparing two lines of about
sixty words by eye (section 1).

**Considered: the `diff` crate, 0.1.13, already in `Cargo.lock`.** Rejected. It is there only through
`pretty_assertions`, a dev-dependency, so using it would add it to the shipped program. Its API also
compares with `PartialEq`, and the second alignment needs a predicate. The function above is short,
and a property test covers it: its `Both` steps never cross, their number equals a brute-force
longest common subsequence on small inputs, and the same inputs give the same steps.

### 5. The author's marks, where the accepted text confirms them

`htmlspec-parse` gains a reader of a unit's change markup:

```rust
pub enum Run { Kept(String), Inserted(String), Deleted(String) }
pub struct MarkedText(Vec<Run>);              // never empty
pub struct MarkedUnit {
    pub statements: Vec<MarkedText>,          // document order, including a paragraph wholly deleted
    pub given: Vec<MarkedText>, pub when: Vec<MarkedText>, pub then: Vec<MarkedText>,
}
pub fn marked_units(document: &NodeRef) -> BTreeMap<String, MarkedUnit>; // only units carrying ins or del
```

It walks a unit's nodes as the projection does, through `own_nodes`, which keeps what sits inside a
`del` where `own_descendants` drops it. It selects the same statement paragraphs and scenario entries,
keeping the `ins` and `del` boundaries the projection drops. Two shapes hold a paragraph wholly
deleted, `<del><p>…</p></del>` and `<p><del>…</del></p>`, and neither projects a statement. A paragraph
inside an `ins` is wholly inserted, and since the walk tags no node by `ins`, the reader tracks it. A property test
holds it to the projection. For every mapping fixture, including `mark-the-words-a-modification-changed`'s
`change-markup`, and every staged document, a unit's `Kept` and `Inserted` text, normalized, is its
projected statements and entries. A paragraph wholly inside a `del` is the only one with no current
text.

For a requirement's statements, and separately for each scenario's entries, the view reads the marked
text back as the text before the edit: each `Kept` and `Deleted` run, without the `Inserted` ones. It
then compares that text with the accepted text, as normalized text, which is the comparison's own
test.

- **Equal:** the author's marks account for the whole difference. The view uses them as written. A
  paragraph with only `Kept` runs is held by both, and a paragraph with an `Inserted` or `Deleted` run
  is a pair. A paragraph wholly inserted is stated only by the change, and one wholly deleted is held
  only by the accepted specification. Inserted words are marked on the change's side and deleted
  words on the accepted side, where the author put them. Decision 4's alignments are not used.
  `marks` is `author`.
- **Not equal:** the marks leave part of the difference unmarked, or the accepted text has moved since
  the change was written. The view uses decision 4 over the current text and says that the author's
  marks do not account for the difference. `marks` is `unreconciled`.
- **No marks:** decision 4, and `marks` is `comparison`.

The three values are one closed type, `Marking { Author, Comparison, Unreconciled }`. The rendering
matches on it, so a fourth case would be a compile error rather than a silent default.

**Considered: always compute the marks.** Rejected. The page would show the author's account of the
edit and the terminal a different one wherever an alignment pairs words differently from the author.
An author who marks `<del>the whole list</del><ins>every list</ins>` as one phrase sees, from a
computed alignment, `list` kept and only the words around it marked. A reviewer holding both would be
comparing two descriptions of one edit.

**Considered: always show the author's marks.** Rejected. `mark-the-words-a-modification-changed`'s
own risk section says a mark "claims only what it marks", and that the diff is "where incompleteness
shows". Marks shown unchecked would hide exactly the words an author forgot to mark.

**Considered: read the markup in the CLI with `kuchikiki`.** Rejected. Which paragraphs are statements
and which are scenario entries is the projection's rule. A second walk in the CLI is a second
definition, and the property test could not tie it to the first.

**Considered: a model field holding runs.** Rejected. `mark-the-words-a-modification-changed` rejected
reshaping `statements` into runs, because it breaks every reader of the model to serve this view. The
reader adds nothing to the model or the schema, so the document contract does not move.

### 6. Scenarios: by text, then by identifier

The content comparison has already settled which scenarios both sides hold, by text. The view pairs
the rest:

| The change's scenario | The accepted requirement | Shown as |
| --- | --- | --- |
| its text is held | — | held by both |
| in `scenariosNotHeld` | holds one with the same identifier, in `scenariosNotKept` | a pair: heading and each entry list aligned as statements are (decisions 4 and 5) |
| in `scenariosNotHeld` | holds none with that identifier | stated only by the change, whole |
| marked `removed` | still holds one with its identifier, or, where no identifier finds one, with its text | withdrawn, still held |
| marked `removed` | holds none with its identifier or its text | withdrawn, no longer held |
| — | holds one in `scenariosNotKept` that nothing above pairs | held only by the accepted specification, whole |

Each scenario the change states is shown with the state it declares. The view does not judge a
declared state against the text. A scenario marked `unchanged` whose text differs is shown as a pair
under its declared state, and the `sync` instructions already tell the merging agent to report it.

**Why identifiers now, when the comparison uses none.** The comparison asks whether the accepted
requirement says a text, and an identifier does not answer that. The view asks which accepted
scenario a stated one replaces, and that is what `state-a-modified-requirement-whole`'s identifier
sentence answers. That change's design (decision 4) kept the sentence and
`see-a-modification-the-merge-did-not-take`'s review task asks whether it still has a reader. This view
is one.

**Considered: pair by heading.** Rejected. A change may alter a scenario's heading, which is why
`state-a-modified-requirement-whole` pairs by identifier.

**Considered: pair by position.** Rejected. The comparison gives a scenario's position no meaning,
and a merge may append kept scenarios after new ones.

**Considered: pair by likeness, as paragraphs are.** Rejected. An identifier already says which
scenario is which, and likeness would override the author's statement of it with a guess.

### 7. The view compares what the comparison compares, and lists the rest

**Settled with the contributor on 2026-10-07**, after task 1.1 read the comparison as built.

A requirement the comparison reads as establishing text is compared: one marked `added` or
`modified`, and one marked `unchanged` that holds a scenario the change adds, modifies, or withdraws.
Each is shown with the state the change declares on it. A requirement marked `removed`, and one
marked `unchanged` whose every scenario is marked `unchanged`, is listed with its identifier, its
heading, and its declared state, under "listed, not compared".

The view asks `ContractEffect::of_requirement` which case a requirement is in, so there is one
reading of a declared state and not two. It needs no refinement of that type: it shows the declared
state as the document declares it, and asks the type only whether to compare.

As first written, this decision compared only a requirement marked `modified`, and listed every other.
The comparison as built compares two more kinds, and the view would have contradicted `sync` on both:

- **A requirement marked `unchanged` that holds a changed scenario.**
  `see-a-modification-the-merge-did-not-take`'s review made the comparison compare it, because judged
  by its own mark alone, a scenario added inside it reached the contract unmerged. Listed, the view
  would hide that scenario's edit while `sync` reported it.
- **A requirement marked `added` whose identifier is held with other text.** `sync` reports it in
  `untakenContent`, and `sync.md` asks the merging agent to tell whether the accepted requirement is
  this change's, merged incompletely, or another obligation under the same identifier. Listed, the view
  would show nothing of the two texts that decide it.

**Considered: `modified` alone, as first written.** Rejected, for those two cases. Its argument was
that an added requirement is held by none before the merge and held as stated after it. That holds
for the ordinary case, where comparing costs one line, and fails exactly where a merge went wrong.

**Considered: compare `modified` and the `unchanged` case, and keep listing `added`.** Rejected. The
parity of decision 3 would then hold only for the requirements the view compares, and an added
requirement merged otherwise would be the one case where `sync` and the view disagree.

**Considered: print an added requirement whole, as OpenSpec does.** Rejected. OpenSpec prints it whole
even after the accepted text holds it (evidence, section 1), so the reader cannot tell from the view
whether anything is left to do. Compared, it is one line in both ordinary states.

**Considered: omit the listed ones.** Rejected. A view that silently skips a change's requirements
invites the reading that the change states nothing else.

### 8. An archived change is refused

`ProjectRoot` gains `is_archived_change(directory)`, which is true where the directory's canonical
parent is the canonical `archive()`, beside `is_change_evidence`. The view asks it after resolving
the change on either route, before reading anything else, and refuses:

```
htmlspec: <directory> is an archived change. The accepted specifications hold its edit together with
every later change's, so a comparison with them shows a later change's edit, reversed, where there is
one, and nothing where there is none. `show --diff` compares a change before its merge. What an
archived change modified is in its specification's marks, where it has them, and in the repository's
history
```

It exits 2, the code for an operation that could not run.

The evidence, section 6, shows both outcomes on this repository's staged archive:

- `2026-08-31-sync-a-change-into-the-accepted-contract` compared with today's text shows nothing.
- `2026-08-30-finish-the-archival` shows one paragraph where the accepted text holds `sync,` and the
  change does not. That word is the next change's edit. finish-the-archival's own edit, which includes
  a new heading and four scenarios it marks `added`, does not appear.

**Considered: compare with today's text and print a warning.** Rejected on that measurement. A
warning does not make the marked word finish-the-archival's.

**Considered: compare with the requirement as it stood before the change**, which OpenSpec cannot do.
The nearest thing the program could read is the statement of the last change archived before it that
states the same identifier. Rejected, and recorded under "Not in this change":

- **It needs an order the program does not derive.** `runbook_release::history` sorts runbook
  snapshots by recorded version. Nothing orders changes' specifications. Building that order is a new
  reading of the archive, not a reuse.
- **The earlier statement equals the accepted text of its day only under two conditions.** Every
  change in between must have stated the requirement whole, and each merge must have taken it as
  stated. `state-a-modified-requirement-whole` and `see-a-modification-the-merge-did-not-take` make
  both checkable from now on. Nothing checked them for the archive that exists.

**Considered: say nothing and exit 1, as OpenSpec does with `Unknown item`.** Rejected. The directory
exists, and "unknown" is false. A refusal that names the reason tells the reader where to look
instead.

`sync` is not changed. It runs on an archived change's path today, and
`see-a-modification-the-merge-did-not-take` left it so. Its report on an archived change is still a
true statement about whether the text is in the accepted specification. The view's output would be
read as what the change modified, which is false for an archived one.

### 9. The two forms, and why neither is merge text

**The human form** leads with what it is, then each requirement it compares, then the list:

```
sync-a-change-into-the-accepted-contract states 3 requirements, and compares 3 of them.
Each text is quoted, as projected, from the document named beside it. {+ +} marks words only the
change states, and [- -] words only the accepted specification holds.

req-a-single-command-exposes-validate-format-and-init, modified
  The command surface names its operations as OpenSpec names them
  compared with htmlspec/specs/toolchain-packaging/spec.html
  statements: 9 of 10 paragraphs held by both, words marked by comparison
    paragraph 3, and paragraph 3 in the accepted specification
      the accepted specification holds:
        The spellings are therefore fixed rather than left to an implementer: validate, … status, instructions, and store …
      the change states:
        The spellings are therefore fixed rather than left to an implementer: validate, … status, {+sync,+} instructions, and store …
  scenarios: 18 of 18 held by both

req-the-command-surface-reports-whether-a-change-reached-the-accepted-contract, added
  The command surface reports whether a change reached the accepted contract
  held by no accepted specification, so nothing is compared

req-the-workflow-instructions-carry-a-stage-for-applying-a-change-s-contract, added
  The workflow instructions carry a stage for applying a change's contract
  held by no accepted specification, so nothing is compared
```

A requirement the view lists rather than compares follows, under `listed, not compared:`, one line
each with its declared state and identifier. This change lists none.

The `…` are this document's. The view prints each paragraph whole, on one line.

- The accepted text comes first and the change's second, the order in which a reader meets them.
- A paragraph held by both is counted and not printed. The one-word case prints two paragraphs of
  the 10 and none of the 18 scenarios.
- The marks are `git diff --word-diff=plain`'s notation, `{+…+}` and `[-…-]`, split over two
  labelled lines. It is ASCII and needs no colour, so it survives a pipe and a log. It is also
  familiar to anyone who has read a word diff. Each mark appears only on the side that holds the
  words.

**The structured form** holds the same facts. Every text is a list of runs, and a run holds the
words only its side holds when `onlyThisSide` is true:

```json
{
  "change": "sync-a-change-into-the-accepted-contract",
  "requirements": [
    {
      "id": "req-a-single-command-exposes-validate-format-and-init",
      "declared": "modified",
      "heading": "The command surface names its operations as OpenSpec names them",
      "comparison": {
        "shown": "differs",
        "acceptedIn": "htmlspec/specs/toolchain-packaging/spec.html",
        "acceptedHeading": "The command surface names its operations as OpenSpec names them",
        "statements": {
          "marks": "comparison",
          "paragraphs": [
            { "kind": "both", "position": 1, "acceptedPosition": 1 },
            { "kind": "both", "position": 2, "acceptedPosition": 2 },
            {
              "kind": "pair", "position": 3, "acceptedPosition": 3,
              "acceptedHolds": [ { "text": "The spellings are therefore fixed … status, instructions, and store …" } ],
              "changeStates": [
                { "text": "The spellings are therefore fixed … status, " },
                { "text": "sync, ", "onlyThisSide": true },
                { "text": "instructions, and store …" }
              ]
            }
          ]
        },
        "scenarios": [ { "kind": "both", "id": "scenario-validating-a-project", "declared": "unchanged" } ]
      }
    },
    {
      "id": "req-the-workflow-instructions-carry-a-stage-for-applying-a-change-s-contract",
      "declared": "added",
      "heading": "The workflow instructions carry a stage for applying a change's contract",
      "comparison": { "shown": "held-by-none" }
    }
  ]
}
```

- `shown` is `differs`, `held-as-stated`, `held-by-none`, or `not-compared`.
- `acceptedHeading` is the accepted requirement's heading name, and `null` only where it has none.
  As first designed it was `null` where the two names were equal, which could not tell an equal
  heading from an accepted requirement with no heading. The human form still shows both headings
  only where they differ.
- A side's runs, joined, are that side's text exactly: a space between two words goes with the word
  before it, and a mark inside a word keeps the word whole. So no text is formed from both sides,
  and a reader can rebuild either side from its runs alone.
- A scenario shown whole carries its heading and entries under `changeStates` or `acceptedHolds`,
  so it is labelled with its side like every other text.
- A paragraph's `kind` is `both`, `pair`, `change`, or `accepted`. A scenario's is the same, or
  `withdrawn`, with `stillHeld`.
- `marks` is `author`, `comparison`, or `unreconciled`.
- Paths are relative to the project's root.

In Rust each of these is a closed enum, and the payload is produced from them by one function. So a
payload value cannot exist that the types do not.

**The contributor accepted this reading on 2026-10-01.** The ban stays on the report an agent works from, and a labelled view that no tool can apply is not merge text.

**Why neither form is merge text.** toolchain-packaging's "The command surface reports whether a
change reached the accepted contract" and "Archival assesses the merge and never performs it" each say
"SHALL NOT propose the text of a merge". What is forbidden is a program deciding what the accepted
specification says next. The view restates two texts that already exist, each under the name of the
document that holds it, and states no third:

- no text is labelled as a result, a merge, a replacement, or something to apply;
- no text is formed from both sides;
- the human form has no `@@` hunk header, no `---` or `+++` file header, and no line whose first
  character marks the whole line as added or removed. So neither `patch` nor `git apply` can take it,
  as they can take OpenSpec's hunk.

`see-a-modification-the-merge-did-not-take`'s design rejected carrying both texts in `sync`'s payload,
calling them "the text of a merge", and left a diff view to this change. The two positions agree once
the setting is named. In a worklist that an agent executes, a text reads as the text to write. In a
view with no worklist, labelled by source, it reads as a quotation. The `sync` instructions do not
point at the view, for that reason (decision 11).

**Considered: a unified diff, as OpenSpec prints and as its JSON carries in `diff`.** Rejected. It is
the edit in the form a tool applies, which is the one thing the view must not be.

### 10. Refusals, exit codes, `--store`, and the payload

The view refuses as `sync` does, in `sync`'s order, and for its reasons:

1. a store that cannot be resolved;
2. a change that cannot be resolved, including a runbook, with a new `Refusal::View` whose reason says
   a runbook's specification states what the runbook requires of a run, so no accepted specification
   holds its requirements;
3. an archived change (decision 8);
4. no specification artifact;
5. an artifact of another kind;
6. a specification that does not validate.

Steps 1, 2, and 4 to 6 are taken out of `sync` into two functions, one that resolves and one that
loads, and both commands call them. The view's archived check sits between them. `sync`'s messages
and outcomes are unchanged, and its existing tests hold that. `sync`'s fifth refusal, a template's
example content, is not shared. `sync` refuses there because its report would direct an agent to merge
`req-example`. The view directs nothing, and shows an example requirement as it shows any added one:
held by none, until something merges it.

Exit codes follow toolchain-packaging's "Exit codes distinguish outcomes":

| Situation | Outcome | Code |
| --- | --- | --- |
| the view is shown, whatever it shows | `Success` | 0 |
| the specification does not validate | `DocumentViolation` | 1 |
| every other refusal, `--diff` missing included | `OperationalFailure` | 2 |

With `--json`, standard output holds the payload and nothing else, and every refusal goes to standard
error, as for `sync`. The view never prompts. With `--store`, the project is the registered one, and
the change and the accepted specifications are both read from it, through `store::project_for` and
`resolve_change_in`.

A test holds the view to reading: `the_view_writes_nothing` hashes every file under the scratch
project before and after each outcome, and a source check like `the_archive_check_reads_only_the_specification`
asserts that `show.rs` calls no `fs::write`, `create_dir`, `rename`, or `remove`, and never names
`proposal.html` or `design.html`.

### 11. The instructions

- **`verify.md`**, in "1. Read the change, then read the implementation": where the change already has
  a `specification.html` that marks a requirement `modified`, read each one beside the accepted text
  with `htmlspec show <change> --diff`. A modification the design did not decide, or a decided one the
  view does not show, is a divergence to classify like any other.
- **`archive.md`**, at the end of step 2, after the paragraph `mark-the-words-a-modification-changed`
  adds on marking words: before the merge, read every requirement you marked `modified` with
  `htmlspec show <change-name> --diff`. Each difference it shows should be one this change made. A
  difference this change did not make is another change's edit that reached the accepted
  specification after this change was written. Merging your restatement would revert it, so restate
  that paragraph or scenario from the accepted text, and mark your own words again.
  `see-a-modification-the-merge-did-not-take`'s first risk is exactly this case, and this is where it
  can be caught before the merge.

`sync.md` is not edited. The merge reads the change's specification for meaning and works through
`sync`'s own worklist, and a side-by-side view invites the wholesale copy `sync.md` forbids. `new.md`,
`continue.md`, `ff.md`, `apply.md`, and `explore.md` write no specification. `migrate.md` is
`migrate-into-the-extended-vocabulary`'s. Task 1.4 checks each by reading it.

## Risks / Trade-offs

- **[`see-a-modification-the-merge-did-not-take` or `mark-the-words-a-modification-changed` is built
  differently from its design.]** → Task 1.1 reads both as built and corrects decisions 3, 5, and 6
  before any code. The parity test in decision 3 then holds the view to the comparison as built.
- **[The half-words threshold pairs the wrong paragraphs, or fails to pair the right ones.]** → Both
  texts are shown in full either way. Only which words are marked, and whether they sit in one pair
  or two whole paragraphs, changes. Task 9.1 asks for confirmation, and a mutation test pins it.
- **[Presentational markup is invisible.]** → A change that only alters emphasis, a code span, or a link
  shows as held by both. The comparison has the same limit, and the header says the texts are quoted
  as projected. A backtick the markdown source carried is not in the projected text (evidence,
  section 6).
- **[Whitespace inside a code span is normalized away.]** → As for the comparison.
- **[A requirement with many edits makes a long view.]** → Paragraphs and scenarios held by both are
  counted and not printed, so the view grows with the edit and not with the requirement.
- **[An identifier held by two accepted specifications.]** → The view compares with the one the
  comparison chooses, and names the file, so the reader sees which.
- **[An accepted specification that does not validate.]** → It is read without validation, as `sync`
  reads it, and the view names the file. A fault there shows up as a difference.
- **[Sub-word marks.]** → An author can mark part of a word. The view shows the marks as written, and
  checks them against the accepted text as normalized text, so they are shown only where they are
  right.
- **[In this project's workflow the view applies for a short time.]** → A specification is usually
  written at archival step 2 and merged at step 5. That window is when a restatement is checked
  against the accepted text, and it is where `archive.md` sends the agent. After the merge the view
  shows "held as stated", which is true.
- **[The staged migration is not served.]** → All 24 staged changes are archived (evidence, section 4).
  Their record is git history and the prose of their summaries. The comparison of archived changes
  is under "Not in this change".
- **[An agent reads the view as the merge to perform.]** → The labels, the absence of a patch form,
  and `sync.md`'s silence about the view (decision 9).
- **[The marks reader and the projection drift.]** → The property test in decision 5 runs over every
  mapping fixture and every staged document.

## Migration Plan

No document changes meaning, and none stops validating. The marks reader adds nothing to the model or
the schema, so under the bump table in `crates/htmlspec-schema/CHANGELOG.md` the contract version stays
where the changes before this one leave it. What changes is the command surface: one subcommand and one
flag, which are additive. `sync` and archival give the same results and payloads as before.

This change follows `mark-the-words-a-modification-changed`, whose markup it reads, and it relies on
`see-a-modification-the-merge-did-not-take`, whose comparison it calls. Both must be archived before it
lands. It modifies toolchain-packaging's "The command surface names its operations as OpenSpec names
them" after `resolve-a-reference-across-its-change` does, so task 1.0 re-bases that block.

The release is expected to be `0.24.0`, the minor increment after `mark-the-words-a-modification-changed`'s
`0.23.0`, because the command surface gains an operation. The contributor confirms the increment at the
release task.

Rollback is a revert. It touches no document and no stored data, and no other command depends on the
view.
