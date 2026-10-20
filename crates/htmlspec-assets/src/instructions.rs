//! The workflow instructions `htmlspec init` installs.
//!
//! Compiled into the binary rather than read from disk, for the same reason the vendored
//! assets are: the standalone-artifact requirement says the packaged program runs with no
//! source checkout, so a runtime file read is not available. `build.rs` records that
//! reasoning for `include_bytes!`, and this is the same constraint one level up.
//!
//! The set covers the lifecycle from an idea to an archived change, and a workflow ships
//! whether or not a command performs it. Four of the seven have none. `archive` is why
//! that is the ordinary case rather than an exception: it ships no extractor, says so in
//! its opening line, and leaves the judgment to the agent while the program checks the
//! result.
//!
//! The constraint an earlier release stated as "one workflow, because archival is the one
//! operation the program gates" was aimed at a real hazard and drawn too wide. The hazard
//! is an instruction file **naming** a command that is not there, which sends an agent to
//! a parse error. It is not a workflow lacking one. `unresolved` in the command-line
//! crate's `invocations` module holds the narrower rule against clap's own command tree,
//! so a body naming an absent command fails the build. That check lives there because
//! that crate owns the command definition and this one must not depend on it.
//!
//! The two properties left here are the ones local to a body: it has length, and it
//! carries no frontmatter of its own. The body is tool-independent and the frontmatter is
//! generated per tool, which is why the source file has none. A skill for Claude Code and
//! a prompt for Pi differ in their header and not in what they say to do.

/// A workflow an agent can be instructed to perform.
#[derive(Clone, Copy, Debug)]
pub struct Workflow {
    /// The workflow's name, which is also its command name and its skill directory's
    /// suffix.
    pub name: &'static str,
    /// The skill directory this installs as, beneath a tool's `skills/`.
    pub directory: &'static str,
    /// A one-line summary, used as the description in generated frontmatter.
    pub summary: &'static str,
    /// What tools the instructions permit, as Claude Code spells it.
    pub allowed_tools: &'static str,
    /// Whether this workflow begins the lifecycle.
    ///
    /// The closing message a bootstrap prints names this one, because a bootstrap is a
    /// first run and the stage a contributor reaches last describes work they have not
    /// started.
    ///
    /// A flag rather than "whichever entry comes first", because those are two facts that
    /// happen to agree today. Order is for reading. The entry point is what a contributor
    /// types first. A later change adding a row above `new` must not silently repoint the
    /// message.
    pub entry_point: bool,
    /// What a contributor may type after the invocation, as a tool's `argument-hint`.
    ///
    /// A property of the workflow rather than one string shared by all of them, because
    /// what a workflow takes differs: `new` begins from a description of work that does not
    /// exist, `apply` acts on a change already present. A shared hint would describe
    /// neither accurately, and the contract requires the workflow set to be data so that
    /// covering another stage is an entry here rather than a change to a writer.
    ///
    /// **Optional in form — `[thing]`, never `<thing>`.** A required hint makes a tool
    /// reject a bare invocation, and a bare invocation must keep working: where a
    /// contributor has described nothing, asking is the correct behavior and the
    /// instructions say so. The hint tells a contributor what to type; it does not gate
    /// them on typing it.
    pub argument_hint: &'static str,
    /// The instruction body, with no frontmatter.
    pub body: &'static str,
}

/// The token a tool substitutes a contributor's arguments into.
///
/// One spelling for every tool, because a body is tool-independent — see the module
/// comment: a skill and a prompt differ in their header and not in what they say to do. A
/// per-tool placeholder would put a tool-specific string into shared prose.
///
/// `$ARGUMENTS` is what the tools this program installs for actually read, and it captures
/// the whole remainder as one string rather than splitting it into fields. That is the
/// right shape here: a workflow takes prose a contributor typed, not positional arguments.
pub const PLACEHOLDER: &str = "$ARGUMENTS";

