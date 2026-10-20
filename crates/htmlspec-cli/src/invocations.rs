//! Checking that instruction bodies name only commands this program provides.
//!
//! An instruction file tells an agent how to perform a stage of the lifecycle, and a
//! file naming `htmlspec propose` sends that agent to a parse error the project shipped
//! to it. That is the one hazard the workflow set has to be protected from, and it is
//! narrower than the rule it replaces: a workflow does not need a command that performs
//! it — `archive` has none and is agent-driven by design — it needs to not *name* a
//! command that is not there.
//!
//! The check resolves each invocation against [`clap`]'s own command tree rather than
//! against a list written here. A hand-written list states the command surface a second
//! time, and it drifts in the direction that keeps this check passing while the
//! instructions are wrong.
//!
//! **Extraction is deliberately crude.** A body is prose with fenced code blocks in it,
//! and a parser that knew which occurrences were real invocations would be a parser with
//! its own bugs. Over-eager is the right failure direction: a false positive costs a
//! reworded sentence, and a false negative ships a skill that breaks on first use.

use std::collections::BTreeSet;

use clap::Command;

/// An invocation found in an instruction body that this program does not provide.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Unresolved {
    /// The invocation as the body spells it, beginning with the program name.
    pub invocation: String,
    /// The word that failed to resolve.
    pub word: String,
    /// The subcommand path that did resolve, as a space-separated string. Empty when
    /// the first word after the program name is already unknown.
    pub resolved: String,
}

impl std::fmt::Display for Unresolved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}`: ", self.invocation)?;
        if self.resolved.is_empty() {
            write!(f, "`{}` is not a subcommand of `htmlspec`", self.word)
        } else {
            write!(
                f,
                "`{}` is not a subcommand of `htmlspec {}`",
                self.word, self.resolved
            )
        }
    }
}

/// The words a body offers as a potential invocation, after the program name.
///
/// Stops at the first word that cannot be part of a subcommand path. A flag, a
/// placeholder, or an argument ends the path, because the check is about subcommand
/// names and an argument's spelling is the contributor's business.
fn candidate_words(rest: &str) -> Vec<&str> {
    let mut words = Vec::new();
    for word in rest.split_whitespace() {
        // A subcommand is lowercase letters and hyphens. Anything else — a flag, a
        // `<placeholder>`, a path, a backtick, a sentence's punctuation — ends the path.
        let trimmed = word.trim_end_matches(['.', ',', ':', '`', ')']);
        // A hyphen is legal *inside* a subcommand name (`bulk-archive`) and marks a flag
        // at the start of one (`--tools`). Without the leading-hyphen case, `htmlspec
        // --version` reads as an invocation of a subcommand named `--version`.
        if trimmed.is_empty()
            || trimmed.starts_with('-')
            || !trimmed.chars().all(|c| c.is_ascii_lowercase() || c == '-')
        {
            break;
        }
        words.push(trimmed);
        // Four words is past the deepest path this program has (`new document`), and a
        // bound keeps a prose sentence of lowercase words from being read as a path.
        if words.len() == 4 {
            break;
        }
    }
    words
}

