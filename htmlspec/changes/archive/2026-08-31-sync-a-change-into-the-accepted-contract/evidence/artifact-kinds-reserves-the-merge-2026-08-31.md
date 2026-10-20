# The merge is reserved for an agent, read from the source

Task 1.4, recorded 2026-08-31. The task asks for this to be confirmed by reading
`artifact-kinds` rather than by citing archival's code comment that cites it, because a design
whose central constraint comes from a summary has derived it from a summary.

## What the requirement actually says

`artifact-kinds`, `Every synthesis and every contract update is agent-driven`:

> Four operations move content between artifacts, and each SHALL be performed by an agent
> following stated rules and guides rather than by a program that parses one artifact and emits
> another:
>
> 1. Creating a **design**, using the proposal.
> 2. Creating **tasks**, using the proposal and the design.
> 3. Producing the change's **specification**, from the completed change.
> 4. **Updating the project's accepted specifications**, from that specification.
>
> [...] The fourth is a **merge**, and it is the only one. It applies a change's contract into
> the project's accepted specifications, which already exist and hold content this change did
> not touch.

**The operation this change is about is item 4, and the requirement names it the only merge in
the model.**

## The division the requirement already draws

The paragraph that settles the design:

> The project SHALL supply the rules and guides that direct these merges, as instructions rather
> than as code. What the project SHALL implement in code is the **checking**: validation reads
> the merged artifact and reports what is wrong with it. That division is the whole arrangement —
> judgment where judgment is needed, mechanical checking where a rule is exact.

This is stronger than the proposal claimed. The proposal argued that sync *should* divide into
an agent's merge and a program's check. The accepted contract already requires that division for
this exact operation: **rules as instructions, checking as code.** Sync's shape is not a choice
this change makes; it is the shape the contract already mandates, and the change's contribution
is that the checking half finally exists as a command and the instructions half finally exists
as a stage.

## The constraint this puts on the implementation

> The merge SHALL apply partial updates rather than replacing wholesale. Adding one scenario to
> an existing requirement SHALL be expressible without restating the requirement, and content
> the merge does not mention SHALL be preserved.

A programmatic merger would have to satisfy this, and could not: deciding which existing
requirement a new scenario belongs under is the reading the requirement says a parser cannot do.
This is the direct argument against building one, and it comes from the contract rather than
from taste.

## What this fixes in the proposal's reasoning

The proposal cites archival's comment — "*`artifact-kinds` reserves it for an agent*" — which is
accurate but is a paraphrase inside a code comment. The requirement above is the source, it is
more specific than the paraphrase, and it makes one of the proposal's arguments unnecessary:
there is no need to argue that sync must not merge, because the contract already says so about
this operation by number.