/// Every workflow whose instructions this program installs.
///
/// **In lifecycle order**, and the order carries weight. A shared instructions file
/// receives one section per workflow in this order, so a contributor reads the region as
/// a document. Alphabetical order would open with `apply` and close with `verify`, which
/// reads as a list of unrelated commands.
pub const WORKFLOWS: &[Workflow] = &[
    Workflow {
        name: "migrate",
        directory: "htmlspec-migrate-project",
        summary: "Bring an existing OpenSpec project into html-spec — survey what it holds, \
                  read what each document does, plan every entity's destination, then stage and \
                  promote the whole project. Use once, on a project that is not html-spec yet.",
        allowed_tools: "Bash(htmlspec:*), Read, Write, Edit, Glob, Grep",
        entry_point: false,
        argument_hint: "[the OpenSpec project to migrate, or nothing to infer it]",
        body: include_str!("../instructions/migrate.md"),
    },
    Workflow {
        name: "explore",
        directory: "htmlspec-explore",
        summary: "Think through an idea before it becomes a change — investigate the \
                  problem, read the code, and clarify what the work is for. Use before a \
                  change exists, when the shape of the problem is not yet settled.",
        // No write tools. The stance is the whole point: an agent that can edit will edit,
        // and the failure this workflow prevents is work done before anyone decided what
        // it was for.
        allowed_tools: "Read, Glob, Grep",
        entry_point: false,
        argument_hint: "[the idea or question to think through]",
        body: include_str!("../instructions/explore.md"),
    },
    Workflow {
        name: "new",
        directory: "htmlspec-new-change",
        summary: "Start an html-spec change — create it, then write its proposal, its \
                  design, and its tasks. Use when the work is decided and the change does \
                  not exist yet.",
        allowed_tools: "Bash(htmlspec:*), Read, Write, Edit, Glob, Grep",
        entry_point: true,
        argument_hint: "[a description of the work]",
        body: include_str!("../instructions/new.md"),
    },
    Workflow {
        name: "continue",
        directory: "htmlspec-continue-change",
        summary: "Produce the next artifact an html-spec change is missing. Use on a \
                  change that exists and is not yet ready to implement.",
        allowed_tools: "Bash(htmlspec:*), Read, Write, Edit, Glob, Grep",
        entry_point: false,
        argument_hint: "[the change to continue, or nothing to infer it]",
        body: include_str!("../instructions/continue.md"),
    },
    Workflow {
        name: "ff",
        directory: "htmlspec-ff-change",
        summary: "Create an html-spec change and carry it through every artifact in one \
                  pass, checking each before the next. Use when the work is well \
                  understood and the shape of the change is settled.",
        allowed_tools: "Bash(htmlspec:*), Read, Write, Edit, Glob, Grep",
        entry_point: false,
        argument_hint: "[a description of the work]",
        body: include_str!("../instructions/ff.md"),
    },
    Workflow {
        name: "apply",
        directory: "htmlspec-apply-change",
        summary: "Implement an html-spec change — work its tasks, settle each task's state \
                  as the work lands, and record what the implementation revealed. Use when \
                  a change has its artifacts and the work is ready to start.",
        allowed_tools: "Bash(htmlspec:*), Read, Write, Edit, Glob, Grep",
        entry_point: false,
        argument_hint: "[the change to implement, or nothing to infer it]",
        body: include_str!("../instructions/apply.md"),
    },
    Workflow {
        name: "verify",
        directory: "htmlspec-verify-change",
        summary: "Check an implemented html-spec change against its own artifacts before \
                  archival — what was built against what the tasks claim, and the code \
                  against the design's decisions. Use when the tasks are complete and \
                  archival has not started.",
        allowed_tools: "Bash(htmlspec:*), Read, Glob, Grep",
        entry_point: false,
        argument_hint: "[the change to check, or nothing to infer it]",
        body: include_str!("../instructions/verify.md"),
    },
    Workflow {
        name: "sync",
        directory: "htmlspec-sync-change",
        summary: "Apply an html-spec change's contract to the project's accepted \
                  specifications — merge the requirements it states into the documents the \
                  project holds. Use when a change's work is done and its contract should \
                  reach the project before, or independently of, archiving it.",
        allowed_tools: "Bash(htmlspec:*), Read, Write, Edit, Glob, Grep",
        entry_point: false,
        argument_hint: "[the change to sync, or nothing to infer it]",
        body: include_str!("../instructions/sync.md"),
    },
    Workflow {
        name: "archive",
        directory: "htmlspec-archive-change",
        summary: "Archive an html-spec change — produce its specification artifact from the \
                  completed change, advance the project version, validate, apply its contract \
                  changes to the project's accepted specifications, and archive the directory \
                  intact. Use when a change's tasks are complete and it is ready to be archived.",
        allowed_tools: "Bash(htmlspec:*), Read, Write, Edit, Glob, Grep",
        entry_point: false,
        argument_hint: "[the change to archive, or nothing to infer it]",
        body: include_str!("../instructions/archive.md"),
    },
];

