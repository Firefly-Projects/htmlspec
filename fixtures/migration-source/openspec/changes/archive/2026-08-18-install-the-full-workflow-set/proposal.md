## Why

`htmlspec init` installs one workflow. A contributor who bootstraps a project, opens their agent, and types `/htmlspec` sees a single entry:

```
/htmlspec:archive    Archive an html-spec change — produce its specification artifact…
```

That is the last step of the lifecycle and the only one offered. Everything before it — deciding what to build, writing the proposal, carrying it into a design, working the tasks, checking the result against what was promised — is unattributed. A contributor either reads `AGENTS.md` and drives the CLI by hand, or asks their agent to improvise a workflow the project has opinions about but never states.

The comparison is direct, because the arrangement being imitated is installed in this very repository. OpenSpec's `init` installs ten:

```
/opsx:ff          Create a change and generate all artifacts needed for implementation in one go
/opsx:new         Start a new change using the experimental artifact workflow
/opsx:sync        Sync delta specs from a change to main specs
/opsx:apply       Implement tasks from an OpenSpec change
/opsx:verify      Verify implementation matches change artifacts before archiving
/opsx:archive     Archive a completed change in the experimental workflow
/opsx:explore     Enter explore mode - think through ideas, investigate problems, clarify requirements
/opsx:onboard     Guided onboarding - walk through a complete OpenSpec workflow cycle with narration
/opsx:continue    Continue working on a change - create the next artifact
/opsx:bulk-archive  Archive multiple completed changes at once
```

The gap is not that htmlspec has fewer commands. It is that htmlspec has *the same lifecycle* and describes one seventh of it. `htmlspec new change` already writes a proposal, a design, and a tasks document. The artifact-kinds change established what each of those four kinds carries and what it must not, and the accepted contract states those obligations at length. None of that reaches an agent, because the only instruction file installed is the one about archival.

**The rule that produced this is recorded, and it is narrower than the behavior it produced.** `crates/htmlspec-assets/src/instructions.rs:8-11` says:

> One workflow ships here — `archive` — because archival is the one operation the program already gates. The rest of the lifecycle is named as a non-goal: installing an instruction file for a command that does nothing would tell an agent to run something that is not there.

The stated hazard is real: a skill that says to run `htmlspec propose` would send an agent to a clap error. But the conclusion drawn from it is wrong, and `archive` is the counterexample sitting in the same table. Archival ships **no** extractor command — the instruction file opens by saying so, in bold, and then spends 196 lines telling an agent how to do the work with its own judgment. `htmlspec archive` only checks the result. So a workflow does not need a command that performs it. It needs to not *name* a command that does not exist.

That distinction is the whole change. Six more workflows can be written under the rule as stated, provided each one names only commands the program actually has.

## What Changes

- **`init` installs seven workflows instead of one.** The lifecycle, in the order a change moves through it:

  | Workflow | What it is | CLI behind it |
  | --- | --- | --- |
  | `explore` | Think through an idea before committing to a change. A stance, not a sequence. | none — reads only |
  | `new` | Start a change: write its proposal, then its design, then its tasks. | `htmlspec new change` |
  | `continue` | Produce the next artifact the change is missing. | `htmlspec new document`, `validate` |
  | `ff` | Create a change and carry it through every artifact in one pass. | `htmlspec new change`, `validate` |
  | `apply` | Work the tasks, settling each task's state as it goes. | `htmlspec validate`, `format` |
  | `verify` | Check the implementation against the change's own artifacts before archival. | `htmlspec validate` |
  | `archive` | Produce the specification, advance the version, merge, archive. | `htmlspec archive` |

  `archive` is unchanged. Its instruction file is the model the six new ones follow.

- **Every workflow is agent-driven, and each says so.** Four of the seven have no command that performs them, which is the same footing `archive` has always been on. Each instruction file names only commands the program has, and a test holds that: the existing `no_body_names_a_withdrawn_command` test generalizes into one that extracts every `htmlspec …` invocation from every body and checks it against the CLI's own command tree. That is a stronger guarantee than the rule the code comment currently states by hand, and it is what makes adding a seventh workflow safe.

- **The closing message names the workflow to start with, not the one to finish with.** `init` currently ends by printing the archival invocation, because archival was the only thing installed. With a lifecycle installed, the command a contributor needs on a first run is the one that begins it. The message names the entry point per tool spelling, and the existing per-spelling grouping is kept.

- **The instruction bodies draw on the accepted contract rather than restating it loosely.** The four artifact kinds, what each carries, and what each must not carry are already stated in `openspec/specs/artifact-kinds`. The `new`, `continue`, and `ff` bodies are written against that, so an agent producing a proposal is told the same things the validator will check — including the ones that are errors rather than advice, like a requirement in a proposal or a task in a specification.

## Non-Goals

- **`onboard` and `bulk-archive` are not included.** `onboard` narrates a full cycle, and narrating a lifecycle whose bodies are being written for the first time in this change would pin wording that has not been exercised yet. `bulk-archive` answers a pressure that arrives with many concurrent changes; this project has archived four, one at a time. Both remain table entries a later change can add, which is the shape the tool table was built for.

- **`sync` does not become a standalone workflow.** Applying a change's contract into the accepted specifications is step 5 of `archive`, and the archival body already carries the partial-update guidance that makes it safe. Splitting it out would put that guidance in two places, and the one that drifted would be the one someone read.

- **No new CLI commands.** Nothing in this change adds a subcommand, a flag, or an output mode. The seven workflows are instruction files over the command surface that exists.

- **The tool table does not change.** Five targets, the same detection, the same three deliveries. Seven workflows times five tools is a loop count, not a new case: `write_skills` and `write_commands` already iterate `WORKFLOWS`.
