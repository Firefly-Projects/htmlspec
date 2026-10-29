//! How far each run of a runbook has come: `htmlspec run <runbook> --status [<record>]`.
//!
//! The change `show-how-far-a-run-has-come`. Every file under the runbook's `runs/` is read and
//! judged by [`crate::prerequisites::judge_records`], the run check's own reading and rule, so a
//! record this report calls finished and passed is one a run check without `atLeast` counts, and
//! one it calls open is one preflight names as open (the design's decision 2). What this module adds
//! is the detail the rule reads and does not keep: each step and its state, each dropped step's
//! reason, each requirement's recorded outcomes, and the release the record states.
//!
//! The report writes nothing and exits zero whatever the runs say. Only a runbook or a record that
//! is not there is a failure, because then nothing was reported.

use std::io::Write;

use crate::exit::Outcome;
use crate::output::Streams;
use crate::prerequisites::{
    named_by, outcome_of, owed, Judged, NotSatisfying, Read, RecordRelease, StepState,
};
use crate::terminal::shown;

/// How far one run has come, as its record says.
#[derive(Debug)]
pub(crate) struct RunStatus {
    /// The record's name, without its extension.
    record: String,
    /// What the record holds, or the first reason the file was not read as one of this runbook's
    /// records.
    read: Result<Detail, NotSatisfying>,
}

/// What a record read as one of this runbook's records says.
#[derive(Debug)]
struct Detail {
    /// Where the run stands by the run check's rule.
    standing: Standing,
    /// Every step, in the record's order.
    steps: Vec<Step>,
    /// Every requirement the record owes an outcome for, in the record's order.
    requirements: Vec<Owed>,
    /// Every verification that references none of those requirements, in the record's order.
    others: Vec<Other>,
    /// The runbook release its head states.
    release: RecordRelease,
}

/// Where a read record stands by the run check's rule, without `atLeast`.
#[derive(Debug)]
enum Standing {
    /// Finished and passed: a run check naming this runbook counts it.
    Passed,
    /// Under way: a step is neither done nor dropped. The reason is the rule's first.
    Open(NotSatisfying),
    /// Every step settled, and short of the rule for the reason given.
    NotPassed(NotSatisfying),
}

/// One step of a run.
#[derive(Debug)]
struct Step {
    /// Its place in the record, from 1.
    number: usize,
    /// Its identifier, where it declares one.
    id: Option<String>,
    /// Its heading as a reader names it, where it has one.
    heading: Option<String>,
    /// The state the record gives it.
    state: StepState,
    /// The reason the record gives, which a dropped step states.
    reason: Option<String>,
}

/// One requirement a record owes an outcome for, with the outcomes it records.
#[derive(Debug)]
struct Owed {
    /// The requirement's identifier.
    id: String,
    /// Every outcome a verification referencing it states, in the record's order. Empty where none
    /// does.
    outcomes: Vec<String>,
}

/// A verification that references no requirement the record owes an outcome for: one that
/// references nothing, or only a requirement the record withdraws or does not carry.
#[derive(Debug)]
struct Other {
    /// What the rule calls it: the requirement it references first, or its own identifier.
    name: Option<String>,
    /// The outcome it records, where it records one.
    outcome: Option<String>,
}

impl RunStatus {
    /// A judged file's status.
    pub(crate) fn of(judged: Judged) -> Self {
        Self {
            record: judged.record,
            read: judged.read.map(Detail::of),
        }
    }
}

impl Detail {
    fn of(read: Read) -> Self {
        let Read {
            reading,
            verdict,
            open,
        } = read;
        // `open` is the rule's own answer to "is a step neither done nor dropped", so a run the
        // rule calls open is open here whatever reason the rule gives first.
        let standing = match (verdict, open) {
            (Ok(_), _) => Standing::Passed,
            (Err(reason), Some(_)) => Standing::Open(reason),
            (Err(reason), None) => Standing::NotPassed(reason),
        };
        let model = &reading.model;
        let steps = model
            .tasks
            .iter()
            .enumerate()
            .map(|(index, task)| Step {
                number: index + 1,
                id: task.id.clone(),
                // As a reader names it, without the identifier the heading shows beside it.
                heading: task.title.as_deref().map(|title| {
                    htmlspec_parse::model::HeadingName::of(title, task.id.as_deref())
                        .as_str()
                        .to_owned()
                }),
                state: StepState::of(task),
                reason: task.reason.clone(),
            })
            .collect();
        let owed: Vec<&str> = owed(model).collect();
        let references_owed = |verification: &htmlspec_parse::model::Verification| {
            verification
                .references
                .iter()
                .any(|reference| owed.contains(&reference.target.as_str()))
        };
        let requirements = owed
            .iter()
            .map(|id| Owed {
                id: (*id).to_owned(),
                outcomes: model
                    .verifications
                    .iter()
                    .filter(|verification| {
                        verification
                            .references
                            .iter()
                            .any(|reference| reference.target == *id)
                    })
                    .filter_map(|verification| outcome_of(verification).map(str::to_owned))
                    .collect(),
            })
            .collect();
        // Every verification the rule reads is shown: under a requirement it references, or here.
        // The rule refuses a record for any verification that did not pass, so one that references
        // no requirement the record owes, a stray one or one under a withdrawn requirement, can be
        // the reason the verdict gives, and a view that left it out would name a reason it never
        // shows.
        let others = model
            .verifications
            .iter()
            .filter(|verification| !references_owed(verification))
            .map(|verification| Other {
                name: named_by(verification).map(str::to_owned),
                outcome: outcome_of(verification).map(str::to_owned),
            })
            .collect();
        Self {
            standing,
            steps,
            requirements,
            others,
            release: reading.release,
        }
    }

