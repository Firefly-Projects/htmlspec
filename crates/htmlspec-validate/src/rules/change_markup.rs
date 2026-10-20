//! `schema/change-markup`: HTML's change markup in or around a semantic unit, outside the
//! one place it states something true.
//!
//! `del` marks text an edit removed and `ins` text an edit added. Projection section 8.6
//! gives them their meaning wherever they sit: a deleted word leaves every field, and is
//! recorded in its requirement's or scenario's `deletions`. That meaning is a claim about
//! accepted text, and a unit restates accepted text as a change edited it in one place
//! only: a requirement or a scenario marked `modified`, in a change's specification. There
//! the markup is permitted. Anywhere else it states an edit of nothing a reader can check,
//! and in an accepted specification it is a merge that left its marks behind. So this is a
//! `schema/*` error and not a review warning.
//!
//! Where the document sits is read from [`Subject::placement`], which the command line
//! derives from the document's path, so the rule never works it out again. What a change
//! element stands against is read from the parse tree the projection carries, as
//! [`super::executable`] does, because the model holds text and not markup. A unit is
//! recognized by the projection's own test, a whole `class` token that
//! [`UnitKind::from_class`] accepts, so the two cannot disagree about where a unit begins.
//! The decision itself is [`verdict`], a pure function of the two.
//!
//! Change markup outside every unit is not reported. Prose there projects into no
//! unit's field, and a proposal may show a wording change with it.

use kuchikiki::iter::NodeIterator;
use kuchikiki::{ElementData, NodeRef};

use htmlspec_parse::model::{DEFAULT_CHANGE_STATE, RENAMED_FROM_FIELD};
use htmlspec_parse::UnitKind;

use crate::diagnostic::{Location, Namespace, RuleId};
use crate::engine::{Diagnostics, Subject};
use crate::references::Placement;
use crate::schema_facts;

/// An `ins` or a `del` element inside a semantic unit, or enclosing one.
pub const CHANGE_MARKUP: RuleId = RuleId::new(Namespace::Schema, "change-markup");

/// How many words of an element's text a diagnostic quotes before eliding the rest.
///
/// Enough to find a phrase in a long statement, and short enough that a unit wrapped
/// whole is not repeated in full.
const QUOTED_WORDS: usize = 8;

/// The namespace of HTML's elements.
///
/// An element named `del` inside SVG or MathML is that language's element, not HTML's
/// change markup, and the parser places it in that language's namespace.
const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// HTML's two change elements.
///
/// Their names are HTML's vocabulary and not the schema's, so they are spelled here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Change {
    /// `ins`: text an edit added.
    Inserted,
    /// `del`: text an edit removed.
    Deleted,
}

impl Change {
    /// The change element `element` is, if it is one.
    fn of(element: &ElementData) -> Option<Self> {
        if &*element.name.ns != HTML_NAMESPACE {
            return None;
        }
        let name = element.name.local.as_ref();
        if name.eq_ignore_ascii_case("ins") {
            Some(Self::Inserted)
        } else if name.eq_ignore_ascii_case("del") {
            Some(Self::Deleted)
        } else {
            None
        }
    }

    /// The element's tag name.
    const fn name(self) -> &'static str {
        match self {
            Self::Inserted => "ins",
            Self::Deleted => "del",
        }
    }
}

/// A unit as a diagnostic names it: its kind, and its identifier when it declares one. It
/// also keeps its change state and its element, which decide whether markup in it is permitted.
struct Unit {
    kind: UnitKind,
    id: Option<String>,
    /// The unit's `data-change`, trimmed, or the schema's default when it declares none.
    state: String,
    node: NodeRef,
}

impl Unit {
    /// The unit `node` creates, if it creates one.
    ///
    /// Projection section 1.2's test: a whole, case-sensitive token of `class`. An
    /// element carrying two reserved tokens creates no unit in the projection, which
    /// reports it under section 1.3. It is still a unit here, named by its first
    /// token, because the markup inside it is wrong whichever unit was meant.
    fn of(node: &NodeRef) -> Option<Self> {
        let element = node.as_element()?;
        let attributes = element.attributes.borrow();
        let kind = attributes
            .get("class")?
            .split_whitespace()
            .find_map(UnitKind::from_class)?;
        let id = attributes
            .get("id")
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_owned);
        let state = attributes
            .get("data-change")
            .map_or(DEFAULT_CHANGE_STATE, str::trim)
            .to_owned();
        Some(Self {
            kind,
            id,
            state,
            node: node.clone(),
        })
    }

    /// The unit as a sentence names it.
    fn described(&self) -> String {
        match &self.id {
            Some(id) => format!("the {} {id}", self.kind),
            None => format!("a {} with no identifier", self.kind),
        }
    }
}

/// Where a change element stands against the units, which decides the location and
/// the fix.
enum Reach {
    /// The element is itself a unit, because its own `class` holds a reserved token.
    Itself(Unit),
    /// The element sits inside `unit`, the nearest one enclosing it. When it also
    /// encloses a unit, `enclosed` is the first in document order, and the fix is that
    /// unit's change state.
    Inside { unit: Unit, enclosed: Option<Unit> },
    /// The element sits outside every unit and encloses this one, the first in
    /// document order.
    Around(Unit),
}

impl Reach {
    /// Where `node` stands, or `None` when it is outside every unit and encloses none.
    fn of(node: &NodeRef) -> Option<Self> {
        if let Some(unit) = Unit::of(node) {
            return Some(Self::Itself(unit));
        }
        let enclosed = || {
            node.descendants()
                .find_map(|descendant| Unit::of(&descendant))
        };
        if let Some(unit) = node.ancestors().find_map(|ancestor| Unit::of(&ancestor)) {
            return Some(Self::Inside {
                unit,
                enclosed: enclosed(),
            });
        }
        enclosed().map(Self::Around)
    }

    /// What [`verdict`] reads of where `element`, the change element, stands. `modified` is the
    /// schema's value for a unit whose text a change edited.
    fn context(&self, element: &NodeRef, modified: Option<&str>) -> MarkupContext {
        match self {
            Self::Itself(_) => MarkupContext::IsAUnit,
            Self::Inside {
                enclosed: Some(_), ..
            }
            | Self::Around(_) => MarkupContext::EnclosesAUnit,
            Self::Inside {
                unit,
                enclosed: None,
            } => MarkupContext::Inside {
                unit: unit.kind,
                marked: if modified == Some(unit.state.as_str()) {
                    Marked::Modified
                } else {
                    Marked::Otherwise
                },
                part: Part::of(element, &unit.node),
            },
        }
    }
}

impl Part {
    /// Where `element` sits in `unit`, its nearest enclosing unit: in or around a name when a
    /// heading or a `renamed-from` paragraph is among its ancestors below the unit, or among its
    /// descendants.
    fn of(element: &NodeRef, unit: &NodeRef) -> Self {
        let is_name = |node: &NodeRef| {
            node.as_element().is_some_and(|data| {
                matches!(
                    data.name.local.as_ref(),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                ) || data.attributes.borrow().get("data-field").map(str::trim)
                    == Some(RENAMED_FROM_FIELD)
            })
        };
        let within = element
            .ancestors()
            .take_while(|ancestor| ancestor != unit)
            .any(|ancestor| is_name(&ancestor));
        if within || element.descendants().any(|descendant| is_name(&descendant)) {
            Self::Name
        } else {
            Self::Content
        }
    }
}

