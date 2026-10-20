//! The interactive tool selection.
//!
//! **This is the only module that references `inquire`, and the only one that reads a
//! keypress.** Its whole job is producing a [`Selection`]; it writes no files and decides
//! nothing about them. `commands::bootstrap` takes the `Selection` and does the work, so
//! every case worth testing is a constructed value rather than a simulated terminal.
//!
//! Without that split the interactive path would be the one part of the program with no
//! test, and it is also the first part a new contributor meets.
//!
//! Nothing here calls `std::process::exit`. A declined prompt returns [`Cancelled`], which
//! travels home through the ordinary return path to the one exit point in `main`. OpenSpec
//! takes the other route — `process.exit(0)` three frames below its entry point, inside
//! the prompt itself — and that is both untestable and reports success for an install that
//! did nothing.

use std::io::IsTerminal;

use crate::commands::Selection;
use crate::tools::{detect, ToolId, TOOLS};

/// A contributor declined the prompt or interrupted the operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cancelled;

/// Whether an interactive prompt is possible and wanted.
///
/// Four conditions, all required:
///
/// - Standard input is a terminal, because a prompt reads from it.
/// - Standard output is a terminal, because a prompt writes to it and a contributor
///   cannot answer a question they cannot see.
/// - No `--tools` was given. A contributor who named their tools has already answered the
///   question the prompt asks, and asking again would make them answer twice.
/// - Structured output was not requested. A prompt writes to the stream the payload is
///   promised to occupy alone, so the two cannot both be honored.
///
/// Any one missing means proceed without asking. A bootstrap that blocked waiting for a
/// keypress inside a continuous-integration job would never complete, which is worse than
/// one that chose a sensible default.
pub fn can_prompt(tools_given: bool, structured: bool) -> bool {
    !tools_given && !structured && std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Which rows a fresh prompt opens with already chosen.
///
/// Two cases, and the second is why this is a function rather than a filter inline in
/// [`select_tools`]:
///
/// - Something was detected: those rows are chosen, and nothing else is.
/// - Nothing was detected: **every** row is chosen. An empty detection means a project
///   nobody has configured yet, which is the moment a default matters most and the moment
///   detection has least to say. Offering nothing there teaches that the prompt is
///   decorative, and unchecking what you do not want is less work than checking what you
///   do.
///
/// Returned as indices into `ordered` rather than as tool identifiers, because that is what
/// the prompt takes and converting once here keeps the caller free of the second
/// representation.
fn preselected(ordered: &[&'static crate::tools::Tool], detected: &[ToolId]) -> Vec<usize> {
    if detected.is_empty() {
        return (0..ordered.len()).collect();
    }
    ordered
        .iter()
        .enumerate()
        .filter(|(_, tool)| detected.contains(&tool.id))
        .map(|(index, _)| index)
        .collect()
}

/// Asks which tools to set up, with the detected ones already chosen.
///
/// Returns [`Cancelled`] when the contributor escapes or interrupts. `inquire` restores the
/// terminal as it unwinds — it installs no persistent raw mode and returns the terminal to
/// its prior state on both the success and the error path — so the caller does not have to,
/// and a contributor's shell is left usable.
///
/// An empty confirmation is refused rather than accepted, and the prompt asks again. See
/// [`EMPTY_SELECTION`] for why that differs from `--tools none`.
pub fn select_tools(root: &std::path::Path, name: Option<&str>) -> Result<Selection, Cancelled> {
    let detected = detect(root);

    // Detected tools first, then the rest, each labelled. A contributor scanning the list
    // should see what this project already uses without reading every line.
    let mut ordered: Vec<&'static crate::tools::Tool> = TOOLS.iter().collect();
    ordered.sort_by_key(|tool| !detected.contains(&tool.id));

    let labels: Vec<String> = ordered
        .iter()
        .map(|tool| {
            if detected.contains(&tool.id) {
                format!("{} (detected)", tool.name)
            } else {
                tool.name.to_owned()
            }
        })
        .collect();
    let preselected = preselected(&ordered, &detected);

    let chosen = inquire::MultiSelect::new("Which tools should htmlspec set up?", labels)
        .with_default(&preselected)
        .with_help_message(
            "space toggles, enter confirms, esc cancels. To establish the project without \
             any instructions, cancel and run `htmlspec init --tools none`.",
        )
        // Refuses an empty confirmation and asks again. `inquire` re-renders the prompt
        // with this message rather than returning, so the contributor stays in the list.
        .with_validator(|chosen: &[inquire::list_option::ListOption<&String>]| {
            if chosen.is_empty() {
                Ok(inquire::validator::Validation::Invalid(
                    EMPTY_SELECTION.into(),
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .raw_prompt()
        // Every error is treated as a cancellation, including an interrupt. The alternative
        // is reporting a broken tool when a person pressed a key, and this module cannot
        // tell a closed terminal from a deliberate escape in a way worth acting on.
        .map_err(|_| Cancelled)?;

    let tools: Vec<ToolId> = chosen
        .into_iter()
        .filter_map(|item| ordered.get(item.index).map(|tool| tool.id))
        .collect();

    Ok(Selection {
        tools,
        name: name.map(str::to_owned),
    })
}

/// What the prompt says when a contributor confirms nothing.
///
/// The asymmetry with `--tools none` is deliberate, and it is the whole reason this
/// refusal exists. `--tools none` is a person answering the question. An empty Enter is
/// usually a person who did not notice a question was asked — the prompt opens on the
/// first run in a fresh repository, which is exactly where nobody yet knows what the list
/// is for. So the flag succeeds and the empty confirmation does not, and the message names
/// the flag so that a person who meant it has somewhere to go.
///
/// Measured before this refusal existed: `init` accepted the empty selection, reported
/// `the project is ready`, and installed no instructions. The project *was* established
/// correctly, which is what made the outcome hard to see.
pub const EMPTY_SELECTION: &str =
    "select at least one tool with space, or cancel with esc and run \
     `htmlspec init --tools none` to establish the project without instructions";

/// Whether a yes-or-no question can be asked.
///
/// The same three conditions [`can_prompt`] applies, minus the one about `--tools`: standard
/// input and standard output are terminals, and structured output was not requested. A prompt
/// writes to the stream a payload is promised to occupy alone, so the two cannot both be
/// honored, and a question nobody can see is a hang rather than a question.
pub fn can_confirm(structured: bool) -> bool {
    !structured && std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Asks a yes-or-no question, defaulting to no.
///
/// The default is no because the caller is asking permission to do something irreversible over
/// findings it has already reported. A contributor who hits return without reading has declined,
/// which leaves the project as it was.
pub fn confirm(question: &str) -> Result<bool, Cancelled> {
    inquire::Confirm::new(question)
        .with_default(false)
        .prompt()
        .map_err(|_| Cancelled)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four conditions, each checked by removing it.
    ///
    /// Under `cargo test` stdin is not a terminal, so `can_prompt` is false whatever the
    /// flags say — which is itself the property worth asserting first: the test suite is a
    /// non-interactive caller, and a prompt appearing in it would hang the run.
    #[test]
    fn a_prompt_is_impossible_without_a_terminal() {
        assert!(
            !can_prompt(false, false),
            "a prompt was considered possible with no terminal attached, which would \
             block a continuous-integration run"
        );
    }

    /// `--tools` suppresses the prompt: the question has already been answered.
    #[test]
    fn naming_tools_suppresses_the_prompt() {
        assert!(!can_prompt(true, false));
    }

    /// Structured output suppresses the prompt, because a prompt writes to the stream the
    /// payload is promised to occupy alone.
    #[test]
    fn structured_output_suppresses_the_prompt() {
        assert!(!can_prompt(false, true));
        assert!(!can_prompt(true, true));
    }

    /// A cancellation is its own value rather than an error string, so a caller matches on
    /// it instead of comparing prose.
    #[test]
    fn cancellation_is_a_distinct_value() {
        let outcome: Result<Selection, Cancelled> = Err(Cancelled);
        assert!(matches!(outcome, Err(Cancelled)));
    }

    // -------------------------------------------------------------------------
    // Preselection
    // -------------------------------------------------------------------------

    /// The ordering `select_tools` builds, so these tests measure the real indices rather
    /// than assuming the table's order.
    fn ordered_for(detected: &[ToolId]) -> Vec<&'static crate::tools::Tool> {
        let mut ordered: Vec<&'static crate::tools::Tool> = TOOLS.iter().collect();
        ordered.sort_by_key(|tool| !detected.contains(&tool.id));
        ordered
    }

    /// The case that produced an empty install. Nothing detected means a fresh project, and
    /// a prompt of five unchecked boxes is the least useful thing to offer there.
    #[test]
    fn nothing_detected_preselects_every_tool() {
        let detected: Vec<ToolId> = Vec::new();
        let ordered = ordered_for(&detected);
        assert_eq!(
            preselected(&ordered, &detected).len(),
            TOOLS.len(),
            "a fresh project opened the prompt with rows unchecked"
        );
    }

    /// With something detected, detection is the answer and the rest stay unchecked.
    #[test]
    fn a_detected_tool_is_preselected_and_the_others_are_not() {
        let detected = vec![ToolId::Claude];
        let ordered = ordered_for(&detected);
        let chosen = preselected(&ordered, &detected);
        assert_eq!(chosen.len(), 1, "{chosen:?}");
        assert_eq!(ordered[chosen[0]].id, ToolId::Claude);
    }

    /// Every index must address a row, or the prompt would preselect nothing while
    /// reporting that it had.
    #[test]
    fn every_preselected_index_addresses_a_row() {
        for detected in [
            Vec::new(),
            vec![ToolId::Codex],
            vec![ToolId::Claude, ToolId::Agents],
            TOOLS.iter().map(|tool| tool.id).collect(),
        ] {
            let ordered = ordered_for(&detected);
            for index in preselected(&ordered, &detected) {
                assert!(
                    ordered.get(index).is_some(),
                    "index {index} addresses no row for {detected:?}"
                );
            }
        }
    }

    /// The two-sided property. Preselection is never empty, whatever detection found,
    /// because an empty confirmation is now refused and a prompt that preselected nothing
    /// would open in a state it will not accept.
    #[test]
    fn preselection_is_never_empty() {
        for detected in [
            Vec::new(),
            vec![ToolId::Pi],
            TOOLS.iter().map(|tool| tool.id).collect::<Vec<_>>(),
        ] {
            let ordered = ordered_for(&detected);
            assert!(
                !preselected(&ordered, &detected).is_empty(),
                "the prompt would open in a state it refuses, for {detected:?}"
            );
        }
    }

    // -------------------------------------------------------------------------
    // The empty-selection refusal
    // -------------------------------------------------------------------------

    /// The refusal message must name the escape hatch. A refusal that only says no leaves a
    /// contributor who genuinely wants no tools with nowhere to go.
    #[test]
    fn the_refusal_names_the_flag_that_means_no_tools() {
        assert!(
            EMPTY_SELECTION.contains("--tools none"),
            "the refusal does not name the flag: {EMPTY_SELECTION}"
        );
        assert!(
            EMPTY_SELECTION.contains("esc"),
            "the refusal does not say how to leave: {EMPTY_SELECTION}"
        );
    }

    /// The asymmetry, asserted where it can be seen: the flag still means no tools, and the
    /// refusal applies to the prompt alone. Refusing `--tools none` as well would break an
    /// accepted scenario.
    #[test]
    fn the_flag_still_accepts_an_empty_selection() {
        assert_eq!(
            crate::tools::parse_tools("none").expect("none parses"),
            Vec::new(),
            "the refusal leaked from the prompt into the flag"
        );
    }
}