    fn count(&self, state: StepState) -> usize {
        self.steps.iter().filter(|step| step.state == state).count()
    }

    /// "4/9 done", with the dropped steps where there are any.
    fn tally(&self, separator: &str) -> String {
        let done = self.count(StepState::Done);
        let dropped = self.count(StepState::Dropped);
        let total = self.steps.len();
        if dropped == 0 {
            format!("{done}{separator}{total} done")
        } else {
            format!("{done}{separator}{total} done, {dropped} dropped")
        }
    }

    /// Where an open run is: the steps in progress, or, where none is, the first step neither done
    /// nor dropped.
    fn whereabouts(&self) -> String {
        let in_progress: Vec<String> = self
            .steps
            .iter()
            .filter(|step| step.state == StepState::InProgress)
            .map(|step| step.number.to_string())
            .collect();
        match in_progress.as_slice() {
            [] => self
                .steps
                .iter()
                .find(|step| !step.state.is_settled())
                .map_or_else(
                    || String::from("open"),
                    |step| format!("step {} next", step.number),
                ),
            [one] => format!("step {one} in progress"),
            [first @ .., last] => format!("steps {} and {last} in progress", first.join(", ")),
        }
    }
}

impl StepState {
    /// The word the report uses for the state.
    fn label(self) -> &'static str {
        match self {
            Self::Todo => "to do",
            Self::InProgress => "in progress",
            Self::Done => "done",
            Self::Dropped => "dropped",
            Self::Undeclared => "undeclared",
        }
    }

    /// The value the payload uses, spelled as the task vocabulary spells it.
    fn value(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in-progress",
            Self::Done => "done",
            Self::Dropped => "dropped",
            Self::Undeclared => "undeclared",
        }
    }
}

/// The width of the step label column: the longest label.
const LABEL_WIDTH: usize = "in progress".len();

/// The line one run gets in the listing of every run, after its name.
fn summary(status: &RunStatus) -> String {
    match &status.read {
        Err(reason) => format!("not read: {reason}"),
        Ok(detail) => {
            let tally = detail.tally("/");
            match &detail.standing {
                Standing::Passed => format!("{tally}, finished and passed"),
                Standing::Open(_) => format!("{tally}, {}", detail.whereabouts()),
                Standing::NotPassed(reason) => {
                    format!("{tally}, finished, not passed: {reason}")
                }
            }
        }
    }
}

/// The report of every run of `runbook`, one line each, in name order.
fn listing(runbook: &str, statuses: &[RunStatus]) -> String {
    let runbook = shown(runbook);
    if statuses.is_empty() {
        return format!(
            "{runbook} has no runs on record. Start one with `htmlspec run {runbook} --as <record>`\n"
        );
    }
    let names: Vec<String> = statuses
        .iter()
        .map(|status| shown(&status.record))
        .collect();
    let width = names
        .iter()
        .map(|name| name.chars().count())
        .max()
        .unwrap_or(0);
    let runs = if statuses.len() == 1 { "run" } else { "runs" };
    let mut text = format!("{runbook} has {} {runs}, in name order:\n", statuses.len());
    for (name, status) in names.iter().zip(statuses) {
        text.push_str(&format!("  {name:<width$}  {}\n", summary(status)));
    }
    text
}

/// The release line's value.
fn release_text(release: &RecordRelease) -> String {
    match release {
        RecordRelease::One(version) => version.to_string(),
        RecordRelease::Absent => String::from("not stated"),
        RecordRelease::Malformed(given) => {
            format!("not three dot-separated integers: {}", shown(given))
        }
        RecordRelease::Several => String::from("stated more than once"),
    }
}