/// Every invocation in `body` that `command` does not provide.
///
/// `command` is the root command, and it must be built — an unbuilt `clap::Command` has
/// not propagated its global arguments or normalized its subcommands.
pub fn unresolved(body: &str, command: &Command) -> Vec<Unresolved> {
    let program = command.get_name().to_string();
    let mut found: BTreeSet<Unresolved> = BTreeSet::new();

    // Every occurrence of the program name, wherever it appears. `htmlspec-kind` and
    // `htmlspec-cli` are not invocations, so a hyphen immediately after the name ends
    // the match — as does any other character that could continue an identifier.
    let mut haystack = body;
    while let Some(at) = haystack.find(&program) {
        let after = &haystack[at + program.len()..];
        // The name must be followed by whitespace to be a command. `htmlspec-kind`,
        // `htmlspec_assets`, and `htmlspecs` are words rather than invocations.
        let boundary_ok = after.chars().next().is_some_and(|c| c == ' ' || c == '\t');
        // And it must not be part of a longer word to its left.
        let before_ok = haystack[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_' && c != '/');

        if boundary_ok && before_ok {
            let words = candidate_words(after);
            if !words.is_empty() {
                if let Some(bad) = resolve(command, &program, &words) {
                    found.insert(bad);
                }
            }
        }
        haystack = &haystack[at + program.len()..];
    }

    found.into_iter().collect()
}

/// Walks `words` down `command`'s subcommands, reporting the first that does not resolve.
///
/// A word that resolves to no subcommand is only a fault when nothing before it consumed
/// it as an argument, which is why the walk stops at the first leaf: `htmlspec archive
/// add-dark-mode` resolves `archive`, and `add-dark-mode` is then that command's
/// argument rather than an unknown subcommand.
fn resolve(command: &Command, program: &str, words: &[&str]) -> Option<Unresolved> {
    let mut current = command;
    let mut resolved: Vec<&str> = Vec::new();

    for word in words {
        match current.find_subcommand(word) {
            Some(next) => {
                resolved.push(word);
                current = next;
            }
            None => {
                // A command with no subcommands of its own takes arguments here, so the
                // word is an argument and the path is complete.
                //
                // Spelled out rather than as `current.get_subcommands().next()?`, which
                // clippy suggests: the `?` reads as "propagate the absence" when what this
                // means is "the word is an argument, so there is no fault to report". The
                // two produce the same value and say different things.
                #[allow(clippy::question_mark)]
                if current.get_subcommands().next().is_none() {
                    return None;
                }
                // Otherwise the word claimed to be a subcommand of something that has
                // them, and it is not one.
                return Some(Unresolved {
                    invocation: format!("{program} {}", words.join(" ")),
                    word: (*word).to_string(),
                    resolved: resolved.join(" "),
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cli;
    use clap::CommandFactory;

    /// The built root command, which is what a caller must pass.
    fn root() -> Command {
        let mut command = Cli::command();
        command.build();
        command
    }

    /// Every invocation the archival instructions already carry resolves.
    ///
    /// This is the body that shipped before the workflow set grew, so it is the one
    /// case known to be correct. A check that failed it would be measuring its own
    /// extraction rather than the instructions.
    #[test]
    fn the_shipped_archival_body_resolves_completely() {
        let body = htmlspec_assets::instructions::workflow_for("archive")
            .expect("archive ships")
            .body;
        let bad = unresolved(body, &root());
        assert!(
            bad.is_empty(),
            "the shipped body reports invocations: {bad:?}"
        );
    }

    /// The withdrawn spelling, which the previous guard existed for.
    #[test]
    fn the_withdrawn_init_path_is_not_itself_an_invocation_fault() {
        // `init doc.html` resolves: `init` is real and `doc.html` is its argument. The
        // withdrawn spelling is reported by the program at runtime, not here, and this
        // check does not claim otherwise. What it claims is the narrower thing.
        assert!(unresolved("run `htmlspec init doc.html` first", &root()).is_empty());
    }

    /// The three faults the previous guard missed.
    #[test]
    fn an_absent_subcommand_is_reported() {
        for body in [
            "then run htmlspec propose to state the obligation",
            "produce it with htmlspec new spec at the change root",
            "check the result with htmlspec verify before archiving",
        ] {
            let bad = unresolved(body, &root());
            assert!(
                !bad.is_empty(),
                "an absent subcommand went unreported in: {body}"
            );
        }
    }

    /// The reported fault names the word and the path that did resolve, because
    /// "something is wrong in this body" is not enough to act on.
    #[test]
    fn a_report_names_the_word_and_the_resolved_path() {
        let bad = unresolved("run htmlspec new spec now", &root());
        let one = bad.first().expect("reported");
        assert_eq!(one.word, "spec");
        assert_eq!(one.resolved, "new");
        assert!(one.to_string().contains("htmlspec new"), "{one}");
    }

    /// A real invocation with real arguments resolves, including the named-change form
    /// the archival body uses.
    #[test]
    fn a_real_invocation_with_arguments_resolves() {
        for body in [
            "htmlspec archive add-dark-mode",
            "htmlspec new change add-dark-mode --subject \"Dark mode\"",
            "htmlspec new document specification spec.html",
            "htmlspec validate",
            "htmlspec format --check",
            "htmlspec init --tools all",
        ] {
            let bad = unresolved(body, &root());
            assert!(bad.is_empty(), "{body} reported {bad:?}");
        }
    }

    /// The program's name inside a longer identifier is not an invocation. The archival
    /// body carries `htmlspec-kind` in a meta tag, and reading that as a command would
    /// make this check unusable.
    #[test]
    fn the_name_inside_an_identifier_is_not_an_invocation() {
        for body in [
            "declared in `<meta name=\"htmlspec-kind\" content=\"proposal\">`",
            "the crate htmlspec-assets holds the bodies",
            "under crates/htmlspec-cli/src",
        ] {
            assert!(
                unresolved(body, &root()).is_empty(),
                "an identifier was read as an invocation: {body}"
            );
        }
    }

    /// Prose after the program's name does not resolve into a false report when the
    /// following word is not subcommand-shaped.
    #[test]
    fn the_bare_name_in_prose_reports_nothing() {
        for body in [
            "the htmlspec 0.1.0 release",
            "htmlspec --version names the build",
            "what htmlspec, as a program, contributes",
        ] {
            assert!(
                unresolved(body, &root()).is_empty(),
                "prose was read as an invocation: {body}"
            );
        }
    }

    /// Every shipped body, which is the assertion that makes the workflow set safe to
    /// grow. This is the test the change exists to put in place: a body added later
    /// naming a command that is not there fails the build rather than a contributor.
    #[test]
    fn no_shipped_body_names_an_absent_command() {
        let command = root();
        for workflow in htmlspec_assets::instructions::WORKFLOWS {
            let bad = unresolved(workflow.body, &command);
            assert!(
                bad.is_empty(),
                "{}'s instructions name commands this program does not provide:\n  {}",
                workflow.name,
                bad.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        }
    }
}
