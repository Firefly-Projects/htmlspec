## Why

A runbook that runs only on the machine it was written on is a note to self.

The two changes before this one produce a procedure that lives in a project and records its runs
there. Both facts depend on the project being present. **Move the runbook and every one of its
assumptions goes unstated:** the tools it calls, the versions of those tools, the skills the agent
needs installed, and the state of the working tree it starts from. Nothing in the runbook says what
those are, so the failure arrives partway through — at step 6, with the oven already hot.

This project has measured that failure on itself. The release procedure's own record says so:

> `init` also bootstrapped a fresh `htmlspec/` project directory and a `spec.nix` at version 0.0.0
> in the repository root, which then failed `self-assets` with 9 duplicate-asset errors — a second
> asset store beside the project's own.

That is a precondition discovered by violating it. The step assumed a state — "you are not already
inside a project" — that nothing recorded, so the assumption was found by breaking it and then
written into an archived `tasks.md` that binds nothing.

**The shape of the answer is a contract rather than a bundle.** That is the contributor's framing,
and it is the right one:

> imported skills and install procedures are similarly contracts to do in order that the runbooks
> are standalone and deployable

A bundle carries copies and the copies drift. A contract states what is needed and lets the tool
report what is absent. This project already runs a check whose entire subject is that drift:
`scripts/check-installed` regenerates 45 instruction files into a temporary directory and compares
them, because a vendored copy of generated content goes stale the moment its source moves. Vendoring
skills into a runbook rebuilds exactly that problem inside the runbook.

## What Changes

- **A runbook declares what it needs of a machine**, as requirement units in its specification
  beside the log contract `log-a-runbook-run` establishes. Three kinds of need, all stated the same
  way: a tool at a version, a skill that must be installed, and a state the procedure assumes.

- **A preflight operation checks those requirements and reports what is missing, before the run
  starts.** The ordering is the whole value. A missing `python3` discovered at step 1 costs
  nothing; discovered at step 6 it costs the work of the first five steps. This follows the safety
  pattern the project's own reporting already uses — state the condition first, then the
  consequence.

- **Preflight refuses the run rather than warning about it.** This is the one place in the runbook
  design where a gate is correct, and the reason is that the alternative was measured: a warning
  about a missing dependency is a warning an operator reads after the failure it predicted.
  `log-a-runbook-run` argues the opposite for run conformance, and the difference is that a run in
  progress legitimately has unverified requirements while a run started on a machine missing its
  tools is legitimately nothing.

- **A per-runbook version earns itself here, and only here.** `establish-a-runbook` argues that a
  runbook needs no version of its own because a specification has none — its history is derived
  from the archived changes that touched it. **That derivation requires the archive.** On another
  machine the project is absent, so there is nothing to derive from, and the runbook that arrived
  cannot say which release it came from. A version becomes the only way to answer that.

  The honest cost, stated rather than discovered later: `scripts/check-release` grows a fourth
  question, and every runbook then owes a distinct version with a changelog heading. That is a real
  obligation on every future edit to every runbook, and it buys one thing — a runbook that can name
  itself when the project is not there.

- **Nothing is vendored except the runbook's own documents and its asset store.** The assets are
  already content-addressed and shared, which is the mechanism `establish-a-runbook` inherits. A
  skill is not vendored, for `check-installed`'s reason: a generated file copied into a second
  location is a file that disagrees with its source on the next release.

**Not in scope:**

- **Installing what is missing.** Reporting that `cargo` is absent is exact. Choosing how to
  install it on an unknown machine — which package manager, which channel, whether the operator
  even wants it — is judgment, and `artifact-kinds` reserves judgment for an agent. Preflight
  reports; it does not remediate.
- **Verifying a tool behaves correctly.** Preflight answers "is it here, at a version this runbook
  states" and stops. Whether `cargo` works is what the run finds out.
- **Transport.** How a runbook reaches the other machine — `git`, an archive, a copy — is not this
  change's business. `htmlspec store` already registers a project reachable from anywhere and is
  the nearest existing mechanism.

## Capabilities

**New Capabilities:**

- `runbook-portability` — what a runbook declares about the machine it needs, when that is checked,
  and what a runbook carries versus what it requires. The last of those is the boundary this
  capability exists to hold: a bundle that carries everything and a contract that states everything
  are different artifacts, and the choice needs a stated home.

**Modified Capabilities:**

- `runbook-authority` — a runbook's specification gains its machine requirements, alongside the log
  contract from `log-a-runbook-run`.
- `toolchain-packaging` — it "defines the command-line surface contributors and agents use", so the
  preflight operation joins it there.
- `project-bootstrap` — its requirement that archival compares the declared version against the
  last archived one is where a per-runbook version has to be reconciled. A runbook version that no
  gate compares is the "accuracy without authority" failure `establish-a-runbook` argues against,
  so introducing one obliges this change to say what compares it.

## Impact

- `crates/htmlspec-cli/src/main.rs` and `commands.rs` — the preflight subcommand and its report.
- `scripts/check-release` — the fourth question. Its docstring currently states three, and the file
  explains why each one asks about the archive rather than the working change.
- `openspec/specs/toolchain-packaging/spec.md` — the new operation in the CLI surface.
- `openspec/specs/project-bootstrap/spec.md` — the version comparison.
- `fixtures/conformance/` — a runbook whose machine requirements are satisfiable and one whose are
  not.
- No change to the schema, the units, or the artifact kinds. A machine requirement is a requirement
  unit, and preflight reads the model that already exists.

## The counter-argument

**This is what a package manager does, and it does it better.** Declaring dependencies with
versions, checking them against a machine, and refusing to proceed when they are absent is solved
work. `nix`, which this project already uses to pin its own toolchain, does exactly this and does
not need a runbook to reimplement it.

That is largely correct, and it bounds this change rather than defeating it. Where a dependency is
expressible to a package manager, **the runbook's requirement should name the manifest rather than
restate the dependency** — a requirement that says "the toolchain `flake.nix` pins" is checkable and
does not fork the pin. This change must not grow a second dependency language beside `flake.nix`,
and that constraint belongs in its design.

What no package manager holds is the other two kinds of need. A skill installed for an agent is not
a system package. A state assumption — "you are not standing inside an existing project" — is not a
dependency at all, and it is the one this project got wrong and wrote into an archived tasks
document. Those are the needs preflight exists for, and the dependency check is the part it should
delegate as far as it can.

## Findings left open

- **Whether a runbook carried alone is still a runbook.** Its run records live in the project it
  came from. A runbook that arrives without them has no history, and whether that matters depends on
  whether run records are audit trail or working notes — the question `establish-a-runbook` already
  records as open.
- **What a per-runbook version is compared against.** Named under Capabilities. Archival compares
  the project's declared version to the last archived one, and no equivalent exists for a runbook
  arriving on a machine with no archive. An uncompared version is the failure mode this design
  otherwise avoids.
- **The input gap remains the nearest unclaimed neighbour.** A tool a runbook requires and an
  ingredient a runbook consumes are close enough that settling one may settle the other, and
  `establish-a-runbook` records inputs as open for that reason. This change deliberately does not
  claim them: a machine requirement is checked once before a run, and 1 cup of chocolate chips is
  consumed during it.