/// The report of one run: every step, every requirement, the release, and the verdict.
fn one_run(runbook: &str, status: &RunStatus) -> String {
    let mut text = format!("{} run {}\n", shown(runbook), shown(&status.record));
    let detail = match &status.read {
        Err(reason) => {
            text.push_str(&format!("Finished and passed: no, it {reason}\n"));
            return text;
        }
        Ok(detail) => detail,
    };

    if detail.steps.is_empty() {
        text.push_str("Steps: none\n");
    } else {
        text.push_str(&format!("Steps: {}\n", detail.tally(" of ")));
        let number_width = detail.steps.len().to_string().len();
        // Where a reason sits beneath its step: past the number, its dot, and the label.
        let indent = " ".repeat(2 + number_width + 2 + LABEL_WIDTH + 2);
        for step in &detail.steps {
            let heading = step
                .heading
                .as_deref()
                .map_or_else(|| String::from("(no heading)"), shown);
            text.push_str(
                format!(
                    "  {:>number_width$}. {:<LABEL_WIDTH$}  {heading}",
                    step.number,
                    step.state.label()
                )
                .trim_end(),
            );
            text.push('\n');
            if step.state == StepState::Dropped {
                let reason = step
                    .reason
                    .as_deref()
                    .map_or_else(|| String::from("none given"), shown);
                text.push_str(&format!("{indent}reason: {reason}\n"));
            }
        }
    }

    if detail.requirements.is_empty() {
        text.push_str("Requirements: none\n");
    } else {
        let recorded = detail
            .requirements
            .iter()
            .filter(|requirement| !requirement.outcomes.is_empty())
            .count();
        text.push_str(&format!(
            "Requirements: {recorded} of {} have an outcome\n",
            detail.requirements.len()
        ));
        let labels: Vec<String> = detail
            .requirements
            .iter()
            .map(|requirement| {
                if requirement.outcomes.is_empty() {
                    String::from("no outcome")
                } else {
                    requirement
                        .outcomes
                        .iter()
                        .map(|outcome| shown(outcome))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            })
            .collect();
        let width = labels
            .iter()
            .map(|label| label.chars().count())
            .max()
            .unwrap_or(0);
        for (label, requirement) in labels.iter().zip(&detail.requirements) {
            text.push_str(&format!("  {label:<width$}  {}\n", shown(&requirement.id)));
        }
    }

    if !detail.others.is_empty() {
        text.push_str("Other outcomes:\n");
        let labels: Vec<String> = detail
            .others
            .iter()
            .map(|other| {
                other
                    .outcome
                    .as_deref()
                    .map_or_else(|| String::from("no outcome"), shown)
            })
            .collect();
        let width = labels
            .iter()
            .map(|label| label.chars().count())
            .max()
            .unwrap_or(0);
        for (label, other) in labels.iter().zip(&detail.others) {
            let name = other
                .name
                .as_deref()
                .map_or_else(|| String::from("(unnamed)"), shown);
            text.push_str(&format!("  {label:<width$}  {name}\n"));
        }
    }

    text.push_str(&format!(
        "Runbook release: {}\n",
        release_text(&detail.release)
    ));
    text.push_str(&match &detail.standing {
        Standing::Passed => String::from("Finished and passed: yes\n"),
        Standing::Open(reason) | Standing::NotPassed(reason) => {
            format!("Finished and passed: no, it {reason}\n")
        }
    });
    text
}

/// One run's entry in the payload.
fn entry(status: &RunStatus) -> serde_json::Value {
    let mut entry = serde_json::Map::new();
    entry.insert("record".into(), status.record.clone().into());
    let detail = match &status.read {
        Err(reason) => {
            entry.insert("state".into(), "unreadable".into());
            entry.insert("reason".into(), reason.to_string().into());
            return entry.into();
        }
        Ok(detail) => detail,
    };
    let (state, reason) = match &detail.standing {
        Standing::Passed => ("passed", None),
        Standing::Open(reason) => ("open", Some(reason)),
        Standing::NotPassed(reason) => ("not-passed", Some(reason)),
    };
    entry.insert("state".into(), state.into());
    if let Some(reason) = reason {
        entry.insert("reason".into(), reason.to_string().into());
    }
    entry.insert("done".into(), detail.count(StepState::Done).into());
    entry.insert("dropped".into(), detail.count(StepState::Dropped).into());
    entry.insert("total".into(), detail.steps.len().into());
    let steps: Vec<serde_json::Value> = detail
        .steps
        .iter()
        .map(|step| {
            let mut entry = serde_json::Map::new();
            entry.insert("number".into(), step.number.into());
            if let Some(id) = &step.id {
                entry.insert("id".into(), id.clone().into());
            }
            if let Some(heading) = &step.heading {
                entry.insert("heading".into(), heading.clone().into());
            }
            entry.insert("state".into(), step.state.value().into());
            if let Some(reason) = &step.reason {
                entry.insert("reason".into(), reason.clone().into());
            }
            entry.into()
        })
        .collect();
    entry.insert("steps".into(), steps.into());
    let requirements: Vec<serde_json::Value> = detail
        .requirements
        .iter()
        .map(|requirement| {
            serde_json::json!({ "id": requirement.id, "outcomes": requirement.outcomes })
        })
        .collect();
    entry.insert("requirements".into(), requirements.into());
    let others: Vec<serde_json::Value> = detail
        .others
        .iter()
        .map(|other| {
            let mut entry = serde_json::Map::new();
            if let Some(name) = &other.name {
                entry.insert("name".into(), name.clone().into());
            }
            if let Some(outcome) = &other.outcome {
                entry.insert("outcome".into(), outcome.clone().into());
            }
            entry.into()
        })
        .collect();
    entry.insert("otherOutcomes".into(), others.into());
    let release = match &detail.release {
        RecordRelease::One(version) => {
            serde_json::json!({ "stated": "one", "version": version.to_string() })
        }
        RecordRelease::Absent => serde_json::json!({ "stated": "none" }),
        RecordRelease::Malformed(given) => {
            serde_json::json!({ "stated": "malformed", "given": given })
        }
        RecordRelease::Several => serde_json::json!({ "stated": "several" }),
    };
    entry.insert("release".into(), release);
    entry.into()
}

/// The payload both forms write: the runbook, and one entry per run reported.
fn payload(runbook: &str, statuses: &[&RunStatus]) -> serde_json::Value {
    serde_json::json!({
        "runbook": runbook,
        "records": statuses.iter().map(|status| entry(status)).collect::<Vec<_>>(),
    })
}

/// Why `--status <record>` names no run: the records the runbook has, and the spelling where the
/// name was given with its extension.
fn no_such_record(runbook: &str, wanted: &str, records: &[String]) -> String {
    let runbook = shown(runbook);
    let mut message = if records.is_empty() {
        format!(
            "{runbook} has no run recorded as {}, and no runs on record",
            shown(wanted)
        )
    } else {
        format!(
            "{runbook} has no run recorded as {}. Its records: {}",
            shown(wanted),
            records
                .iter()
                .map(|record| shown(record))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    if let Some(stem) = wanted.strip_suffix(".html") {
        if records.iter().any(|record| record == stem) {
            message.push_str(&format!(
                ". A record is named without its .html: `htmlspec run {runbook} --status {}`",
                shown(stem)
            ));
        }
    }
    message
}

/// Reports how far each run of `typed` has come, or the run recorded as `record`.
///
/// Writes nothing. Exits zero for any report, and with an operational failure where the runbook or
/// the record is not there.
pub fn status<O: Write, E: Write>(
    typed: &str,
    record: Option<&str>,
    store: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let project = match crate::store::project_for(store) {
        Ok(project) => project,
        Err(report) => {
            let _ = streams.failure(&report);
            return Outcome::OperationalFailure;
        }
    };
    // Found as `run --check` finds a runbook, by 1.0's name rule, so every name 1.2 ran still
    // reports, and then known by its enumerated name, as `run --as` and `preflight` know it.
    let directory = match project.named_runbook(typed) {
        Ok(directory) => directory,
        Err(error) => {
            let _ = streams.failure(&error.to_string());
            return Outcome::OperationalFailure;
        }
    };
    if !directory.is_dir() {
        let _ = streams.failure(&format!(
            "no runbook named {} in {}. To see what the project holds, run `htmlspec list \
             --runbooks`",
            shown(typed),
            project.runbooks().display()
        ));
        return Outcome::OperationalFailure;
    }
    let Some(runbook) = crate::prerequisites::enumerated_name(&directory, typed) else {
        let _ = streams.failure(&format!(
            "{} reaches a runbook directory that is listed under no one name, so which runbook it \
             means is not known. Name the runbook as `htmlspec list --runbooks` prints it",
            shown(typed)
        ));
        return Outcome::OperationalFailure;
    };

    let statuses: Vec<RunStatus> =
        crate::prerequisites::judge_records(&project.runs(&runbook), None)
            .into_iter()
            .map(RunStatus::of)
            .collect();
    let selected = match record {
        None => Selected::Every(statuses),
        Some(wanted) => {
            let names: Vec<String> = statuses
                .iter()
                .map(|status| status.record.clone())
                .collect();
            // Matched byte for byte against the names `runs/` holds, and never opened by the typed
            // name, so a path or a name differing only in case finds nothing on any filesystem.
            match statuses.into_iter().find(|status| status.record == wanted) {
                Some(found) => Selected::One(found),
                None => {
                    let _ = streams.failure(&no_such_record(&runbook, wanted, &names));
                    return Outcome::OperationalFailure;
                }
            }
        }
    };

    if streams.form().is_structured() {
        let statuses = match &selected {
            Selected::Every(statuses) => statuses.iter().collect(),
            Selected::One(status) => vec![status],
        };
        if let Ok(json) = serde_json::to_string_pretty(&payload(&runbook, &statuses)) {
            let _ = streams.payload(&json);
        }
        return Outcome::Success;
    }
    match &selected {
        Selected::One(status) => {
            let _ = streams.report(&one_run(&runbook, status));
        }
        Selected::Every(statuses) => {
            let _ = streams.report(&listing(&runbook, statuses));
            if !statuses.is_empty() {
                let _ = streams.progress(&format!(
                    "To see one run's steps and outcomes: `htmlspec run {} --status <record>`",
                    shown(&runbook)
                ));
            }
        }
    }
    Outcome::Success
}

/// What `--status` was asked about: every run, or the one it named.
enum Selected {
    /// Every run of the runbook, in name order.
    Every(Vec<RunStatus>),
    /// The run recorded under the name given.
    One(RunStatus),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prerequisites::{judge, reading_of, RELEASE_META};

    /// One step of a record to build: its state, its heading, and its reason, where it has one.
    struct Task<'a>(&'a str, &'a str, Option<&'a str>);

    /// A record's source: `tasks`, the requirements it carries, the outcome each verification
    /// records for a requirement, and the releases its head states. A requirement written `~req-x`
    /// is marked withdrawn, and a verification for the requirement `""` references none.
    fn record(
        tasks: &[Task],
        requirements: &[&str],
        verifications: &[(&str, &str)],
        releases: &[&str],
    ) -> String {
        let metas: String = releases
            .iter()
            .map(|release| format!("<meta content=\"{release}\" name=\"{RELEASE_META}\">"))
            .collect();
        let tasks: String = tasks
            .iter()
            .enumerate()
            .map(|(index, Task(state, heading, reason))| {
                let reason = reason.map_or(String::new(), |reason| {
                    format!("<p data-field=\"reason\">{reason}</p>")
                });
                format!(
                    "<li class=\"task\" id=\"task-{index}\" data-change=\"unchanged\" \
                     data-state=\"{state}\"><h3>{heading} <span class=\"hs-id\">task-{index}</span>\
                     </h3>{reason}</li>"
                )
            })
            .collect();
        let requirements: String = requirements
            .iter()
            .map(|id| {
                let (id, change) = id
                    .strip_prefix('~')
                    .map_or((*id, "unchanged"), |id| (id, "removed"));
                format!(
                    "<section class=\"requirement\" id=\"{id}\" data-change=\"{change}\">\
                     <h3>It</h3><p>A run MUST do it.</p>\
                     <article class=\"scenario\" id=\"scenario-{id}\" data-change=\"unchanged\">\
                     <h4>Done</h4><dl><dt>When</dt><dd>It runs.</dd><dt>Then</dt><dd>It is \
                     done.</dd></dl></article></section>"
                )
            })
            .collect();
        let verifications: String = verifications
            .iter()
            .enumerate()
            .map(|(index, (requirement, outcome))| {
                let refs = if requirement.is_empty() {
                    String::new()
                } else {
                    format!(" data-refs=\"{requirement}\"")
                };
                format!(
                    "<section class=\"verification\" id=\"verify-{index}\" \
                     data-change=\"unchanged\" data-outcome=\"{outcome}\"{refs}>\
                     <h3>Recorded</h3>\
                     <p data-field=\"method\">Looked.</p></section>"
                )
            })
            .collect();
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"1.0.0\" name=\"htmlspec-version\">\
             <meta content=\"tasks\" name=\"htmlspec-kind\">{metas}<title>A run</title></head>\
             <body><main><ul>{tasks}</ul>{requirements}{verifications}</main></body></html>"
        )
    }

    /// The status of a file named `name` holding `source`, as `judge_records` reads one.
    fn status(name: &str, source: &str) -> RunStatus {
        RunStatus {
            record: name.to_owned(),
            read: reading_of(source).map(|reading| {
                let (verdict, open) = judge(&reading, None);
                Detail::of(Read {
                    reading,
                    verdict,
                    open,
                })
            }),
        }
    }

    /// The listing line of a record whose steps are in `states`, carrying one requirement with
    /// `verifications`.
    fn line(states: &[&str], verifications: &[(&str, &str)]) -> String {
        let tasks: Vec<Task> = states
            .iter()
            .map(|state| Task(state, "A step", None))
            .collect();
        summary(&status(
            "run",
            &record(&tasks, &["req-a"], verifications, &[]),
        ))
    }

    #[test]
    fn each_standing_has_its_line() {
        let passed = [("req-a", "passed")];
        assert_eq!(
            line(&["done", "dropped"], &passed),
            "1/2 done, 1 dropped, finished and passed"
        );
        assert_eq!(line(&["done"], &passed), "1/1 done, finished and passed");
        assert_eq!(
            line(&["done", "in-progress", "todo"], &[]),
            "1/3 done, step 2 in progress"
        );
        assert_eq!(
            line(&["in-progress", "done", "in-progress"], &[]),
            "1/3 done, steps 1 and 3 in progress"
        );
        assert_eq!(
            line(
                &["in-progress", "in-progress", "dropped", "in-progress"],
                &[]
            ),
            "0/4 done, 1 dropped, steps 1, 2 and 4 in progress"
        );
        assert_eq!(
            line(&["done", "dropped", "todo", "todo"], &[]),
            "1/4 done, 1 dropped, step 3 next"
        );
        // A run just started is open, and its first step is next.
        assert_eq!(line(&["todo", "todo"], &[]), "0/2 done, step 1 next");
        // A step with no state the vocabulary holds is not finished.
        assert_eq!(
            line(&["done", "sideways"], &passed),
            "1/2 done, step 2 next"
        );
        assert_eq!(
            line(&["done"], &[("req-a", "failed")]),
            "1/1 done, finished, not passed: recorded failed for req-a"
        );
        assert_eq!(
            line(&["done"], &[]),
            "1/1 done, finished, not passed: records no outcome for req-a"
        );
        assert_eq!(
            line(&["dropped", "dropped"], &passed),
            "0/2 done, 2 dropped, finished, not passed: has no task done, so it records no work"
        );
        // A record with no steps did no work, and is finished: nothing in it is open.
        assert_eq!(
            summary(&status("run", &record(&[], &[], &[], &[]))),
            "0/0 done, finished, not passed: has no task done, so it records no work"
        );
    }

    #[test]
    fn every_outcome_but_passed_is_named_as_not_passed() {
        for outcome in ["failed", "blocked", "partial", "not-run"] {
            assert_eq!(
                line(&["done"], &[("req-a", outcome)]),
                format!("1/1 done, finished, not passed: recorded {outcome} for req-a"),
            );
        }
    }

    #[test]
    fn a_file_not_read_as_a_record_says_why() {
        assert_eq!(
            summary(&status("empty", "")),
            "not read: is not a run's record: not a tasks document"
        );
        assert_eq!(
            summary(&status("recovering", "<p>an <b>unclosed paragraph</p>")),
            "not read: needs error recovery to parse, so it has more than one reading and none is \
             chosen"
        );
        let proposal = record(&[], &[], &[], &[]).replace(
            "<meta content=\"tasks\" name=\"htmlspec-kind\">",
            "<meta content=\"proposal\" name=\"htmlspec-kind\">",
        );
        assert_eq!(
            summary(&status("proposal", &proposal)),
            "not read: is not a run's record: not a tasks document"
        );
        let unread = RunStatus {
            record: String::from("locked"),
            read: Err(NotSatisfying::Unreadable(String::from("permission denied"))),
        };
        assert_eq!(
            summary(&unread),
            "not read: cannot be read: permission denied"
        );
        assert_eq!(
            one_run("rb", &unread),
            "rb run locked\nFinished and passed: no, it cannot be read: permission denied\n"
        );
    }

    #[test]
    fn one_run_lists_every_step_requirement_the_release_and_the_verdict() {
        let source = record(
            &[
                Task("done", "Start", None),
                Task("dropped", "Bootstrap", Some("Already there.")),
                Task("in-progress", "Push", None),
                Task("todo", "Tag", None),
                Task("dropped", "Announce", None),
                Task("sideways", "Watch", None),
            ],
            &["req-a", "req-b", "req-c"],
            &[
                ("req-a", "passed"),
                ("req-b", "failed"),
                ("req-b", "passed"),
            ],
            &["0.3.0"],
        );
        assert_eq!(
            one_run("publish", &status("v1.3.0", &source)),
            "publish run v1.3.0\n\
             Steps: 1 of 6 done, 2 dropped\n\
             \x20 1. done         Start\n\
             \x20 2. dropped      Bootstrap\n\
             \x20                 reason: Already there.\n\
             \x20 3. in progress  Push\n\
             \x20 4. to do        Tag\n\
             \x20 5. dropped      Announce\n\
             \x20                 reason: none given\n\
             \x20 6. undeclared   Watch\n\
             Requirements: 2 of 3 have an outcome\n\
             \x20 passed          req-a\n\
             \x20 failed, passed  req-b\n\
             \x20 no outcome      req-c\n\
             Runbook release: 0.3.0\n\
             Finished and passed: no, it is still open, with 1 of 6 steps done\n"
        );
    }

    #[test]
    fn a_passed_run_and_a_record_with_nothing_in_it() {
        let passed = record(
            &[Task("done", "Start", None)],
            &["req-a"],
            &[("req-a", "passed")],
            &[],
        );
        assert_eq!(
            one_run("rb", &status("one", &passed)),
            "rb run one\n\
             Steps: 1 of 1 done\n\
             \x20 1. done         Start\n\
             Requirements: 1 of 1 have an outcome\n\
             \x20 passed  req-a\n\
             Runbook release: not stated\n\
             Finished and passed: yes\n"
        );
        assert_eq!(
            one_run("rb", &status("empty", &record(&[], &[], &[], &[]))),
            "rb run empty\n\
             Steps: none\n\
             Requirements: none\n\
             Runbook release: not stated\n\
             Finished and passed: no, it has no task done, so it records no work\n"
        );
    }

    #[test]
    fn each_release_a_head_can_state() {
        let release = |releases: &[&str]| {
            let source = record(&[Task("done", "Start", None)], &[], &[], releases);
            let status = status("run", &source);
            let Ok(detail) = &status.read else {
                panic!("the record reads");
            };
            (
                release_text(&detail.release),
                entry(&status)["release"].clone(),
            )
        };
        assert_eq!(
            release(&["1.4.2"]),
            (
                String::from("1.4.2"),
                serde_json::json!({ "stated": "one", "version": "1.4.2" })
            )
        );
        assert_eq!(
            release(&[]),
            (
                String::from("not stated"),
                serde_json::json!({ "stated": "none" })
            )
        );
        assert_eq!(
            release(&["one"]),
            (
                String::from("not three dot-separated integers: one"),
                serde_json::json!({ "stated": "malformed", "given": "one" })
            )
        );
        assert_eq!(
            release(&["1.0.0", "1.0.0"]),
            (
                String::from("stated more than once"),
                serde_json::json!({ "stated": "several" })
            )
        );
    }

    #[test]
    fn the_listing_pads_names_and_counts_runs() {
        let passed = record(&[Task("done", "Start", None)], &[], &[], &[]);
        let statuses = [status("a", &passed), status("longer-name", "")];
        assert_eq!(
            listing("rb", &statuses),
            "rb has 2 runs, in name order:\n\
             \x20 a            1/1 done, finished and passed\n\
             \x20 longer-name  not read: is not a run's record: not a tasks document\n"
        );
        assert_eq!(
            listing("rb", &statuses[..1]),
            "rb has 1 run, in name order:\n  a  1/1 done, finished and passed\n"
        );
        assert_eq!(
            listing("rb", &[]),
            "rb has no runs on record. Start one with `htmlspec run rb --as <record>`\n"
        );
    }

    #[test]
    fn the_payload_carries_what_is_known_and_leaves_out_what_is_not() {
        let open = record(
            &[
                Task("done", "Start", None),
                Task("dropped", "Skip", Some("Not needed.")),
                Task("in-progress", "Push", None),
            ],
            &["req-a", "req-b"],
            &[("req-a", "passed")],
            &["0.3.0"],
        );
        assert_eq!(
            entry(&status("v1", &open)),
            serde_json::json!({
                "record": "v1",
                "state": "open",
                "reason": "is still open, with 1 of 3 steps done",
                "done": 1,
                "dropped": 1,
                "total": 3,
                "steps": [
                    { "number": 1, "id": "task-0", "heading": "Start", "state": "done" },
                    { "number": 2, "id": "task-1", "heading": "Skip", "state": "dropped",
                      "reason": "Not needed." },
                    { "number": 3, "id": "task-2", "heading": "Push", "state": "in-progress" },
                ],
                "requirements": [
                    { "id": "req-a", "outcomes": ["passed"] },
                    { "id": "req-b", "outcomes": [] },
                ],
                "otherOutcomes": [],
                "release": { "stated": "one", "version": "0.3.0" },
            })
        );
        let passed = record(&[Task("done", "Start", None)], &[], &[], &[]);
        let entry_of_passed = entry(&status("v2", &passed));
        assert_eq!(entry_of_passed["state"], "passed");
        assert_eq!(entry_of_passed.get("reason"), None);
        let failed = record(
            &[Task("done", "Start", None)],
            &["req-a"],
            &[("req-a", "failed")],
            &[],
        );
        assert_eq!(entry(&status("v3", &failed))["state"], "not-passed");
        assert_eq!(
            entry(&status("v4", "")),
            serde_json::json!({
                "record": "v4",
                "state": "unreadable",
                "reason": "is not a run's record: not a tasks document",
            })
        );
    }

    /// An empty outcome is no outcome, as the rule reads it: the requirement is listed without one,
    /// and the payload carries none.
    #[test]
    fn an_empty_outcome_is_no_outcome() {
        let source = record(
            &[Task("done", "Start", None)],
            &["req-a"],
            &[("req-a", "")],
            &[],
        );
        let empty = status("v1", &source);
        let text = one_run("rb", &empty);
        assert!(
            text.contains("Requirements: 0 of 1 have an outcome\n  no outcome  req-a\n"),
            "{text}"
        );
        assert!(
            text.ends_with("Finished and passed: no, it recorded no outcome for req-a\n"),
            "{text}"
        );
        assert_eq!(
            entry(&empty)["requirements"],
            serde_json::json!([{ "id": "req-a", "outcomes": [] }])
        );
    }

    /// A verification naming no requirement the record owes is shown, so a verdict that names it
    /// names something the view shows: a stray one, and one under a withdrawn requirement.
    #[test]
    fn a_verification_naming_no_owed_requirement_is_shown() {
        let stray = record(
            &[Task("done", "Start", None)],
            &["req-a"],
            &[("req-a", "passed"), ("", "failed"), ("", "")],
            &[],
        );
        let strayed = status("v1", &stray);
        let text = one_run("rb", &strayed);
        assert!(
            text.contains(
                "  passed  req-a\n\
                 Other outcomes:\n\
                 \x20 failed      verify-1\n\
                 \x20 no outcome  verify-2\n"
            ),
            "{text}"
        );
        assert!(
            text.ends_with("Finished and passed: no, it recorded failed for verify-1\n"),
            "{text}"
        );
        assert_eq!(
            entry(&strayed)["otherOutcomes"],
            serde_json::json!([{ "name": "verify-1", "outcome": "failed" }, { "name": "verify-2" }])
        );

        let withdrawn = record(
            &[Task("done", "Start", None)],
            &["~req-a"],
            &[("req-a", "failed")],
            &[],
        );
        let text = one_run("rb", &status("v2", &withdrawn));
        assert!(
            text.contains("Requirements: none\nOther outcomes:\n  failed  req-a\n"),
            "{text}"
        );
        assert!(
            text.ends_with("Finished and passed: no, it recorded failed for req-a\n"),
            "{text}"
        );
        // A record with no verification beside its requirements shows no such line.
        let plain = record(
            &[Task("done", "Start", None)],
            &["req-a"],
            &[("req-a", "passed")],
            &[],
        );
        assert!(!one_run("rb", &status("v3", &plain)).contains("Other outcomes"));
    }

    #[test]
    fn text_a_document_or_a_directory_holds_is_shown_by_code_point() {
        let source = record(
            &[
                Task("in-progress", "Push \u{202E}txt.exe", None),
                Task("dropped", "Skip", Some("because \u{202E}reasons")),
            ],
            &["req-a"],
            &[("req-a", "pa\u{202E}ssed")],
            &["\u{202E}1.0.0"],
        );
        let hostile = status("v1\u{1b}[31m", &source);
        let shown_text = format!(
            "{}{}{}",
            one_run("rb\u{1b}[2J", &hostile),
            listing("rb\u{1b}[2J", std::slice::from_ref(&hostile)),
            no_such_record("rb\u{1b}[2J", "x\u{1b}", &[String::from("v1\u{202E}")]),
        );
        assert!(!shown_text.contains('\u{1b}'), "{shown_text}");
        assert!(!shown_text.contains('\u{202E}'), "{shown_text}");
        assert!(shown_text.contains("v1<U+001B>[31m"), "{shown_text}");
        assert!(shown_text.contains("Push <U+202E>txt.exe"), "{shown_text}");
        assert!(
            shown_text.contains("because <U+202E>reasons"),
            "{shown_text}"
        );
        assert!(shown_text.contains("pa<U+202E>ssed"), "{shown_text}");
        assert!(
            shown_text.contains("not three dot-separated integers: <U+202E>1.0.0"),
            "{shown_text}"
        );
        // The payload carries the text as it is, because JSON escapes it.
        assert_eq!(entry(&hostile)["record"], "v1\u{1b}[31m");
    }

    #[test]
    fn a_record_that_is_not_there_is_named_with_the_ones_that_are() {
        let records = [String::from("v1"), String::from("v2")];
        assert_eq!(
            no_such_record("rb", "v3", &records),
            "rb has no run recorded as v3. Its records: v1, v2"
        );
        assert_eq!(
            no_such_record("rb", "v1.html", &records),
            "rb has no run recorded as v1.html. Its records: v1, v2. A record is named without \
             its .html: `htmlspec run rb --status v1`"
        );
        assert_eq!(
            no_such_record("rb", "v1", &[]),
            "rb has no run recorded as v1, and no runs on record"
        );
    }
}