/// The schema's change state for a unit whose text a change edited.
///
/// Found by its stem rather than spelled, as `refs::removed_state` finds the removed one,
/// because `schema_rust_split` keeps the schema's values out of rule code. `None` when the
/// schema holds no such value, or two, and then no unit is taken to be modified, so every mark
/// is refused, which is the rule this one relaxes.
fn modified_state() -> Option<String> {
    let mut matches = schema_facts::enum_values("definitions/changeState")
        .into_iter()
        .filter(|value| value.contains("modif"));
    let found = matches.next()?;
    matches.next().is_none().then_some(found)
}

/// What the rule decides for one change element in or around a unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Verdict {
    /// The element marks words a change edited, where a change's edit can be read.
    Permitted,
    /// The element is reported, for this reason.
    Refused(Refusal),
}

/// Why change markup is not permitted where it sits.
///
/// In the order [`verdict`] tries them, so a message names the fix that applies first: markup in
/// an accepted specification is told to state the text as it now reads, and never to mark the unit
/// `modified` or to use `renamed-from`, because neither belongs there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Refusal {
    /// The element's own class names a unit, so it is that unit.
    IsAUnit,
    /// The element encloses a unit, whose change state says whether it is new or withdrawn.
    EnclosesAUnit,
    /// The nearest unit is one no accepted specification holds, or a task.
    NotMarkable(UnitKind),
    /// The document is not a change's specification, so no accepted text is being restated.
    NotAChangeSpecification(Placement),
    /// The element sits in the unit's heading, or in a requirement's `renamed-from` paragraph.
    InAName,
    /// The nearest unit is marked otherwise than `modified`.
    NotModified,
}

/// Where a change element stands, as far as the permission is concerned.
///
/// Built from the tree by the rule, and read by [`verdict`] alone, so every case can be decided in
/// a unit test with no tree and no project.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MarkupContext {
    /// The element's own class names a unit.
    IsAUnit,
    /// The element encloses a unit, whether or not a unit encloses it.
    EnclosesAUnit,
    /// The element sits inside a unit and encloses none.
    Inside {
        /// The nearest unit enclosing the element, which the element's text belongs to.
        unit: UnitKind,
        /// Whether that unit is marked `modified`.
        marked: Marked,
        /// Where in that unit the element sits.
        part: Part,
    },
}

/// Whether a unit is marked with the change state that says its text was edited.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Marked {
    /// `data-change="modified"`.
    Modified,
    /// Any other change state, the default included.
    Otherwise,
}

/// Where in its nearest unit a change element sits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Part {
    /// In or around the unit's heading, or a requirement's `renamed-from` paragraph.
    Name,
    /// Anywhere else in the unit: a statement, a note, a scenario entry.
    Content,
}

/// Whether change markup is permitted where `markup` places it, in a document at `placement`.
///
/// document-validation's "Change markup in or around a semantic unit is rejected": permitted in
/// the content of a requirement or a scenario marked `modified`, in a change's specification, and
/// refused everywhere else, for the first reason in [`Refusal`]'s order that applies.
pub(crate) fn verdict(placement: Placement, markup: &MarkupContext) -> Verdict {
    let (unit, marked, part) = match *markup {
        MarkupContext::IsAUnit => return Verdict::Refused(Refusal::IsAUnit),
        MarkupContext::EnclosesAUnit => return Verdict::Refused(Refusal::EnclosesAUnit),
        MarkupContext::Inside { unit, marked, part } => (unit, marked, part),
    };
    let refusal = if !matches!(unit, UnitKind::Requirement | UnitKind::Scenario) {
        Refusal::NotMarkable(unit)
    } else if placement != Placement::ChangeSpecification {
        Refusal::NotAChangeSpecification(placement)
    } else if part == Part::Name {
        Refusal::InAName
    } else if marked == Marked::Otherwise {
        Refusal::NotModified
    } else {
        return Verdict::Permitted;
    };
    Verdict::Refused(refusal)
}

/// Reports every change element in or around a unit that [`verdict`] refuses.
pub fn validate(subject: &Subject<'_>, diagnostics: &mut Diagnostics) {
    let Some(document) = subject.projection.document.as_ref() else {
        return;
    };
    let placement = subject.placement();
    let modified = modified_state();
    for element in document.inclusive_descendants().elements() {
        let Some(change) = Change::of(&element) else {
            continue;
        };
        let node = element.as_node();
        let Some(reach) = Reach::of(node) else {
            continue;
        };
        let Verdict::Refused(refusal) =
            verdict(placement, &reach.context(node, modified.as_deref()))
        else {
            continue;
        };
        let (unit, message) = message(
            change,
            &quoted(change, node),
            reach,
            refusal,
            modified.as_deref().unwrap_or_default(),
        );
        diagnostics.violation(
            CHANGE_MARKUP,
            Location::maybe_unit(unit.id.as_deref()),
            message,
        );
    }
}

/// The element as its author wrote it, with its text normalized and long text elided.
///
/// Whitespace runs become one space, as projection section 6.1 normalizes a field, so
/// the quotation reads as the projected text does.
fn quoted(change: Change, node: &NodeRef) -> String {
    let words: Vec<String> = node
        .text_contents()
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    let text = if words.len() > QUOTED_WORDS {
        format!("{} …", words[..QUOTED_WORDS].join(" "))
    } else {
        words.join(" ")
    };
    let name = change.name();
    format!("<{name}>{text}</{name}>")
}

/// The unit a diagnostic is located at, and what it says.
///
/// Each refusal names its own fix, and the reason the markup is not permitted where it sits.
/// Markup around a unit, or that is one, is told the change state that says what it meant. Markup
/// outside a change's specification is told to state the text as it now reads, and never to mark
/// the unit `modified` or to use `renamed-from`, because neither belongs there.
fn message(
    change: Change,
    quoted: &str,
    reach: Reach,
    refusal: Refusal,
    modified: &str,
) -> (Unit, String) {
    let name = change.name();
    match reach {
        Reach::Itself(unit) => {
            let kind = unit.kind;
            let (why, state) = wrapped(change, &unit);
            let message = format!(
                "{quoted} is itself {}, because its class names the unit, so {why}. Put the \
                 {kind} on an element that is not change markup, and mark it {state}",
                unit.described()
            );
            (unit, message)
        }
        Reach::Inside {
            unit,
            enclosed: Some(enclosed),
        } => {
            let (why, state) = wrapped(change, &enclosed);
            let message = format!(
                "{quoted} sits in {} and encloses {}, so {why}. Remove the {name} element, and \
                 mark the {} {state}",
                unit.described(),
                enclosed.described(),
                enclosed.kind
            );
            (unit, message)
        }
        Reach::Around(unit) => {
            let kind = unit.kind;
            let (why, state) = wrapped(change, &unit);
            let message = format!(
                "{quoted} encloses {}, so {why}. Remove the {name} element, and mark the {kind} \
                 {state}",
                unit.described()
            );
            (unit, message)
        }
        Reach::Inside {
            unit,
            enclosed: None,
        } => {
            let message = inside(change, quoted, &unit, refusal, modified);
            (unit, message)
        }
    }
}

