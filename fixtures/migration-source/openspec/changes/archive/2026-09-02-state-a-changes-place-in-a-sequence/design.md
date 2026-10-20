# Where an ordering is written and what reads it

## The declaration goes in the change that follows

One relation, written by the change that has a predecessor: *this change follows that one.*

The alternative — a change naming the changes that follow it — was rejected on two grounds. It
puts the fact in the change least able to maintain it: the contract is written first, before its
successors exist, so their names cannot be known. And it makes the fact grow after the file is
finished, which is how a list of successors goes stale silently. A successor knows its own
predecessor at the moment it is created, which is the only moment the fact is cheap to record.

## In the change's declaration, not in a document

The ordering goes in `.openspec.yaml`, beside `schema` and `created`.

The alternative was a unit inside `proposal.html` carrying a `data-refs` attribute, which would
reuse the reference machinery already there. Rejected, and the reason is a fact about the
projection rather than a preference: `refs/unresolved` resolves a reference against the
identifiers of *the same projection*, so a reference to another change resolves to nothing and
the rule reports it as an error. Making it not an error would mean teaching the reference rules
that some targets live outside the document, which weakens a rule that is currently exact.

There is a stronger reason. A reference addresses a *unit* — a requirement, a task, a decision.
A change is not a unit. It is a directory. `artifact-kinds` is explicit that the four kinds carry
units and that a change is the thing that holds them, so making a change addressable as a
reference target would put two different things in one namespace.

The declaration is already the file that carries facts about the change as a whole rather than
about its contents. `schema` says which workflow it follows. `created` says when. `follows` says
what came before it. All three are properties of the directory.

## Resolving a predecessor across the archive boundary

This is the part that looks trivial and is not.

A change is named `contract-the-guidance-solver-comparison` while active. Archival moves it to
`archive/2026-09-02-contract-the-guidance-solver-comparison`, because `archived_name` prepends
the date. So a successor that recorded its predecessor's name cannot find that predecessor by
equality once it has been archived — which is precisely the state the successor is waiting for.

The resolution therefore accepts three outcomes, and the distinction between them is the whole
report:

| The named change is | Meaning | What archival says |
| --- | --- | --- |
| in the archive | the predecessor is done | nothing |
| active | the predecessor is not done | a finding |
| nowhere | the name is wrong, or the change was renamed | a different finding |

The third case must be reported separately from the second. A successor whose predecessor was
renamed and one whose predecessor is merely unfinished are in different situations, and a report
that conflated them would send a contributor looking for work that was already done under
another name.

Matching an archived predecessor means comparing against the archived directory's name with a
leading `YYYY-MM-DD-` removed, using the same date-prefix test `archived_name` already uses. This
is stated in the contract rather than left to an implementer, because the obvious implementation
— string equality — is wrong in the one case the whole feature exists to detect, and it is wrong
silently.

**Note what this does not do.** `project-bootstrap` forbids deriving the last archived version
from a directory name, and this is not that. The version is a *fact about the work* that the
project records in a `VERSION` file. A change's name is its identity, and matching a name against
a name is the only thing that can resolve a named predecessor at all. The prohibition is against
deriving one kind of fact from another, not against reading a name as a name.

## Why archival reports and does not refuse

An ordering violation joins the two findings archival already has: an unfinished task and a
requirement that never reached the accepted specifications. Both are legitimate states that a
contributor is the authority on, and this is a third.

A gate would be wrong here on a case that will occur. A sequence gets abandoned. Its first change
is dropped and its second is completed and archived on its own merits, and a gate would refuse
that archival with no way to proceed except editing a file to lie about the history. Archival
already solved this shape: report the finding, ask, and let `--yes` answer in advance for an
agent. The same machinery takes a third finding with no new mechanism.

## The enumeration carries it too

The listing is where a workflow selects what to act on, and 0.8.0 established that it carries
what selection needs. A change waiting on another is exactly that: the strongest reason not to
select a change is that its predecessor is unfinished.

So the payload carries each change's predecessor and whether that predecessor is resolved. The
human report names it in a way that does not disturb the columns the fraction occupies, because
that alignment was a stated requirement one change ago.

**The order does not change.** The accepted contract requires that the enumeration's order not
depend on what it carries, and a dependency ordering is a thing it carries. Sorting the listing
topologically would also be unstable in a way name order is not: a cycle has no topological order
at all, and a listing that reordered itself when a `follows` line was edited could not be
selected from twice. Name order, with the fact reported beside each change.

## A cycle is reported and never resolved

Two changes that each follow the other are a contradiction in the recorded facts. The program
reports it and stops there.

This is not a graph algorithm. With one relation per change, a cycle is found by walking
predecessors from a starting change and noticing a name twice, and the walk is bounded by the
number of changes. There is no transitive closure to compute and no ambiguity about which edge to
blame, because a contributor who wrote both `follows` lines can see both.

## Rejected: inferring the sequence

Three inferences were considered and all three are rejected on the same ground — each would make
the program assert an ordering nobody stated.

**From names.** `artifact-kinds` says a change is renamed by moving its directory "because nothing
depends on the first name". An ordering inferred from names would make that sentence false.

**From creation dates.** `created` records when a change was written, not what it depends on. The
contract in the motivating example was written first *and* a project could equally write a
successor first and the contract second, on discovering the successor needed one.

**From requirement identifiers.** A change stating a requirement that another change also states
is a real signal, and it is the signal `sync` already reads for a different question. Two changes
touching one requirement are not ordered by that fact — they might be, or they might conflict, and
deciding which is judgment. `artifact-kinds` reserves that judgment for an agent.

## Rejected: a general dependency graph

Many-to-many edges, edge kinds, transitive resolution, and parallel branches were considered and
rejected as building on no evidence. The observed problem is one project stating a linear
sequence in prose. One relation covers it. If a project later needs a change with two independent
predecessors, that is a change to make then, with the case in hand.