/// The workflow of this name, if there is one.
pub fn workflow_for(name: &str) -> Option<&'static Workflow> {
    WORKFLOWS.iter().find(|workflow| workflow.name == name)
}

/// The workflow that begins the lifecycle.
///
/// Returns an `Option` because the table is data and this reads it. `exactly_one_entry_point`
/// pins the invariant that makes the value present.
pub fn entry_point() -> Option<&'static Workflow> {
    WORKFLOWS.iter().find(|workflow| workflow.entry_point)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    /// The heading under which a body tells an agent what to report.
    ///
    /// A report is not an artifact — nothing writes it to a file, validates it, or archives it — so
    /// the program cannot check what a report says. It can check that guidance for one exists, and
    /// this constant is what makes that checkable rather than a matter of review.
    const REPORT_SECTION: &str = "## What to report";

    /// The obligation that a report names what its references point at.
    ///
    /// Held here so the wording is one string rather than seven. The prose itself is repeated in
    /// each body, because a body must be self-contained — an agent reads one file and no other —
    /// and a test pins the repetition against the drift that produced this rule's need.
    const POINTER_RULE: &str = "Name what your references point at";

    /// The obligation that an assertion about the project's contract names its requirement.
    ///
    /// Held here for the reason [`POINTER_RULE`] is: the prose is repeated in every body, because
    /// an agent reads one file and no other, and one constant is what keeps eight copies from
    /// becoming eight paraphrases. `contract-authority` requires the wording to be identical and
    /// requires the identity to be checked mechanically, so this constant is that check's subject.
    const CONTRACT_RULE: &str = "Name the requirement, or name the silence";

    /// The sentence that bounds [`CONTRACT_RULE`], which is the half a body can lose.
    ///
    /// The obligation without its test fires on every sentence an agent writes, and an agent that
    /// meets a rule firing everywhere learns to ignore it. That is the failure the pointer rule's
    /// own deletion test was added to prevent, so this pins the bound as well as the rule — the
    /// same shape as `DELETION_TEST` in the pointer rule's test.
    const SILENCE_ROUTES: &str = "say the contract is silent on the point";

    /// A workflow's instructions must actually say something. An empty body would install
    /// a skill that instructs an agent to do nothing, which is worse than installing none.
    #[test]
    fn every_workflow_carries_a_body_and_a_summary() {
        for workflow in WORKFLOWS {
            assert!(
                workflow.body.len() > 500,
                "{}'s instructions are too short to be instructions: {} bytes",
                workflow.name,
                workflow.body.len()
            );
            assert!(!workflow.summary.is_empty(), "{}", workflow.name);
            assert!(!workflow.allowed_tools.is_empty(), "{}", workflow.name);
        }
    }

    /// Every workflow declares what it takes, and declares it as optional.
    ///
    /// The optionality is the part worth pinning. A hint written `<thing>` marks the
    /// argument required, and a tool may then reject a bare invocation — which would delete
    /// the behavior that is correct when a contributor has genuinely described nothing:
    /// asking rather than inventing. So the shape of the hint is a contract, not a style
    /// choice, and it is asserted rather than left to whoever adds the next row.
    ///
    /// A body carrying the placeholder is checked where the files are written, because that
    /// is where the frontmatter half lives too and neither half means anything alone.
    #[test]
    fn every_workflow_declares_an_optional_argument() {
        for workflow in WORKFLOWS {
            assert!(
                !workflow.argument_hint.is_empty(),
                "{} declares no argument hint, so a tool has nothing to substitute into",
                workflow.name
            );
            assert!(
                !workflow.argument_hint.contains('<'),
                "{}'s hint `{}` marks the argument required; a bare invocation must keep \
                 working",
                workflow.name,
                workflow.argument_hint
            );
            assert!(
                workflow.body.contains(PLACEHOLDER),
                "{}'s body never substitutes {PLACEHOLDER}, so an argument reaches no agent",
                workflow.name
            );
        }
    }

    /// Every workflow tells an agent what to report.
    ///
    /// A workflow whose instructions end without this leaves the report to improvisation, and a
    /// report is the only part of a workflow a contributor is guaranteed to read. So this is
    /// the one property of a report the program can check: that guidance for it exists.
    ///
    /// What the guidance *says* is not checkable here and deliberately is not attempted.
    /// Whether a report's first claim is stated in a contributor's vocabulary is a judgment, so
    /// the contract states the obligation and a transcript is what verifies it. A word-matching
    /// proxy would pass while the register stayed wrong, which is worse than no check at all
    /// because it would look like coverage.
    #[test]
    fn every_workflow_says_what_to_report() {
        for workflow in WORKFLOWS {
            assert!(
                workflow.body.contains(REPORT_SECTION),
                "{}'s instructions carry no `{REPORT_SECTION}` section, so what a contributor \
                 hears is left to improvisation",
                workflow.name
            );
        }
    }

    /// The pointer rule is worded identically in every body.
    ///
    /// It is prose repeated seven times, which is the arrangement the contract exists to
    /// prevent — the report sections diverged exactly this way before anything governed them,
    /// and `apply` drifted furthest. Repetition is accepted here because a body must be
    /// self-contained: an agent reads one file and no other. So the drift is pinned instead.
    ///
    /// A body that reworded the rule would still pass `every_workflow_says_what_to_report`,
    /// which is why this is a second test rather than another assertion in the first.
    #[test]
    fn the_pointer_rule_is_worded_the_same_everywhere() {
        // The sentence that states the deletion test, which is the part that bounds the rule.
        // A body carrying the instruction without the test makes an agent gloss `--json`.
        const DELETION_TEST: &str = "delete the identifiers from your sentence";
        for workflow in WORKFLOWS {
            assert!(
                workflow.body.contains(POINTER_RULE),
                "{}'s report guidance does not tell an agent to name what its references point \
                 at",
                workflow.name
            );
            assert!(
                workflow.body.contains(DELETION_TEST),
                "{} states the pointer rule without the test that bounds it, so an agent will \
                 gloss references that already name themselves",
                workflow.name
            );
        }
    }

    /// Task 4.4: every body states the contract-authority obligation, in one wording.
    ///
    /// `contract-authority` requires both halves of this: the statement in every body, because an
    /// agent reads one file and no other, and the identical wording checked mechanically, so that
    /// a workflow added later cannot ship without it. Eight independently maintained statements of
    /// one rule diverge — this project measured that divergence in its report guidance one release
    /// ago, at six artifact-subjected lines in `apply.md` against two in `explore.md`.
    ///
    /// The bound is pinned beside the rule, for the reason the pointer rule's deletion test is.
    /// A body carrying the obligation without the sentence about silence turns it into a demand
    /// for a citation on every claim, and an agent that meets a rule firing everywhere learns to
    /// ignore it.
    #[test]
    fn the_contract_rule_is_worded_the_same_everywhere() {
        for workflow in WORKFLOWS {
            assert!(
                workflow.body.contains(CONTRACT_RULE),
                "{}'s instructions do not tell an agent to name the requirement behind a claim \
                 about the project's contract",
                workflow.name
            );
            assert!(
                workflow.body.contains(SILENCE_ROUTES),
                "{} states the contract rule without what to do when no requirement exists, so \
                 an agent that finds nothing has no instruction but to invent one",
                workflow.name
            );
            // The deletion test that bounds it, so the rule does not fire on every sentence.
            assert!(
                workflow.body.contains("remove the claim's subject"),
                "{} states the contract rule without the test that bounds it, so an agent will \
                 cite a requirement for its own reasoning",
                workflow.name
            );
            // The prohibition that names the specific failure the rule exists for.
            assert!(
                workflow.body.contains("invented justification is worse"),
                "{} does not tell an agent that a reason attached to an invented rule is worse \
                 than the bare invention",
                workflow.name
            );
        }
    }

    /// The three routes out of silence reach the bodies that meet an unsettled question.
    ///
    /// Task 4.7 names `new` and `ff` — the two workflows that create a change, where a decision
    /// the contract has not settled arrives most often. The other six are checked for its
    /// *absence* rather than left unasserted: a body carrying advice for a decision it never
    /// takes is a body an agent skims, and the skimming costs the rule its force everywhere.
    ///
    /// A set rather than a sequence. [`WORKFLOWS`] is in lifecycle order and that order is
    /// expected to change — `sync` was inserted into the middle of it one release ago — so a
    /// test comparing an ordered list would fail on a correct set of bodies and report the
    /// wording as wrong.
    #[test]
    fn the_routes_out_of_silence_reach_the_bodies_that_need_them() {
        const ROUTE: &str = "Recording silence is a step, not an exit";
        let carries: BTreeSet<&str> = WORKFLOWS
            .iter()
            .filter(|workflow| workflow.body.contains(ROUTE))
            .map(|workflow| workflow.name)
            .collect();
        assert_eq!(
            carries,
            BTreeSet::from(["new", "ff"]),
            "the three routes are stated in the wrong set of bodies"
        );
    }

    /// No workflow tells an agent to restate part of a requirement it modifies.
    ///
    /// artifact-kinds' "A change's specification states a modified requirement whole" replaced the
    /// partial-updates rule that `sync` and `archive` carried. Under that rule a scenario left out of
    /// a modified requirement was either withdrawn or kept, and the text was the same for both, so
    /// this project's own archive holds both readings in one change. Both phrases are matched
    /// across line breaks, because a body wraps its prose wherever the line runs out.
    ///
    /// The two bodies that perform the merge must also name the requirement, because an agent reads
    /// one body and no other, and the rule is what tells it a scenario left out is a defect to
    /// report rather than a reading to choose.
    #[test]
    fn no_workflow_tells_an_agent_to_restate_part_of_a_requirement() {
        const PARTIAL: &str = "partial update";
        const WHOLE: &str = "A change's specification states a modified requirement whole";
        let partial: Vec<String> = WORKFLOWS
            .iter()
            .filter(|workflow| unwrapped(workflow.body).to_lowercase().contains(PARTIAL))
            .flat_map(|workflow| {
                workflow
                    .body
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| line.to_lowercase().contains("partial"))
                    .map(move |(index, _)| format!("{}.md line {}", workflow.name, index + 1))
            })
            .collect();
        assert!(
            partial.is_empty(),
            "a workflow still tells an agent to apply {PARTIAL}s, so a scenario a change leaves \
             out is read as kept: {}",
            partial.join(", ")
        );
        for name in ["sync", "archive"] {
            let body = workflow_for(name).map_or("", |workflow| workflow.body);
            assert!(
                unwrapped(body).contains(WHOLE),
                "{name} merges a modified requirement without naming \"{WHOLE}\", so an agent \
                 has no rule for a scenario the change left out"
            );
        }
    }

    /// The merge instructions say how a change's specification marks the words it edited, and that
    /// the merge writes the current text.
    ///
    /// document-validation permits `ins` and `del` only in a requirement or a scenario marked
    /// `modified`, in a change's specification, so `archive`, which says how that specification is
    /// written, has to say where the marks go. artifact-kinds' "A marked modification reaches the
    /// accepted contract as its current text" says what the merge leaves behind, and both workflows
    /// that merge have to name it, or an agent copies a `del` into the accepted contract, where
    /// validation refuses it after the fact. The limit every wording test here shares applies: a
    /// body that reworded the instruction would pass.
    #[test]
    fn the_merge_instructions_say_how_marks_are_written_and_removed() {
        const CURRENT_TEXT: &str =
            "A marked modification reaches the accepted contract as its current text";
        const WHERE: &str = "only in a requirement or a scenario marked `modified`";
        let archive = unwrapped(workflow_for("archive").map_or("", |workflow| workflow.body));
        assert!(
            archive.contains("<ins>") && archive.contains("<del>"),
            "archive does not say how a change marks the words it added and deleted"
        );
        assert!(
            archive.contains(WHERE),
            "archive does not say that the marks belong {WHERE}"
        );
        for name in ["sync", "archive"] {
            let body = workflow_for(name).map_or("", |workflow| workflow.body);
            assert!(
                unwrapped(body).contains(CURRENT_TEXT),
                "{name} merges a modified requirement without naming \"{CURRENT_TEXT}\", so an \
                 agent has no rule for the marks a change carries"
            );
        }
    }

    /// The archive instructions say when a verification is `partial`, and when a check gets none.
    ///
    /// `record-a-partial-verification`: a check that reached part of what it names records
    /// `partial` with an `unchecked` paragraph for each part it missed, and a check of something
    /// nearby is not a partial check of the scenario. The second half is the boundary authors meet:
    /// eight of the thirteen declines the change measured were checks of something else, and
    /// without it `partial` becomes the easy answer for them.
    #[test]
    fn the_archive_instructions_say_when_a_check_is_partial() {
        const REQUIREMENT: &str = "A partial verification states what it did not check";
        let archive = unwrapped(workflow_for("archive").map_or("", |workflow| workflow.body));
        assert!(
            archive.contains("data-outcome=\"partial\"")
                && archive.contains("data-field=\"unchecked\""),
            "archive does not say how a check that reached part of its requirement is recorded"
        );
        assert!(
            archive.contains(REQUIREMENT),
            "archive does not name \"{REQUIREMENT}\""
        );
        assert!(
            archive.contains("is not a partial check of it"),
            "archive does not say that a check of something nearby is not a partial check"
        );
    }

    /// The two workflows that read a change's specification beside the accepted contract send the
    /// reader to the view.
    ///
    /// `verify` checks a change against its design, and `archive` restates a modified requirement
    /// before the merge, which is where another change's later edit can still be caught rather
    /// than reverted. toolchain-packaging's "The command surface shows a modified requirement beside
    /// the accepted one" provides the view, and each has to name it and the requirement. `sync`
    /// does not: a merging agent works from `sync`'s own worklist, and both texts side by side
    /// invite the wholesale copy `sync.md` forbids. The limit every wording test here shares
    /// applies: a body that reworded the instruction would pass.
    #[test]
    fn the_review_instructions_send_a_reader_to_the_view() {
        const VIEW: &str =
            "The command surface shows a modified requirement beside the accepted one";
        for name in ["verify", "archive"] {
            let body = unwrapped(workflow_for(name).map_or("", |workflow| workflow.body));
            assert!(
                body.contains("`htmlspec show <change") && body.contains("--diff`"),
                "{name} does not send the reader to `htmlspec show <change> --diff`"
            );
            assert!(body.contains(VIEW), "{name} does not name \"{VIEW}\"");
        }
        let sync = unwrapped(workflow_for("sync").map_or("", |workflow| workflow.body));
        assert!(
            !sync.contains("htmlspec show"),
            "sync sends a merging agent to the view, which shows both texts beside each other"
        );
    }

    /// The workflows that settle, check, and archive tasks teach the deviation.
    ///
    /// `apply` is where a task is settled, so it says how to state one and cites the requirement
    /// that provides it. `verify` and `archive` read tasks for what was done, and for a task done
    /// differently the deviation, not the words, is that claim, so each names it. `archive` also
    /// names `doneDifferently`, the count `list --json` carries. The limit every wording test here
    /// shares applies: a body that reworded the instruction would pass.
    #[test]
    fn the_task_instructions_teach_the_deviation() {
        const REQUIREMENT: &str = "A task declares completion state";
        let body = |name: &str| unwrapped(workflow_for(name).map_or("", |workflow| workflow.body));

        let apply = body("apply");
        assert!(
            apply.contains("data-field=\"deviation\"") && apply.contains(REQUIREMENT),
            "apply does not teach the deviation paragraph and the requirement providing it"
        );
        assert!(
            apply.contains("Keep the task's words as written"),
            "apply does not tell an agent to keep a task's words beside its deviation"
        );
        for name in ["verify", "archive"] {
            assert!(
                body(name).contains("deviation"),
                "{name} does not say to read a task's deviation as what was done"
            );
        }
        assert!(
            body("archive").contains("`doneDifferently`"),
            "archive does not name the count `list --json` carries"
        );
    }

    /// No workflow tells an agent to fold a requirement a change adds into one already accepted.
    ///
    /// `sync` and archival compare identifiers, so a requirement folded into another leaves its own
    /// identifier absent from the accepted specifications for good, and the only report the
    /// instructions leave for an absent identifier calls it withdrawn. document-schema's "Semantic
    /// units declare a change state" gives `added` one meaning, a requirement the accepted
    /// specifications do not hold, so an added requirement that sharpens an accepted one is a
    /// modification stated as an addition, and `sync` tells the agent to report it.
    ///
    /// The phrases are the two the fold-in step was written with. A body that reworded the
    /// instruction would pass, which is the limit every wording test here shares, so `sync` must
    /// also name the requirement its report rests on.
    #[test]
    fn no_workflow_tells_an_agent_to_fold_an_added_requirement_into_another() {
        const FOLDS: [&str; 2] = [
            "an addition to one already accepted",
            "folded a scenario into",
        ];
        const CHANGE_STATE: &str = "Semantic units declare a change state";
        for workflow in WORKFLOWS {
            let body = unwrapped(workflow.body);
            for fold in FOLDS {
                assert!(
                    !body.contains(fold),
                    "{} tells an agent to fold an added requirement into an accepted one (\"{fold}\"), \
                     which leaves the added identifier absent for good",
                    workflow.name
                );
            }
        }
        let sync = workflow_for("sync").map_or("", |workflow| workflow.body);
        assert!(
            unwrapped(sync).contains(CHANGE_STATE),
            "sync reports an added requirement that restates an accepted one without naming \
             \"{CHANGE_STATE}\", the requirement that says what `added` means"
        );
    }

    /// No merge workflow lets a change be archived while a modified requirement is held.
    ///
    /// `sync` holds a modified requirement unmerged when the change leaves an accepted scenario
    /// out, or marks a scenario `unchanged` whose text differs from the accepted one. A held
    /// requirement's identifier is already accepted, and archival compares identifiers, so
    /// `htmlspec archive` counts it as reached and archives without asking. The program cannot see
    /// what this stops on, so the stop has to be in the instructions, and in `archive` it has to
    /// come before the step that archives. A review of this change found `archive` going from its
    /// merge step to archiving with nothing between them.
    #[test]
    fn no_workflow_archives_a_change_while_a_modified_requirement_is_held() {
        const STOPS: [(&str, &str); 2] = [
            (
                "archive",
                "Do not proceed to step 6 while a modified requirement is held",
            ),
            (
                "sync",
                "Do not archive a change while one of its modified requirements is held",
            ),
        ];
        for (name, stop) in STOPS {
            let body =
                workflow_for(name).map_or(String::new(), |workflow| unwrapped(workflow.body));
            let Some(at) = body.find(stop) else {
                panic!(
                    "{name} lets a change be archived while a modified requirement is held: it \
                     no longer says \"{stop}\""
                );
            };
            if name == "archive" {
                let archiving = body.find("### 6. Archive it").unwrap_or(0);
                assert!(
                    at < archiving,
                    "archive states its stop after the step that archives, so an agent reaches \
                     `htmlspec archive` first"
                );
            }
        }
    }

    /// A body with every whitespace run collapsed to one space.
    ///
    /// A body wraps its prose wherever the line runs out, so a phrase a test looks for can span a
    /// line break.
    fn unwrapped(body: &str) -> String {
        body.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// The body carries no frontmatter, because each tool's is generated. A body that
    /// arrived with its own would produce a file with two frontmatter blocks, and the
    /// tool would read the wrong one.
    #[test]
    fn no_body_carries_its_own_frontmatter() {
        for workflow in WORKFLOWS {
            assert!(
                !workflow.body.starts_with("---"),
                "{}'s body starts with frontmatter, which is generated per tool",
                workflow.name
            );
        }
    }

    #[test]
    fn a_workflow_is_found_by_name_and_an_unknown_one_is_not() {
        assert_eq!(workflow_for("archive").map(|w| w.name), Some("archive"));
        assert!(
            workflow_for("propose").is_none(),
            "an unshipped workflow resolved"
        );
    }

    /// Directories are distinct, or two workflows would install over each other. So are
    /// the summaries: seven of them appear as one list in a contributor's tool, and two
    /// workflows described identically cannot be told apart at the moment of choosing.
    #[test]
    fn workflow_names_and_directories_are_distinct() {
        for field in [
            WORKFLOWS.iter().map(|w| w.name).collect::<Vec<_>>(),
            WORKFLOWS.iter().map(|w| w.directory).collect::<Vec<_>>(),
            WORKFLOWS.iter().map(|w| w.summary).collect::<Vec<_>>(),
        ] {
            let mut sorted = field.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(
                sorted.len(),
                field.len(),
                "two workflows collide: {field:?}"
            );
        }
    }

    /// Exactly one workflow begins the lifecycle.
    ///
    /// Two would make the closing message choose arbitrarily between them. None would
    /// make it print nothing on the run where it matters most, which is a contributor's
    /// first.
    #[test]
    fn exactly_one_entry_point() {
        let flagged: Vec<_> = WORKFLOWS
            .iter()
            .filter(|w| w.entry_point)
            .map(|w| w.name)
            .collect();
        assert_eq!(flagged, vec!["new"], "the entry point is not exactly `new`");
        assert_eq!(entry_point().map(|w| w.name), Some("new"));
    }

    /// The table is in lifecycle order, pinned at both ends.
    ///
    /// A shared instructions file writes one section per workflow in this order, so the
    /// order is a property a reader depends on rather than an accident of how the entries
    /// were typed. Pinning both ends catches a row appended in the wrong place, which is
    /// the way this breaks.
    #[test]
    fn the_table_is_in_lifecycle_order() {
        let names: Vec<_> = WORKFLOWS.iter().map(|w| w.name).collect();
        assert_eq!(
            names,
            vec![
                "migrate", "explore", "new", "continue", "ff", "apply", "verify", "sync", "archive"
            ],
            "the workflows are not in lifecycle order"
        );
    }

    /// No workflow body names a request-verb spelling.
    ///
    /// toolchain-packaging's "The command surface names its operations as OpenSpec names them"
    /// gives every operation a second spelling, a request verb and an object (`get sync`), and
    /// requires the instructions this project installs to keep naming the OpenSpec spelling
    /// (`sync`): they are read by agents arriving from OpenSpec, who then have one spelling to
    /// learn.
    ///
    /// This crate cannot see the command line's table of spellings, so the test holds the rule at
    /// the verbs rather than at each object. A body names a verb spelling when the program's name
    /// precedes one of the five verbs, or a backtick opens one followed by its object. That is
    /// stricter than the table, which is the safe direction: no body has a reason to invoke a verb
    /// at all.
    #[test]
    fn no_workflow_names_a_request_verb_spelling() {
        const VERBS: [&str; 5] = ["get", "post", "put", "patch", "delete"];
        for workflow in WORKFLOWS {
            let body = unwrapped(workflow.body);
            for verb in VERBS {
                for spelling in [format!("htmlspec {verb} "), format!("`{verb} ")] {
                    assert!(
                        !body.contains(&spelling),
                        "{}'s instructions name the request-verb spelling \"{}\"; they name the \
                         OpenSpec spelling of every operation",
                        workflow.name,
                        spelling.trim_end()
                    );
                }
            }
        }
    }

    /// The lifecycle covers exploring an idea through archiving the result. Stated as the
    /// stages rather than as a count, because a count is satisfied by seven of anything.
    #[test]
    fn the_lifecycle_is_covered() {
        for stage in ["migrate", "explore", "new", "apply", "verify", "archive"] {
            assert!(
                workflow_for(stage).is_some(),
                "the lifecycle omits {stage}, so a contributor at that stage finds nothing"
            );
        }
    }
}