/// What a diagnostic says about markup inside `unit` that encloses no unit.
fn inside(change: Change, quoted: &str, unit: &Unit, refusal: Refusal, modified: &str) -> String {
    let name = change.name();
    let kind = unit.kind;
    let described = unit.described();
    match refusal {
        Refusal::NotMarkable(UnitKind::Task) => format!(
            "{quoted} sits in {described}. A task carries no change markup: a task whose words \
             were given up is marked data-state=\"dropped\" with a reason, and its state shows \
             it. Write the task as it now reads, without the {name} element"
        ),
        Refusal::NotMarkable(_) => format!(
            "{quoted} sits in {described}. Change markup marks the words a change edited in \
             accepted text, and an accepted specification holds requirements and scenarios and \
             no {kind}, so a {kind} has no accepted text to mark against. Write the text as it \
             now reads, without the {name} element"
        ),
        Refusal::NotAChangeSpecification(placement) => {
            let what = match change {
                Change::Deleted => format!(
                    "A deleted word is left out of what the {kind} states, so the page shows \
                     words the {kind} no longer holds"
                ),
                Change::Inserted => format!(
                    "An inserted word is part of what the {kind} states, and marking it claims \
                     an edit that nothing here applies"
                ),
            };
            format!(
                "{quoted} sits in {described}, in {}. {what}. Change markup is permitted only in \
                 a change's specification, which restates accepted text for a merge to apply. \
                 Write the text as it now reads, without the {name} element",
                placed(placement)
            )
        }
        Refusal::InAName => match kind {
            UnitKind::Requirement => format!(
                "{quoted} sits in the name of {described}. A heading is a name and not a \
                 statement: write the heading as the requirement is now named, and state the \
                 name it replaced in a renamed-from paragraph, without change markup"
            ),
            _ => format!(
                "{quoted} sits in the heading of {described}. A heading is a name and not a \
                 statement, and a {kind} keeps its identifier when its heading changes, so write \
                 the heading as it now reads, without the {name} element"
            ),
        },
        // `verdict` gives these two only for markup that is a unit or encloses one, which
        // `message` answers from the tree before it asks here. Stated in general terms all the
        // same, so a later change to `verdict` cannot put another refusal's reason on them.
        Refusal::IsAUnit | Refusal::EnclosesAUnit => format!(
            "{quoted} sits in {described}, and a unit's change state, not change markup, says \
             whether a unit is new or withdrawn. Remove the {name} element, and give the unit it \
             meant its change state"
        ),
        Refusal::NotModified => format!(
            "{quoted} sits in {described}, which is marked data-change=\"{}\". Change markup \
             marks the words a change edited in accepted text, so it is permitted only in a \
             requirement or a scenario marked data-change=\"{modified}\". Write the text as it \
             now reads, without the {name} element, or mark the {kind} \
             data-change=\"{modified}\" if this change edited its accepted text",
            unit.state
        ),
    }
}

/// Where a document sits, as a diagnostic names it after "in".
fn placed(placement: Placement) -> &'static str {
    match placement {
        Placement::Loose => "a document that sits in no change",
        Placement::ChangeDocument => "a change's proposal, design, or tasks",
        Placement::ChangeSpecification => "a change's specification",
        Placement::AcceptedSpecification => "an accepted specification",
        Placement::RunbookDocument => "a runbook's document",
    }
}

/// What change markup wrapped around a whole unit fails to say, and the change state
/// that says it.
fn wrapped(change: Change, unit: &Unit) -> (String, &'static str) {
    match change {
        Change::Deleted => (
            format!(
                "the {} still projects as one the document states, because a unit is created by \
                 its class and withdrawn by its change state",
                unit.kind
            ),
            "data-change=\"removed\" to withdraw it",
        ),
        Change::Inserted => (
            format!("the {} takes no change state from it", unit.kind),
            "data-change=\"added\" to introduce it",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{run, Strictness};
    use htmlspec_parse::{parse_document, project};

    /// A document of `kind` declaring `version`, with `body` as its body.
    fn document(kind: &str, version: &str, body: &str) -> String {
        format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\
             <meta charset=\"utf-8\">\
             <meta name=\"htmlspec-version\" content=\"{version}\">\
             <meta name=\"htmlspec-kind\" content=\"{kind}\">\
             <title>Fixture</title></head>\n<body>\n{body}\n</body>\n</html>\n"
        )
    }

    /// Runs this rule alone over a document.
    fn check_document(html: &str) -> crate::Report {
        let outcome = parse_document(html);
        let projection = project(&outcome.document);
        run(
            &outcome,
            &projection,
            &htmlspec_assets::Store::Absent,
            &crate::references::Scope::loose(),
            &[validate],
            Strictness::Default,
        )
    }

    /// Runs this rule over a specification declaring the oldest accepted version.
    fn check(body: &str) -> crate::Report {
        check_document(&document("specification", "0.1.0", body))
    }

    fn fired(report: &crate::Report) -> Vec<&crate::Diagnostic> {
        report
            .diagnostics
            .iter()
            .filter(|d| d.rule == CHANGE_MARKUP)
            .collect()
    }

    fn messages(report: &crate::Report) -> String {
        report
            .diagnostics
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The unit identifiers the diagnostics are located at, in report order.
    fn located(report: &crate::Report) -> Vec<Option<&str>> {
        fired(report)
            .iter()
            .map(|d| d.location.unit_id.as_deref())
            .collect()
    }

    const SCENARIO: &str = "<article class=\"scenario\" id=\"scenario-x\"><h3>S</h3>\
         <dl><dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd></dl></article>";

    /// A modified requirement whose statement is `statement`, followed by `extra`.
    fn requirement(statement: &str, extra: &str) -> String {
        format!(
            "<section class=\"requirement\" id=\"req-x\" data-change=\"modified\">\
             <h2>The spellings are fixed <span class=\"hs-id\">req-x</span></h2>\
             <p>{statement}</p>{extra}{SCENARIO}</section>"
        )
    }

    const SPELLINGS: &str = "The spellings are fixed: <code>list</code>, <code>status</code>, \
         <code>sync</code>, <code>instructions</code>, and ";

    /// "A deleted phrase and an inserted word in one statement": one error for each
    /// element, each naming the requirement.
    #[test]
    fn a_deleted_phrase_and_an_inserted_word_are_two_errors() {
        let report = check(&requirement(
            &format!(
                "{SPELLINGS}<del>nothing else</del><ins><code>store</code></ins> SHALL name them."
            ),
            "",
        ));
        let found = fired(&report);
        assert_eq!(found.len(), 2, "{}", messages(&report));
        assert_eq!(located(&report), [Some("req-x"), Some("req-x")]);
        assert!(
            found[0].message.contains("<del>nothing else</del>"),
            "{}",
            found[0].message
        );
        assert!(
            found[1].message.contains("<ins>store</ins>"),
            "{}",
            found[1].message
        );
        assert!(found.iter().all(|d| d.is_error()));
        assert!(!report.is_accepted());
    }

    /// The fix the message gives for markup inside a unit of a loose document: the text as it
    /// now reads, and the one place the markup is permitted.
    ///
    /// `reject-change-markup-inside-a-unit` also told the author that a unit whose words a change
    /// edits is marked `modified`. `mark-the-words-a-modification-changed` permits the markup in
    /// such a unit in a change's specification only, so outside one that hint would point at a
    /// fix that does not apply, and the message names where the markup is permitted instead.
    #[test]
    fn markup_inside_a_unit_is_told_to_state_the_text_as_it_now_reads() {
        let report = check(&requirement("The system <del>MUST</del> SHALL work.", ""));
        let found = fired(&report);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("as it now reads")
                && found[0]
                    .message
                    .contains("permitted only in a change's specification")
                && !found[0].message.contains("data-change=\"modified\""),
            "{}",
            found[0].message
        );
    }

    /// "A deleted paragraph in a requirement".
    #[test]
    fn a_deleted_paragraph_in_a_requirement_is_reported() {
        let report = check(&requirement(
            "The system SHALL work.",
            "<del><p>No other spelling SHALL name a command.</p></del>",
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
        assert!(!report.is_accepted());
    }

    /// "An inserted word with nothing marked as removed".
    #[test]
    fn an_inserted_word_alone_is_reported() {
        let report = check(&requirement("The system SHALL <ins>always</ins> work.", ""));
        let found = fired(&report);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("<ins>always</ins>"),
            "{}",
            found[0].message
        );
        assert!(!report.is_accepted());
    }

    /// "Change markup in a nested unit": the scenario is the nearest unit, so the
    /// diagnostic names it and not the requirement around it.
    #[test]
    fn change_markup_in_a_scenario_names_the_scenario() {
        let scenario = "<article class=\"scenario\" id=\"scenario-y\"><h3>S</h3>\
             <dl><dt>When</dt><dd>It <del>fails</del> happens.</dd>\
             <dt>Then</dt><dd>It works.</dd></dl></article>";
        let report = check(&requirement("The system SHALL work.", scenario));
        assert_eq!(
            located(&report),
            [Some("scenario-y")],
            "{}",
            messages(&report)
        );
    }

    /// "A unit wrapped in change markup": the `del` is outside every unit, so only a
    /// rule that also looks down from it finds the requirement.
    #[test]
    fn a_requirement_wrapped_in_del_is_reported_at_the_requirement() {
        let report = check(&format!(
            "<section><h2>Requirements</h2><del>{}</del></section>",
            requirement("No other spelling SHALL name a command.", "")
        ));
        let found = fired(&report);
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
        assert!(
            found[0].message.contains("encloses")
                && found[0].message.contains("data-change=\"removed\""),
            "{}",
            found[0].message
        );
        assert!(!report.is_accepted());
    }

    /// An `ins` around a unit is the same error, and its fix is the other change
    /// state.
    #[test]
    fn a_requirement_wrapped_in_ins_is_told_to_mark_it_added() {
        let report = check(&format!(
            "<ins>{}</ins>",
            requirement("The system SHALL work.", "")
        ));
        let found = fired(&report);
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
        assert!(
            found[0].message.contains("data-change=\"added\""),
            "{}",
            found[0].message
        );
    }

    /// A wrapped unit is quoted by its opening words, and the rest is elided.
    #[test]
    fn a_long_text_is_quoted_by_its_opening_words() {
        let report = check(&format!(
            "<del>{}</del>",
            requirement(
                "No other spelling SHALL name a command, whichever surface it reaches.",
                ""
            )
        ));
        let found = fired(&report);
        assert_eq!(found.len(), 1, "{}", messages(&report));
        assert!(
            found[0].message.contains("<del>The spellings are fixed")
                && found[0].message.contains('…')
                && !found[0].message.contains("It works."),
            "{}",
            found[0].message
        );
    }

    /// A `del` wrapping a requirement encloses its scenario too, and is reported once,
    /// at the first unit it encloses in document order.
    #[test]
    fn a_wrapping_element_is_reported_once_at_the_first_unit_it_encloses() {
        let report = check(&format!(
            "<del>{}{}</del>",
            requirement("The system SHALL work.", "").replace("req-x", "req-first"),
            requirement("The system SHALL also work.", "")
                .replace("req-x", "req-second")
                .replace("scenario-x", "scenario-second")
        ));
        assert_eq!(
            located(&report),
            [Some("req-first")],
            "{}",
            messages(&report)
        );
    }

    /// "Any unit of any kind": every unit the schema defines, in every artifact kind.
    #[test]
    fn every_unit_in_every_kind_is_reported() {
        let units = [
            (
                "req-x",
                requirement("The system SHALL <del>never</del> work.", ""),
            ),
            (
                "scenario-y",
                "<article class=\"scenario\" id=\"scenario-y\"><h3>S</h3><dl><dt>When</dt>\
                 <dd>It <del>fails</del> happens.</dd><dt>Then</dt><dd>It works.</dd></dl>\
                 </article>"
                    .to_owned(),
            ),
            (
                "decision-x",
                "<section class=\"decision\" id=\"decision-x\" data-status=\"accepted\">\
                 <h2>D</h2><p>The decision.</p>\
                 <p data-field=\"rationale\">Why <del>not</del>.</p></section>"
                    .to_owned(),
            ),
            (
                "task-x",
                "<section class=\"task\" id=\"task-x\" data-state=\"todo\">\
                 <h2>Write <del>the old</del><ins>the new</ins> rule</h2></section>"
                    .to_owned(),
            ),
            (
                "evidence-x",
                "<section class=\"evidence\" id=\"evidence-x\"><h2>E</h2>\
                 <p>Measured <del>twice</del>.</p></section>"
                    .to_owned(),
            ),
            (
                "risk-x",
                "<section class=\"risk\" id=\"risk-x\"><h2>K</h2><p>Fails.</p>\
                 <p data-field=\"mitigation\"><ins>Held.</ins></p></section>"
                    .to_owned(),
            ),
            (
                "verify-x",
                "<section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
                 data-refs=\"req-x\"><h2>V</h2>\
                 <p data-field=\"method\">Ran it<del> once</del>.</p></section>"
                    .to_owned(),
            ),
        ];
        // The unit list is the schema's, so a unit added to the contract fails here
        // until it is covered.
        for (unit, prefix) in crate::schema_facts::identifier_prefixes() {
            assert!(
                units.iter().any(|(id, html)| id.starts_with(&prefix)
                    && html.contains(&format!("class=\"{unit}\""))),
                "no case carries a {unit} unit"
            );
        }
        for kind in crate::schema_facts::artifact_kinds() {
            for (id, html) in &units {
                let report = check_document(&document(&kind, "0.1.0", html));
                assert!(
                    !fired(&report).is_empty() && located(&report).iter().all(|l| l == &Some(*id)),
                    "{kind}, {id}: {}",
                    messages(&report)
                );
                assert!(!report.is_accepted(), "{kind}, {id}");
            }
        }
    }

    /// A requirement in a proposal states nothing authoritative, and its model is read
    /// all the same.
    #[test]
    fn a_requirement_in_a_proposal_or_a_design_is_reported() {
        for kind in ["proposal", "design"] {
            let report = check_document(&document(
                kind,
                "0.1.0",
                &requirement("The system SHALL <del>never</del> work.", ""),
            ));
            assert_eq!(
                located(&report),
                [Some("req-x")],
                "{kind}: {}",
                messages(&report)
            );
        }
    }

    /// "A document declaring an earlier contract version": every accepted version,
    /// the current one included, gets the same report.
    #[test]
    fn no_declared_version_exempts_a_document() {
        let versions = htmlspec_schema::facts::contract_versions();
        assert!(versions.len() > 1, "{versions:?}");
        for version in versions {
            let report = check_document(&document(
                "specification",
                &version,
                &requirement("The system SHALL <del>never</del> work.", ""),
            ));
            assert_eq!(
                located(&report),
                [Some("req-x")],
                "{version}: {}",
                messages(&report)
            );
            assert!(!report.is_accepted(), "{version}");
        }
    }

    /// One diagnostic per element, each quoting its own text, nested elements
    /// included.
    #[test]
    fn each_element_is_reported_with_its_own_text() {
        let report = check(&requirement(
            "The <del>old</del> system SHALL <del>never <ins>always</ins></del> work \
             <ins>fast</ins>.",
            "",
        ));
        let found = fired(&report);
        let quoted: Vec<&str> = [
            "<del>old</del>",
            "<del>never always</del>",
            "<ins>always</ins>",
            "<ins>fast</ins>",
        ]
        .into_iter()
        .filter(|quote| found.iter().any(|d| d.message.contains(quote)))
        .collect();
        assert_eq!(found.len(), 4, "{}", messages(&report));
        assert_eq!(quoted.len(), 4, "{}", messages(&report));
    }

    /// A change element nested in inline markup still has the unit as an ancestor,
    /// though not as its parent.
    #[test]
    fn a_del_nested_in_inline_markup_is_reported() {
        let report = check(&requirement(
            "The system SHALL work <strong><del>quickly</del></strong>.",
            "",
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// A unit's heading is inside the unit.
    #[test]
    fn a_del_in_a_units_heading_is_reported() {
        let report = check(&requirement("The system SHALL work.", "").replace(
            "<h2>The spellings are fixed",
            "<h2>The spellings are <del>open</del> fixed",
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// A unit with no identifier is still reported, at the document.
    #[test]
    fn a_unit_with_no_identifier_is_still_reported() {
        let report = check(
            &requirement("The system SHALL <del>never</del> work.", "")
                .replace(" id=\"req-x\"", ""),
        );
        assert_eq!(located(&report), [None], "{}", messages(&report));
        assert!(
            fired(&report)[0].message.contains("requirement"),
            "{}",
            messages(&report)
        );
    }

    /// An element carrying two reserved tokens creates no unit and is a projection
    /// fault. It is still a unit by this rule's test, so its markup is reported too.
    #[test]
    fn markup_in_an_element_with_two_reserved_classes_is_reported() {
        let report = check(
            "<section class=\"requirement task\" id=\"req-x\"><h2>R</h2>\
             <p>The system SHALL <del>never</del> work.</p></section>",
        );
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// "Change markup outside every unit": a proposal showing a wording change in
    /// prose.
    #[test]
    fn change_markup_outside_every_unit_reports_nothing() {
        let report = check_document(&document(
            "proposal",
            "0.1.0",
            "<section id=\"hs-what-changes\"><h2>What changes</h2>\
             <p>The spellings become <del>nothing else</del><ins><code>store</code></ins>.</p>\
             <del><p>A withdrawn paragraph.</p></del></section>",
        ));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// Change markup in the masthead's summary is outside every unit, and design
    /// decision 5 leaves it unreported.
    #[test]
    fn change_markup_in_the_summary_reports_nothing() {
        let report = check(&format!(
            "<header><h1>T</h1><p>The surface gains <del>no new spelling</del>\
             <ins>one spelling</ins>.</p></header>{}",
            requirement("The system SHALL work.", "")
        ));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// Change markup between a section heading and the first unit sits outside every
    /// unit, which is the valid conformance fixture's placement.
    #[test]
    fn change_markup_beside_a_unit_reports_nothing() {
        let report = check(&format!(
            "<section><h2>Requirements</h2>\
             <p>The spellings become <del>nothing else</del><ins><code>store</code></ins>.</p>\
             {}</section>",
            requirement("The system SHALL work.", "")
        ));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// "A struck-through phrase is not change markup", for `s` and the obsolete
    /// `strike`.
    #[test]
    fn a_struck_through_phrase_reports_nothing() {
        for element in ["s", "strike"] {
            let report = check(&requirement(
                &format!("The system SHALL <{element}>never</{element}> work."),
                "",
            ));
            assert!(
                fired(&report).is_empty(),
                "{element}: {}",
                messages(&report)
            );
        }
    }

    /// `requirements` is not a reserved token. Section 1.2 matches whole tokens, and
    /// so does this rule.
    #[test]
    fn markup_in_an_element_classed_requirements_reports_nothing() {
        let report = check(
            "<section class=\"requirements\"><h2>Requirements</h2>\
             <p>The spellings become <del>nothing else</del>.</p></section>",
        );
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// A change element whose own class names a unit is that unit, and the projection
    /// reads it as one the document states.
    #[test]
    fn a_change_element_that_is_itself_a_unit_is_reported() {
        let report = check_document(&document(
            "design",
            "0.1.0",
            "<del class=\"decision\" id=\"decision-x\" data-status=\"accepted\"><h2>D</h2>\
             <p>The decision.</p><p data-field=\"rationale\">Why.</p></del>",
        ));
        let found = fired(&report);
        assert_eq!(
            located(&report),
            [Some("decision-x")],
            "{}",
            messages(&report)
        );
        assert!(
            found[0]
                .message
                .contains("is itself the decision decision-x")
                && found[0].message.contains("data-change=\"removed\""),
            "{}",
            found[0].message
        );
    }

    /// The element that is the unit is named, and not the first unit inside it.
    #[test]
    fn a_del_classed_as_a_requirement_is_reported_at_the_requirement() {
        let report = check(&format!(
            "<del class=\"requirement\" id=\"req-x\"><h2>R</h2>\
             <p>The system SHALL work.</p>{SCENARIO}</del>"
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// Inside a unit and around another: located at the nearest enclosing unit, and the
    /// fix is the change state of the unit it wraps.
    #[test]
    fn markup_in_a_unit_around_another_names_the_unit_it_wraps() {
        let scenario = "<ins><article class=\"scenario\" id=\"scenario-new\"><h3>S</h3>\
             <dl><dt>When</dt><dd>W.</dd><dt>Then</dt><dd>T.</dd></dl></article></ins>";
        let report = check(&requirement("The system SHALL work.", scenario));
        let found = fired(&report);
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
        assert!(
            found[0]
                .message
                .contains("encloses the scenario scenario-new")
                && found[0]
                    .message
                    .contains("mark the scenario data-change=\"added\""),
            "{}",
            found[0].message
        );
    }

    /// A unit any depth below the change element is enclosed by it, not only a child.
    #[test]
    fn a_unit_wrapped_through_an_intervening_element_is_reported() {
        let report = check(&format!(
            "<del><div>{}</div></del>",
            requirement("The system SHALL work.", "")
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// Section 1.2 matches a reserved token case sensitively, so `Requirement` creates
    /// no unit, and markup in it is outside every unit.
    #[test]
    fn a_class_token_differing_in_case_creates_no_unit() {
        let report = check(
            "<section class=\"Requirement\" id=\"req-x\"><h2>R</h2>\
             <p>The system SHALL <del>never</del> work.</p></section>",
        );
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// The quotation normalizes every whitespace run, as a field's text is normalized.
    #[test]
    fn the_quotation_normalizes_every_whitespace_run() {
        let report = check(&requirement(
            "The system SHALL <del>never\n\t   at  all</del> work.",
            "",
        ));
        let found = fired(&report);
        assert!(
            found[0].message.contains("<del>never at all</del>"),
            "{}",
            found[0].message
        );
    }

    /// Each element gives its own reason. A deleted phrase is out of the statement and still on
    /// the page; an inserted one is current text marked as an edit nothing applies.
    ///
    /// Before projection section 8.6 a deleted phrase was still part of the statement, and the
    /// reason said so. It is not any longer, so the reason is what the page shows against what
    /// the unit states.
    #[test]
    fn each_change_element_gives_its_own_reason() {
        let report = check(&requirement(
            "The system SHALL <del>never</del><ins>always</ins> work.",
            "",
        ));
        let found = fired(&report);
        assert_eq!(found.len(), 2, "{}", messages(&report));
        assert!(
            found[0]
                .message
                .contains("deleted word is left out of what the requirement states"),
            "{}",
            found[0].message
        );
        assert!(
            found[1]
                .message
                .contains("inserted word is part of what the requirement states")
                && !found[1].message.contains("left out"),
            "{}",
            found[1].message
        );
    }

    /// An element named `del` or `ins` inside SVG or MathML is that language's element,
    /// not HTML's change markup.
    #[test]
    fn a_foreign_element_named_like_change_markup_reports_nothing() {
        let report = check(&requirement(
            "The system SHALL work.",
            "<svg><text>Label <del>old</del></text></svg>\
             <math><mi>n</mi><ins>x</ins></math>",
        ));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    // -------------------------------------------------------------------------
    // `verdict`: design decision 3's table, row by row, and each placement
    // -------------------------------------------------------------------------

    /// The context of a change element in the content of a `unit` marked `marked`.
    fn inside(unit: UnitKind, marked: Marked) -> MarkupContext {
        MarkupContext::Inside {
            unit,
            marked,
            part: Part::Content,
        }
    }

    /// Every placement, listed through a match with no wildcard, so a sixth variant fails to
    /// compile here until it is decided.
    fn every_placement() -> [Placement; 5] {
        let decided = |placement: Placement| match placement {
            Placement::Loose
            | Placement::ChangeDocument
            | Placement::ChangeSpecification
            | Placement::AcceptedSpecification
            | Placement::RunbookDocument => placement,
        };
        [
            Placement::Loose,
            Placement::ChangeDocument,
            Placement::ChangeSpecification,
            Placement::AcceptedSpecification,
            Placement::RunbookDocument,
        ]
        .map(decided)
    }

    /// Row "a `modified` scenario inside a `modified` requirement": permitted. The scenario is
    /// the nearest unit, and its own state decides.
    #[test]
    fn a_modified_scenario_is_permitted_in_a_change_specification() {
        assert_eq!(
            verdict(
                Placement::ChangeSpecification,
                &inside(UnitKind::Scenario, Marked::Modified)
            ),
            Verdict::Permitted
        );
    }

    /// Rows "an `unchanged` scenario inside a `modified` requirement", "a unit marked `added`",
    /// and "a unit marked `removed`": the verdict sees only whether the nearest unit is marked
    /// `modified`, so the three rows are one case here. The rule tests below state each with the
    /// schema's own value.
    #[test]
    fn a_unit_marked_otherwise_is_refused_as_not_modified() {
        for unit in [UnitKind::Requirement, UnitKind::Scenario] {
            assert_eq!(
                verdict(
                    Placement::ChangeSpecification,
                    &inside(unit, Marked::Otherwise)
                ),
                Verdict::Refused(Refusal::NotModified),
                "{unit}"
            );
        }
    }

    /// Row "markup in a heading, or in a `renamed-from` paragraph".
    #[test]
    fn markup_in_a_name_is_refused() {
        for unit in [UnitKind::Requirement, UnitKind::Scenario] {
            assert_eq!(
                verdict(
                    Placement::ChangeSpecification,
                    &MarkupContext::Inside {
                        unit,
                        marked: Marked::Modified,
                        part: Part::Name,
                    }
                ),
                Verdict::Refused(Refusal::InAName),
                "{unit}"
            );
        }
    }

    /// Row "a block-level `<del><p>…</p></del>`": the element sits in the requirement's
    /// content, as a deleted word does, so it is permitted on the same terms. The rule test
    /// "A word and a paragraph deleted from a modified requirement" checks the tree.
    #[test]
    fn a_deleted_paragraph_in_a_modified_requirement_is_permitted() {
        assert_eq!(
            verdict(
                Placement::ChangeSpecification,
                &inside(UnitKind::Requirement, Marked::Modified)
            ),
            Verdict::Permitted
        );
    }

    /// Row "`del` or `ins` enclosing a unit", at every placement.
    #[test]
    fn markup_enclosing_a_unit_is_refused_everywhere() {
        for placement in every_placement() {
            assert_eq!(
                verdict(placement, &MarkupContext::EnclosesAUnit),
                Verdict::Refused(Refusal::EnclosesAUnit),
                "{placement:?}"
            );
        }
    }

    /// Row "`del` or `ins` whose own class names a unit", at every placement.
    #[test]
    fn markup_that_is_a_unit_is_refused_everywhere() {
        for placement in every_placement() {
            assert_eq!(
                verdict(placement, &MarkupContext::IsAUnit),
                Verdict::Refused(Refusal::IsAUnit),
                "{placement:?}"
            );
        }
    }

    /// Row "a decision, evidence, risk, or verification unit marked `modified`".
    #[test]
    fn a_unit_no_accepted_specification_holds_is_refused() {
        for unit in [
            UnitKind::Decision,
            UnitKind::Evidence,
            UnitKind::Risk,
            UnitKind::Verification,
        ] {
            assert_eq!(
                verdict(
                    Placement::ChangeSpecification,
                    &inside(unit, Marked::Modified)
                ),
                Verdict::Refused(Refusal::NotMarkable(unit)),
                "{unit}"
            );
        }
    }

    /// Row "a task".
    #[test]
    fn a_task_is_refused() {
        assert_eq!(
            verdict(
                Placement::ChangeSpecification,
                &inside(UnitKind::Task, Marked::Modified)
            ),
            Verdict::Refused(Refusal::NotMarkable(UnitKind::Task))
        );
    }

    /// Row "a document outside any project", and the `Loose` placement.
    #[test]
    fn a_loose_document_is_refused() {
        assert_eq!(
            verdict(
                Placement::Loose,
                &inside(UnitKind::Requirement, Marked::Modified)
            ),
            Verdict::Refused(Refusal::NotAChangeSpecification(Placement::Loose))
        );
    }

    /// `ChangeDocument`: a change's proposal, design, or tasks.
    #[test]
    fn a_change_document_is_refused() {
        assert_eq!(
            verdict(
                Placement::ChangeDocument,
                &inside(UnitKind::Requirement, Marked::Modified)
            ),
            Verdict::Refused(Refusal::NotAChangeSpecification(Placement::ChangeDocument))
        );
    }

    /// `ChangeSpecification`: the one placement that permits it.
    #[test]
    fn a_change_specification_permits_it() {
        assert_eq!(
            verdict(
                Placement::ChangeSpecification,
                &inside(UnitKind::Requirement, Marked::Modified)
            ),
            Verdict::Permitted
        );
    }

    /// `AcceptedSpecification`: the merge leaves no mark there, and a mark left is refused.
    #[test]
    fn an_accepted_specification_is_refused() {
        assert_eq!(
            verdict(
                Placement::AcceptedSpecification,
                &inside(UnitKind::Requirement, Marked::Modified)
            ),
            Verdict::Refused(Refusal::NotAChangeSpecification(
                Placement::AcceptedSpecification
            ))
        );
    }

    /// `RunbookDocument`: a runbook's documents restate no accepted text.
    #[test]
    fn a_runbook_document_is_refused() {
        assert_eq!(
            verdict(
                Placement::RunbookDocument,
                &inside(UnitKind::Requirement, Marked::Modified)
            ),
            Verdict::Refused(Refusal::NotAChangeSpecification(Placement::RunbookDocument))
        );
    }

    /// The refusals are tried in the design's order, so where two apply the first is given: a
    /// task's heading is a task, and a modified requirement's heading in an accepted
    /// specification is told to state the text as it now reads, not to use `renamed-from`.
    #[test]
    fn the_first_refusal_that_applies_is_given() {
        assert_eq!(
            verdict(
                Placement::Loose,
                &MarkupContext::Inside {
                    unit: UnitKind::Task,
                    marked: Marked::Otherwise,
                    part: Part::Name,
                }
            ),
            Verdict::Refused(Refusal::NotMarkable(UnitKind::Task))
        );
        assert_eq!(
            verdict(
                Placement::AcceptedSpecification,
                &MarkupContext::Inside {
                    unit: UnitKind::Requirement,
                    marked: Marked::Otherwise,
                    part: Part::Name,
                }
            ),
            Verdict::Refused(Refusal::NotAChangeSpecification(
                Placement::AcceptedSpecification
            ))
        );
        assert_eq!(
            verdict(
                Placement::ChangeSpecification,
                &MarkupContext::Inside {
                    unit: UnitKind::Requirement,
                    marked: Marked::Otherwise,
                    part: Part::Name,
                }
            ),
            Verdict::Refused(Refusal::InAName)
        );
    }

    // -------------------------------------------------------------------------
    // The rule over whole documents at a placement: the requirement's scenarios
    // -------------------------------------------------------------------------

    /// Runs this rule alone over a document of `kind` sitting at `placement`.
    fn check_at(placement: Placement, kind: &str, body: &str) -> crate::Report {
        let outcome = parse_document(&document(kind, "0.1.0", body));
        let projection = project(&outcome.document);
        let scope =
            crate::references::Scope::gather(placement, |_| crate::references::Declared::default());
        run(
            &outcome,
            &projection,
            &htmlspec_assets::Store::Absent,
            &scope,
            &[validate],
            Strictness::Default,
        )
    }

    /// Runs this rule over a change's specification.
    fn in_change_specification(body: &str) -> crate::Report {
        check_at(Placement::ChangeSpecification, "specification", body)
    }

    /// A requirement marked `state`, whose statement is `statement`, followed by `extra` and an
    /// unchanged scenario.
    fn requirement_marked(state: &str, statement: &str, extra: &str) -> String {
        requirement(statement, extra).replace(
            "data-change=\"modified\"",
            &format!("data-change=\"{state}\""),
        )
    }

    /// The paragraph `sync` was inserted into, as the requirement's scenario states it.
    const SYNC: &str = "The spellings SHALL be fixed: <code>list</code>, <code>status</code>, \
         <ins><code>sync</code>, </ins><code>instructions</code>, and <code>store</code> name the \
         operations OpenSpec also provides.";

    /// A real edit from this repository's history: a word deleted and a phrase inserted.
    const RUNBOOK: &str = "That directory SHALL hold the project's accepted specifications, its \
         active changes, <del>and </del>its archive, <ins>and its runbooks, </ins>each in its own \
         place.";

    /// "A deleted phrase and an inserted word in one statement": in an accepted specification,
    /// one error for each element, each naming the requirement.
    #[test]
    fn scenario_a_deleted_phrase_and_an_inserted_word_in_an_accepted_specification() {
        let report = check_at(
            Placement::AcceptedSpecification,
            "specification",
            &requirement(
                &format!(
                    "{SPELLINGS}<del>nothing else</del><ins><code>store</code></ins> SHALL name them."
                ),
                "",
            ),
        );
        assert_eq!(
            located(&report),
            [Some("req-x"), Some("req-x")],
            "{}",
            messages(&report)
        );
        assert!(fired(&report).iter().all(|d| d.is_error()));
    }

    /// "A deleted paragraph in a requirement": a requirement marked `added` has no accepted
    /// paragraph to delete, even in a change's specification.
    #[test]
    fn scenario_a_deleted_paragraph_in_an_added_requirement() {
        let report = in_change_specification(&requirement_marked(
            "added",
            "The system SHALL work.",
            "<del><p>No other spelling SHALL name a command.</p></del>",
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// "An inserted word with nothing marked as removed": a unit marked `unchanged` states that
    /// none of its text changed.
    #[test]
    fn scenario_an_inserted_word_in_an_unchanged_unit() {
        let report = in_change_specification(&requirement_marked(
            "unchanged",
            "The system SHALL <ins>always</ins> work.",
            "",
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// "Change markup in a nested unit": the scenario marked `unchanged` decides, not the
    /// `modified` requirement around it.
    #[test]
    fn scenario_change_markup_in_an_unchanged_scenario_of_a_modified_requirement() {
        let scenario = "<article class=\"scenario\" id=\"scenario-y\" data-change=\"unchanged\">\
             <h3>S</h3><dl><dt>When</dt><dd>It <del>fails</del> happens.</dd>\
             <dt>Then</dt><dd>It works.</dd></dl></article>";
        let report = in_change_specification(&requirement("The system SHALL work.", scenario));
        assert_eq!(
            located(&report),
            [Some("scenario-y")],
            "{}",
            messages(&report)
        );
    }

    /// "A unit wrapped in change markup", in a change's specification too.
    #[test]
    fn scenario_a_unit_wrapped_in_change_markup_in_a_change_specification() {
        let report = in_change_specification(&format!(
            "<section><h2>Requirements</h2><del>{}</del></section>",
            requirement("The system SHALL work.", "")
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// "A change element that is itself a unit", in a change's specification and marked
    /// `modified`.
    #[test]
    fn scenario_a_change_element_that_is_itself_a_unit_in_a_change_specification() {
        let report = in_change_specification(&format!(
            "<del class=\"requirement\" id=\"req-x\" data-change=\"modified\"><h2>R</h2>\
             <p>The system SHALL work.</p>{SCENARIO}</del>"
        ));
        assert_eq!(located(&report), [Some("req-x")], "{}", messages(&report));
    }

    /// "Any unit of any kind": a decision, task, evidence, risk, or verification unit, marked
    /// `modified`, in every kind of document and at every placement.
    #[test]
    fn scenario_any_unit_of_any_kind_at_every_placement() {
        let units = [
            (
                "decision-x",
                "<section class=\"decision\" id=\"decision-x\" data-status=\"accepted\" \
                 data-change=\"modified\"><h2>D</h2><p>The decision <del>not</del>.</p>\
                 <p data-field=\"rationale\">Why.</p></section>",
            ),
            (
                "task-x",
                "<section class=\"task\" id=\"task-x\" data-state=\"todo\" \
                 data-change=\"modified\"><h2>T</h2><p>Write <del>the old</del> rule.</p>\
                 </section>",
            ),
            (
                "evidence-x",
                "<section class=\"evidence\" id=\"evidence-x\" data-change=\"modified\">\
                 <h2>E</h2><p>Measured <del>twice</del>.</p></section>",
            ),
            (
                "risk-x",
                "<section class=\"risk\" id=\"risk-x\" data-change=\"modified\"><h2>K</h2>\
                 <p>Fails.</p><p data-field=\"mitigation\"><ins>Held.</ins></p></section>",
            ),
            (
                "verify-x",
                "<section class=\"verification\" id=\"verify-x\" data-outcome=\"passed\" \
                 data-change=\"modified\"><h2>V</h2>\
                 <p data-field=\"method\">Ran it<del> once</del>.</p></section>",
            ),
        ];
        for placement in every_placement() {
            for kind in crate::schema_facts::artifact_kinds() {
                for (id, html) in units {
                    let report = check_at(placement, &kind, html);
                    assert_eq!(
                        located(&report),
                        [Some(id)],
                        "{placement:?}, {kind}: {}",
                        messages(&report)
                    );
                }
            }
        }
    }

    /// "A document declaring an earlier contract version": every accepted version gets the
    /// verdict the current one gets, refused outside a change's specification and permitted in
    /// a modified unit inside one.
    #[test]
    fn scenario_an_earlier_contract_version_gets_the_same_verdict() {
        for version in htmlspec_schema::facts::contract_versions() {
            let html = document(
                "specification",
                &version,
                &requirement("The system SHALL <del>never</del> work.", ""),
            );
            let outcome = parse_document(&html);
            let projection = project(&outcome.document);
            for placement in every_placement() {
                let scope = crate::references::Scope::gather(placement, |_| {
                    crate::references::Declared::default()
                });
                let report = run(
                    &outcome,
                    &projection,
                    &htmlspec_assets::Store::Absent,
                    &scope,
                    &[validate],
                    Strictness::Default,
                );
                let expected: &[Option<&str>] = if placement == Placement::ChangeSpecification {
                    &[]
                } else {
                    &[Some("req-x")]
                };
                assert_eq!(
                    located(&report),
                    expected,
                    "{version}, {placement:?}: {}",
                    messages(&report)
                );
            }
        }
    }

    /// "Change markup outside every unit": a change's proposal showing a wording change.
    #[test]
    fn scenario_markup_outside_every_unit_in_a_change_document() {
        let report = check_at(
            Placement::ChangeDocument,
            "proposal",
            "<section id=\"hs-what-changes\"><h2>What changes</h2>\
             <p>The spellings become <del>nothing else</del><ins><code>store</code></ins>.</p>\
             </section>",
        );
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// "A struck-through phrase is not change markup", in a change's specification and in an
    /// accepted one.
    #[test]
    fn scenario_a_struck_through_phrase_at_every_placement() {
        for placement in every_placement() {
            let report = check_at(
                placement,
                "specification",
                &requirement("The system SHALL <s>never</s> work.", ""),
            );
            assert!(
                fired(&report).is_empty(),
                "{placement:?}: {}",
                messages(&report)
            );
        }
    }

    /// "A word inserted into a modified requirement".
    #[test]
    fn scenario_a_word_inserted_into_a_modified_requirement() {
        let report = in_change_specification(&requirement(SYNC, ""));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// "A word and a paragraph deleted from a modified requirement".
    #[test]
    fn scenario_a_word_and_a_paragraph_deleted_from_a_modified_requirement() {
        let report = in_change_specification(&requirement(
            RUNBOOK,
            "<del><p>No other place SHALL hold a document.</p></del>",
        ));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// "A modified scenario in a modified requirement".
    #[test]
    fn scenario_a_modified_scenario_in_a_modified_requirement() {
        let scenario = "<article class=\"scenario\" id=\"scenario-y\" data-change=\"modified\">\
             <h3>S</h3><dl><dt>When</dt><dd>A contributor <del>lists</del><ins>opens</ins> the \
             project.</dd><dt>Then</dt><dd>It works.</dd></dl></article>";
        let report = in_change_specification(&requirement("The system SHALL work.", scenario));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// "An archived change keeps its marks". An archived change's specification is at
    /// `ChangeSpecification` like an active one's, so the rule gives it the same verdict; the
    /// command line's tests check that the placement is derived so for an archived change.
    #[test]
    fn scenario_an_archived_change_keeps_its_marks() {
        let report = in_change_specification(&format!(
            "{}{}",
            requirement(RUNBOOK, ""),
            requirement(SYNC, "")
                .replace("req-x", "req-y")
                .replace("scenario-x", "scenario-z")
        ));
        assert!(fired(&report).is_empty(), "{}", messages(&report));
    }

    /// "The same markup outside a change's specification": an accepted specification, a
    /// change's other documents, a runbook's documents, a change's evidence, and a document
    /// outside any project each report every element.
    #[test]
    fn scenario_the_same_markup_outside_a_change_specification() {
        for placement in every_placement() {
            if placement == Placement::ChangeSpecification {
                continue;
            }
            let report = check_at(placement, "specification", &requirement(RUNBOOK, ""));
            assert_eq!(
                located(&report),
                [Some("req-x"), Some("req-x")],
                "{placement:?}: {}",
                messages(&report)
            );
        }
    }

    /// "Change markup in a unit's heading": refused, and the diagnostic points to `renamed-from`.
    /// Markup in the `renamed-from` paragraph itself is refused too.
    #[test]
    fn scenario_change_markup_in_a_units_heading() {
        let heading = in_change_specification(&requirement("The system SHALL work.", "").replace(
            "<h2>The spellings are fixed",
            "<h2>The spellings are <del>open</del> fixed",
        ));
        let found = fired(&heading);
        assert_eq!(located(&heading), [Some("req-x")], "{}", messages(&heading));
        assert!(
            found[0].message.contains("renamed-from"),
            "{}",
            found[0].message
        );

        let renamed = in_change_specification(&requirement(
            "The system SHALL work.",
            "<p data-field=\"renamed-from\">The spellings are <del>open</del></p>",
        ));
        assert_eq!(located(&renamed), [Some("req-x")], "{}", messages(&renamed));
    }

    /// Each refusal's message names its fix: the unit's change state for markup around a unit,
    /// `dropped` for a task, the current text for a document that is not a change's
    /// specification, `renamed-from` for a heading, and the unit's own state for a unit not
    /// marked `modified`.
    #[test]
    fn each_refusal_names_its_fix() {
        let message = |report: &crate::Report| -> String {
            let found = fired(report);
            assert_eq!(found.len(), 1, "{}", messages(report));
            found[0].message.clone()
        };

        let around = message(&in_change_specification(&format!(
            "<del>{}</del>",
            requirement("The system SHALL work.", "")
        )));
        assert!(around.contains("data-change=\"removed\""), "{around}");

        let task = message(&in_change_specification(
            "<section class=\"task\" id=\"task-x\" data-state=\"todo\"><h2>T</h2>\
             <p>Write <del>the old</del> rule.</p></section>",
        ));
        assert!(task.contains("dropped"), "{task}");

        let accepted = message(&check_at(
            Placement::AcceptedSpecification,
            "specification",
            &requirement("The system SHALL <del>never</del> work.", ""),
        ));
        assert!(
            accepted.contains("as it now reads")
                && !accepted.contains("data-change=\"modified\"")
                && !accepted.contains("renamed-from"),
            "{accepted}"
        );

        let heading = message(&in_change_specification(
            &requirement("The system SHALL work.", "").replace(
                "<h2>The spellings are fixed",
                "<h2>The spellings are <del>open</del> fixed",
            ),
        ));
        assert!(heading.contains("renamed-from"), "{heading}");

        let added = message(&in_change_specification(&requirement_marked(
            "added",
            "The system SHALL <del>never</del> work.",
            "",
        )));
        assert!(
            added.contains("data-change=\"added\"") && added.contains("data-change=\"modified\""),
            "{added}"
        );
    }
}
