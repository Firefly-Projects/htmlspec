//! Projects a parsed HTML tree into the document model.
//!
//! This module implements `doc/projection.md` of the `htmlspec-schema` crate.
//! Section numbers in the comments below refer to that specification, which is
//! normative: where the code and the specification disagree, the specification
//! is right and the code is a defect.
//!
//! Two rules govern everything here. First, the projection produces the model
//! and does not judge it (section 10.1): a wrong prefix, an unknown change
//! state, and a missing statement all survive into the model so that the
//! runtime validator reports them against a schema location. Second, where the
//! specification says a construct is *invalid*, the projection cannot produce a
//! model for it and says so through [`ProjectionFault`] (section 10.2) rather
//! than picking one reading.

use kuchikiki::iter::NodeIterator;
use kuchikiki::{ElementData, NodeData, NodeRef};

pub mod marks;

use crate::model::{
    Decision, DocumentModel, Evidence, Figure, FigureKind, Link, Reference, ReferenceOrigin,
    RenamedFrom, Requirement, Risk, Scenario, Task, UnitKind, Verification, DEFAULT_CHANGE_STATE,
    DEFAULT_RISK_RESPONSE, DEVIATION_FIELD, RENAMED_FROM_FIELD,
};

/// A construct the projection cannot produce a model for.
///
/// The specification's word for this is *invalid*. Each fault names the
/// projection rule it violates, so the validator can report a `schema/*`
/// diagnostic that points at a section rather than at a line of Rust.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionFault {
    /// The projection rule violated, as its section number, such as `1.3`.
    pub rule: &'static str,
    /// What the fault is, in a form a document author can act on.
    pub detail: String,
    /// The identifier of the unit the fault was found in, when it declared one.
    pub unit_id: Option<String>,
}

impl ProjectionFault {
    fn new(rule: &'static str, detail: impl Into<String>) -> Self {
        Self {
            rule,
            detail: detail.into(),
            unit_id: None,
        }
    }

    fn in_unit(mut self, id: Option<&str>) -> Self {
        self.unit_id = id.map(str::to_owned);
        self
    }
}

impl std::fmt::Display for ProjectionFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "projection {}: {}", self.rule, self.detail)?;
        if let Some(id) = &self.unit_id {
            write!(f, " (in {id})")?;
        }
        Ok(())
    }
}

/// The result of projecting one document.
///
/// Faults accumulate rather than short-circuiting, because the validator reports
/// every violation it can detect in one pass. A document with faults still
/// carries a model: the units the projection *could* read are present, so a
/// second fault elsewhere is still reported.
#[derive(Clone, Debug)]
pub struct Projection {
    /// The model, holding every unit the projection could read.
    pub model: DocumentModel,
    /// Every rule of the projection specification the document violates.
    pub faults: Vec<ProjectionFault>,
    /// The parsed tree the model was projected from.
    ///
    /// Most rules read the model, because a rule about a semantic unit should not
    /// depend on markup incidentals. The safety rules are the exception and need
    /// the tree: a remote script is not a semantic unit and a `script` element
    /// projects no text at all (section 8.5), so a validator that only ever saw
    /// the model would report a document that reaches the network as clean.
    pub document: Option<NodeRef>,
    /// Every unit other than a requirement that carries a `renamed-from` paragraph.
    ///
    /// Section 4.1.1: such a paragraph projects nowhere, and section 4.0 does not
    /// report it as a field the unit lacks, because the validator reports it under
    /// a rule of its own. So the fact that it was there has to leave the
    /// projection somewhere other than the model, which has no place for it.
    pub misplaced_renames: Vec<MisplacedRename>,
}

/// A unit that is not a requirement and carries a `renamed-from` paragraph.
///
/// One per unit, however many such paragraphs it holds: the fault is that this
/// unit states a rename at all, and a second paragraph does not make it a second
/// fault.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MisplacedRename {
    /// Which unit carries it.
    pub unit: UnitKind,
    /// The unit's identifier, when it declared one.
    pub unit_id: Option<String>,
}

impl Projection {
    /// Whether the document projected with no fault at all.
    pub fn is_clean(&self) -> bool {
        self.faults.is_empty()
    }
}

/// Projects a parsed document into the model.
pub fn project(document: &NodeRef) -> Projection {
    let mut faults: Vec<ProjectionFault> = Vec::new();
    let mut misplaced_renames: Vec<MisplacedRename> = Vec::new();
    let mut model = DocumentModel {
        schema_version: meta_content(document, "htmlspec-version"),
        subject: first_element(document, "title").and_then(|t| normalized_text_opt(&t)),
        summary: header_summary(document),
        // Section 9.3. The same mechanism as the version above: `meta_content`
        // reads a named meta element's content, and the field being an enumeration
        // rather than a version changes nothing about how it is read. An absent
        // kind projects as absent, and the validator reports the missing property.
        kind: meta_content(document, "htmlspec-kind"),
        ..DocumentModel::default()
    };

    // Section 9.4: depth-first document order, which is the tree walk's order.
    for element in unit_elements(document, &mut faults) {
        // Section 4.1.1: a rename is a requirement's field. Recorded here for every
        // other unit, whatever else is wrong with it, so the validator can report
        // it once under its own rule.
        if element.kind != UnitKind::Requirement && !renamed_from(&element.node).is_empty() {
            misplaced_renames.push(MisplacedRename {
                unit: element.kind,
                unit_id: attribute(&element.node, "id"),
            });
        }
        match element.kind {
            UnitKind::Requirement => {
                // Section 4.3: a requirement nested inside another is invalid.
                if enclosing_unit(&element.node).is_some() {
                    faults.push(
                        ProjectionFault::new(
                            "4.3",
                            "a requirement is nested inside another unit, and no such nesting \
                             is defined",
                        )
                        .in_unit(attribute(&element.node, "id").as_deref()),
                    );
                    continue;
                }
                model
                    .requirements
                    .push(project_requirement(&element.node, &mut faults));
            }
            UnitKind::Scenario => {
                // Section 4.1 projects a scenario through its requirement, so a
                // scenario reached here is one that has no requirement to belong
                // to. Section 4.3 makes that invalid.
                if !encloses_requirement(&element.node) {
                    faults.push(
                        ProjectionFault::new(
                            "4.3",
                            "a scenario appears outside a requirement, and a scenario exists \
                             only inside one",
                        )
                        .in_unit(attribute(&element.node, "id").as_deref()),
                    );
                }
            }
            UnitKind::Decision => model
                .decisions
                .push(project_decision(&element.node, &mut faults)),
            UnitKind::Task => model.tasks.push(project_task(&element.node, &mut faults)),
            UnitKind::Evidence => model
                .evidence
                .push(project_evidence(&element.node, &mut faults)),
            UnitKind::Risk => model.risks.push(project_risk(&element.node, &mut faults)),
            UnitKind::Verification => model
                .verifications
                .push(project_verification(&element.node, &mut faults)),
        }
    }

    for figure in descendant_elements(document, "figure") {
        model.figures.push(project_figure(&figure, &mut faults));
    }
    model.links = document_links(document);

    Projection {
        model,
        faults,
        document: Some(document.clone()),
        misplaced_renames,
    }
}

// ---------------------------------------------------------------------------
// Section 1: what creates a unit
// ---------------------------------------------------------------------------

/// One element that creates a unit, with the unit it creates.
struct UnitElement {
    kind: UnitKind,
    node: NodeRef,
}

/// Every element that creates a unit, in document order.
///
/// Section 1.1: an element creates a unit when, and only when, its `class`
/// attribute holds one of the seven reserved tokens. Section 1.3: two reserved
/// tokens on one element is invalid, and the projection does not choose one — so
/// such an element creates nothing and is reported.
fn unit_elements(root: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Vec<UnitElement> {
    let mut found = Vec::new();
    for node in root.inclusive_descendants().elements() {
        let node = node.as_node().clone();
        let mut kinds = reserved_classes(&node);
        match kinds.len() {
            0 => {}
            1 => found.push(UnitElement {
                kind: kinds.remove(0),
                node,
            }),
            _ => {
                let names: Vec<&str> = kinds.iter().map(|k| k.as_class()).collect();
                faults.push(
                    ProjectionFault::new(
                        "1.3",
                        format!(
                            "an element carries two reserved class names ({}), and the \
                             projection does not choose one",
                            names.join(", ")
                        ),
                    )
                    .in_unit(attribute(&node, "id").as_deref()),
                );
            }
        }
    }
    found
}

/// The reserved class tokens on one element, in class-attribute order.
///
/// Section 1.2: a token is matched whole and case sensitively, so
/// `class="requirement note"` matches and `class="requirements"` does not.
fn reserved_classes(node: &NodeRef) -> Vec<UnitKind> {
    attribute(node, "class")
        .map(|value| {
            value
                .split_whitespace()
                .filter_map(UnitKind::from_class)
                .collect()
        })
        .unwrap_or_default()
}

/// The nearest ancestor that creates a unit, if any.
fn enclosing_unit(node: &NodeRef) -> Option<NodeRef> {
    node.ancestors()
        .find(|ancestor| !reserved_classes(ancestor).is_empty())
}

/// Whether the nearest enclosing unit is a requirement.
fn encloses_requirement(node: &NodeRef) -> bool {
    enclosing_unit(node)
        .map(|unit| reserved_classes(&unit).contains(&UnitKind::Requirement))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Section 2: attributes that project
// ---------------------------------------------------------------------------

/// One attribute's value, whitespace-trimmed per section 2.3.
///
/// No other normalization applies: `data-state="Done"` projects as `Done` and
/// the validator rejects it, because the projection does not judge the model.
fn attribute(node: &NodeRef, name: &str) -> Option<String> {
    let element: &ElementData = node.as_element()?;
    let attributes = element.attributes.borrow();
    attributes.get(name).map(|v| v.trim().to_owned())
}

/// The unit's change state, defaulting per section 2 when absent.
fn change_state(node: &NodeRef) -> String {
    attribute(node, "data-change").unwrap_or_else(|| DEFAULT_CHANGE_STATE.to_owned())
}

/// References from the `data-refs` attribute.
///
/// Section 2.1: a whitespace-separated list, order preserved, a repeated
/// identifier projecting once at its first position.
fn attribute_references(node: &NodeRef) -> Vec<Reference> {
    let Some(value) = attribute(node, "data-refs") else {
        return Vec::new();
    };
    let mut seen: Vec<&str> = Vec::new();
    let mut references = Vec::new();
    for target in value.split_whitespace() {
        if seen.contains(&target) {
            continue;
        }
        seen.push(target);
        references.push(Reference {
            target: target.to_owned(),
            origin: ReferenceOrigin::Attribute,
        });
    }
    references
}

// ---------------------------------------------------------------------------
// Section 3: fields common to every unit
// ---------------------------------------------------------------------------

/// The unit's title, per sections 3.1 to 3.3.
///
/// The title is the first heading among the unit's own descendants. A second
/// heading at the same depth is invalid (3.2), and no heading at all is invalid
/// (3.3) — reported here rather than left to the schema, because the schema
/// cannot see how many headings the HTML had.
fn project_title(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Option<String> {
    let mut headings: Vec<(usize, NodeRef)> = Vec::new();
    for node in own_descendants(unit) {
        if let Some(depth) = heading_depth(unit, &node) {
            headings.push((depth, node));
        }
    }

    let Some((first_depth, first)) = headings.first().cloned() else {
        faults.push(
            ProjectionFault::new("3.3", "a unit has no heading, so it has no name")
                .in_unit(attribute(unit, "id").as_deref()),
        );
        return None;
    };

    // Section 3.2: depth is measured relative to the unit, so a heading inside a
    // presentational wrapper is deeper and does not make the name ambiguous.
    let siblings = headings
        .iter()
        .filter(|(depth, _)| *depth == first_depth)
        .count();
    if siblings > 1 {
        faults.push(
            ProjectionFault::new(
                "3.2",
                format!("a unit contains {siblings} headings at the same depth, so its name is ambiguous"),
            )
            .in_unit(attribute(unit, "id").as_deref()),
        );
    }

    normalized_text_opt(&first)
}

/// The depth of a heading element relative to `unit`, or `None` if it is not a
/// heading.
fn heading_depth(unit: &NodeRef, node: &NodeRef) -> Option<usize> {
    let name = element_name(node)?;
    let is_heading = matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6");
    if !is_heading {
        return None;
    }
    Some(
        node.ancestors()
            .take_while(|ancestor| ancestor != unit)
            .count(),
    )
}

// ---------------------------------------------------------------------------
// Section 4: fields specific to one unit
// ---------------------------------------------------------------------------

/// The paragraphs of one unit, split by their `data-field` attribute.
///
/// Section 4.0: an unmarked paragraph supplies the unit's primary field, and a
/// marked one supplies the field it names. A `data-field` value naming no field
/// of this unit is invalid, so the caller passes the names it accepts.
struct Paragraphs {
    /// Unmarked paragraphs, in document order.
    unmarked: Vec<String>,
    /// Marked paragraphs, as (field, text) in document order.
    marked: Vec<(String, String)>,
}

impl Paragraphs {
    /// Every marked paragraph naming `field`, in document order.
    fn field(&self, field: &str) -> Vec<String> {
        self.marked
            .iter()
            .filter(|(name, _)| name == field)
            .map(|(_, text)| text.clone())
            .collect()
    }

    /// The first paragraph naming `field`, or `None`.
    ///
    /// A repeated single-valued field takes the first, and this reports nothing.
    /// Only a task's `deviation` is a fault when repeated (section 4.5.1), and
    /// [`project_task`] reports that itself. A repeated `reason`, `mitigation`, or
    /// `method` is not yet a fault anywhere in the specification, so for those the
    /// second paragraph is dropped without a diagnostic.
    fn single(&self, field: &str) -> Option<String> {
        self.field(field).into_iter().next()
    }
}

/// Collects the paragraphs of one unit and validates their `data-field` names.
///
/// `fields` is every `data-field` value this unit accepts. `note` is accepted by
/// every unit (section 4.0) and does not need listing.
fn project_paragraphs(
    unit: &NodeRef,
    fields: &[&str],
    faults: &mut Vec<ProjectionFault>,
) -> Paragraphs {
    let mut collected = Paragraphs {
        unmarked: Vec::new(),
        marked: Vec::new(),
    };
    for node in own_descendants(unit) {
        if element_name(&node).as_deref() != Some("p") {
            continue;
        }
        let field = attribute(&node, "data-field");
        // Section 4.1.1: collected by `renamed_from` on every unit, including an
        // empty one, and reported by the validator rather than by section 4.0
        // when the unit is not a requirement. Checked before the emptiness test
        // below, which would otherwise discard an empty one.
        if field.as_deref() == Some(RENAMED_FROM_FIELD) {
            continue;
        }
        let Some(text) = normalized_text_opt(&node) else {
            continue;
        };
        match field {
            None => collected.unmarked.push(text),
            Some(field) if field == "note" || fields.contains(&field.as_str()) => {
                collected.marked.push((field, text));
            }
            Some(field) => faults.push(
                ProjectionFault::new(
                    "4.0",
                    format!(
                        "data-field=\"{field}\" names no field of this unit; it accepts {}",
                        accepted_fields(fields)
                    ),
                )
                .in_unit(attribute(unit, "id").as_deref()),
            ),
        }
    }
    collected
}

fn accepted_fields(fields: &[&str]) -> String {
    let mut names: Vec<String> = fields.iter().map(|f| format!("\"{f}\"")).collect();
    names.push("\"note\"".to_owned());
    names.join(", ")
}

/// A repeatable primary field: every unmarked paragraph plus every paragraph
/// marked with the field's own name, in document order.
///
/// Section 4.1 for a requirement's `statements` and section 4.6 for evidence's
/// `summary`. Document order across both sources is what the specification
/// requires, so this walks the tree once rather than concatenating two lists.
///
/// An unrecognized `data-field` is reported by [`project_paragraphs`], which the
/// caller also runs; this function only collects, so a paragraph naming a field
/// this unit does not accept contributes nothing here and is reported there.
fn repeatable_primary(unit: &NodeRef, field: &str) -> Vec<String> {
    own_descendants(unit)
        .iter()
        .filter(|node| element_name(node).as_deref() == Some("p"))
        .filter(|node| match attribute(node, "data-field") {
            None => true,
            Some(name) => name == field,
        })
        .filter_map(normalized_text_opt)
        .collect()
}

/// A single-valued primary field: the one unmarked paragraph, or the one
/// paragraph marked with the field's own name.
///
/// Section 4.0.1: two unmarked paragraphs where the primary field is
/// single-valued is invalid, because the projection does not choose one.
fn single_primary(
    unit: &NodeRef,
    field: &str,
    paragraphs: &Paragraphs,
    faults: &mut Vec<ProjectionFault>,
) -> Option<String> {
    if paragraphs.unmarked.len() > 1 {
        faults.push(
            ProjectionFault::new(
                "4.0.1",
                format!(
                    "a unit whose primary field \"{field}\" is single-valued contains {} \
                     unmarked paragraphs, and the projection does not choose one",
                    paragraphs.unmarked.len()
                ),
            )
            .in_unit(attribute(unit, "id").as_deref()),
        );
        return None;
    }
    paragraphs
        .unmarked
        .first()
        .cloned()
        .or_else(|| paragraphs.single(field))
}

fn project_requirement(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Requirement {
    let id = attribute(unit, "id");
    let paragraphs = project_paragraphs(unit, &["statement"], faults);
    // Section 8.6.1: a scenario inside a `del` still projects, so scenarios are found among every
    // nested unit and not only among the descendants a field rule reads.
    let scenarios = nested_units(unit)
        .into_iter()
        .filter(|node| reserved_classes(node) == vec![UnitKind::Scenario])
        .map(|node| project_scenario(&node, faults))
        .collect();
    Requirement {
        unit: UnitKind::Requirement,
        title: project_title(unit, faults),
        change: change_state(unit),
        statements: repeatable_primary(unit, "statement"),
        scenarios,
        notes: paragraphs.field("note"),
        renamed_from: renamed_from(unit),
        deletions: own_deletions(unit),
        id,
    }
}

/// Every `data-field="renamed-from"` paragraph among the unit's own descendants,
/// in document order.
///
/// Section 4.1.1. An empty paragraph is kept, as [`RenamedFrom::Empty`], where
/// every other empty paragraph is discarded. A rename stated without its heading
/// is a fault the validator reports, and discarding it here would turn that fault
/// into a requirement that was never renamed.
fn renamed_from(unit: &NodeRef) -> Vec<RenamedFrom> {
    own_descendants(unit)
        .iter()
        .filter(|node| element_name(node).as_deref() == Some("p"))
        .filter(|node| attribute(node, "data-field").as_deref() == Some(RENAMED_FROM_FIELD))
        .map(|node| normalized_text_opt(node).map_or(RenamedFrom::Empty, RenamedFrom::Heading))
        .collect()
}

fn project_scenario(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Scenario {
    let id = attribute(unit, "id");
    let paragraphs = project_paragraphs(unit, &[], faults);
    let entries = project_scenario_body(unit, faults);
    Scenario {
        unit: UnitKind::Scenario,
        title: project_title(unit, faults),
        change: change_state(unit),
        given: entries.given,
        when: entries.when,
        then: entries.then,
        notes: {
            // Section 4.0.2: a scenario has no primary field, so an unmarked
            // paragraph is a note rather than discarded content.
            let mut notes = paragraphs.unmarked.clone();
            notes.extend(paragraphs.field("note"));
            notes
        },
        deletions: own_deletions(unit),
        id,
    }
}

/// The `given`, `when`, and `then` entries of one scenario.
#[derive(Default)]
struct ScenarioBody {
    given: Vec<String>,
    when: Vec<String>,
    then: Vec<String>,
}

/// Projects a scenario's description list, per section 4.2.
///
/// Each `dt` supplies a label and each following `dd` supplies one entry. The
/// label is matched case-insensitively after trimming and after removing one
/// trailing colon. A label outside the set, a `dt` with no `dd`, and a `dd` with
/// no `dt` are each invalid.
fn project_scenario_body(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> ScenarioBody {
    let mut body = ScenarioBody::default();
    let id = attribute(unit, "id");
    let Some(list) = own_descendants(unit)
        .into_iter()
        .find(|node| element_name(node).as_deref() == Some("dl"))
    else {
        return body;
    };

    // The label state has three cases, and collapsing the last two would report
    // a second, false fault: an entry under an unrecognized label has a `dt`, so
    // saying it has none would send an author looking for the wrong defect.
    enum Label {
        /// No `dt` has been seen yet in this list.
        None,
        /// The last `dt` named a label outside the set. Already reported.
        Unrecognized,
        /// The last `dt` named this label.
        Named(ScenarioField),
    }

    let mut label = Label::None;
    let mut label_had_entry = false;
    for node in list.children().filter(|n| n.as_element().is_some()) {
        match element_name(&node).as_deref() {
            Some("dt") => {
                if !matches!(label, Label::None) && !label_had_entry {
                    faults.push(
                        ProjectionFault::new("4.2", "a dt has no following dd")
                            .in_unit(id.as_deref()),
                    );
                }
                label = match scenario_label(&node) {
                    Some(field) => Label::Named(field),
                    None => {
                        faults.push(
                            ProjectionFault::new(
                                "4.2",
                                format!(
                                    "a scenario label {:?} is outside the set Given, When, Then",
                                    normalized_text_opt(&node).unwrap_or_default()
                                ),
                            )
                            .in_unit(id.as_deref()),
                        );
                        Label::Unrecognized
                    }
                };
                label_had_entry = false;
            }
            Some("dd") => {
                label_had_entry = true;
                let Some(text) = normalized_text_opt(&node) else {
                    continue;
                };
                match label {
                    Label::Named(ScenarioField::Given) => body.given.push(text),
                    Label::Named(ScenarioField::When) => body.when.push(text),
                    Label::Named(ScenarioField::Then) => body.then.push(text),
                    // The unrecognized label was reported at its own `dt`. Its
                    // entries project nowhere rather than into a guessed field.
                    Label::Unrecognized => {}
                    Label::None => faults.push(
                        ProjectionFault::new("4.2", "a dd has no preceding dt")
                            .in_unit(id.as_deref()),
                    ),
                }
            }
            _ => {}
        }
    }
    if !matches!(label, Label::None) && !label_had_entry {
        faults.push(ProjectionFault::new("4.2", "a dt has no following dd").in_unit(id.as_deref()));
    }
    body
}

/// The field one scenario label names.
#[derive(Clone, Copy)]
enum ScenarioField {
    Given,
    When,
    Then,
}

/// The scenario label a `dt` carries, normalized per section 4.2.
///
/// Matched case-insensitively, after trimming and after removing *one* trailing
/// colon. `When::` keeps a colon and is therefore not a label.
fn scenario_label(node: &NodeRef) -> Option<ScenarioField> {
    let text = normalized_text_opt(node)?;
    let without_colon = text.strip_suffix(':').unwrap_or(&text);
    match without_colon.trim().to_lowercase().as_str() {
        "given" => Some(ScenarioField::Given),
        "when" => Some(ScenarioField::When),
        "then" => Some(ScenarioField::Then),
        _ => None,
    }
}

fn project_decision(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Decision {
    let id = attribute(unit, "id");
    let paragraphs = project_paragraphs(unit, &["statement", "rationale"], faults);
    Decision {
        unit: UnitKind::Decision,
        title: project_title(unit, faults),
        change: change_state(unit),
        statement: single_primary(unit, "statement", &paragraphs, faults),
        rationale: paragraphs.field("rationale"),
        status: attribute(unit, "data-status"),
        superseded_by: attribute(unit, "data-superseded-by"),
        references: unit_references(unit),
        notes: paragraphs.field("note"),
        id,
    }
}

fn project_task(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Task {
    let id = attribute(unit, "id");
    let paragraphs = project_paragraphs(unit, &["reason", DEVIATION_FIELD], faults);
    let title = project_title(unit, faults);
    // Section 4.5.1: one account of what was done. The first is kept rather than
    // none, because a consumer reading an unvalidated projection would otherwise
    // read a task its author said was done differently as done as written.
    let deviations = paragraphs.field(DEVIATION_FIELD);
    if deviations.len() > 1 {
        faults.push(
            ProjectionFault::new(
                "4.5.1",
                format!(
                    "a task contains {} deviation paragraphs, and the projection does not \
                     choose between two accounts of what was done",
                    deviations.len()
                ),
            )
            .in_unit(id.as_deref()),
        );
    }
    Task {
        unit: UnitKind::Task,
        title,
        change: change_state(unit),
        state: attribute(unit, "data-state"),
        reason: paragraphs.single("reason"),
        deviation: deviations.into_iter().next(),
        references: unit_references(unit),
        notes: {
            // Section 4.5: a task has no primary field, so an unmarked paragraph
            // is a note rather than a reason that would mean nothing.
            let mut notes = paragraphs.unmarked.clone();
            notes.extend(paragraphs.field("note"));
            notes
        },
        id,
    }
}

fn project_evidence(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Evidence {
    let id = attribute(unit, "id");
    let paragraphs = project_paragraphs(unit, &["summary"], faults);
    Evidence {
        unit: UnitKind::Evidence,
        title: project_title(unit, faults),
        change: change_state(unit),
        summary: repeatable_primary(unit, "summary"),
        references: unit_references(unit),
        notes: paragraphs.field("note"),
        id,
    }
}

fn project_risk(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Risk {
    let id = attribute(unit, "id");
    let paragraphs = project_paragraphs(unit, &["statement", "mitigation"], faults);
    Risk {
        unit: UnitKind::Risk,
        title: project_title(unit, faults),
        change: change_state(unit),
        statement: single_primary(unit, "statement", &paragraphs, faults),
        mitigation: paragraphs.single("mitigation"),
        // Section 4.7.1: trimmed per section 2.3, and the default when absent.
        response: attribute(unit, "data-response")
            .unwrap_or_else(|| DEFAULT_RISK_RESPONSE.to_owned()),
        references: unit_references(unit),
        notes: paragraphs.field("note"),
        id,
    }
}

fn project_verification(unit: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Verification {
    let id = attribute(unit, "id");
    let paragraphs = project_paragraphs(unit, &["method", "unchecked"], faults);
    Verification {
        unit: UnitKind::Verification,
        title: project_title(unit, faults),
        change: change_state(unit),
        outcome: attribute(unit, "data-outcome"),
        method: paragraphs.single("method"),
        // Section 4.8: every unchecked paragraph, in document order. Whether a `partial`
        // verification carries one is the validator's to judge (section 10.1), so none is
        // recorded as none.
        unchecked: paragraphs.field("unchecked"),
        references: unit_references(unit),
        notes: {
            // Section 4.0.2: a verification has no primary field.
            let mut notes = paragraphs.unmarked.clone();
            notes.extend(paragraphs.field("note"));
            notes
        },
        id,
    }
}

// ---------------------------------------------------------------------------
// Section 5: references
// ---------------------------------------------------------------------------

/// Every reference one unit makes.
///
/// Section 5.5: `data-refs` entries first in attribute order, then link
/// references in document order.
fn unit_references(unit: &NodeRef) -> Vec<Reference> {
    let mut references = attribute_references(unit);
    for anchor in own_descendants(unit) {
        if element_name(&anchor).as_deref() != Some("a") {
            continue;
        }
        // Section 5.3: only a fragment matching the schema's `unitId` grammar is
        // a reference. Anything else is a link and nothing more.
        if let Some(target) = same_document_fragment(&anchor)
            .filter(|t| htmlspec_schema::facts::is_unit_identifier(t))
        {
            references.push(Reference {
                target,
                origin: ReferenceOrigin::Link,
            });
        }
    }
    references
}

/// Every internal fragment link in the document, in document order (section 5.2).
fn document_links(root: &NodeRef) -> Vec<Link> {
    descendant_elements(root, "a")
        .into_iter()
        .filter_map(|anchor| {
            let target = same_document_fragment(&anchor)?;
            Some(Link {
                target,
                text: normalized_text(&anchor),
            })
        })
        .collect()
}

/// The fragment of a same-document `href`, without its leading `#`.
///
/// Section 5.4: an absolute or protocol-relative `href` projects into neither
/// array, so it yields `None` here.
fn same_document_fragment(anchor: &NodeRef) -> Option<String> {
    let href = attribute(anchor, "href")?;
    let fragment = href.strip_prefix('#')?;
    if fragment.is_empty() {
        return None;
    }
    Some(fragment.to_owned())
}

// ---------------------------------------------------------------------------
// Section 7: figures
// ---------------------------------------------------------------------------

fn project_figure(figure: &NodeRef, faults: &mut Vec<ProjectionFault>) -> Figure {
    let id = attribute(figure, "id");
    // Section 8.6.1: an element inside a `del` within the figure supplies nothing to it.
    let current = |name: &str| -> Vec<NodeRef> {
        descendant_elements(figure, name)
            .into_iter()
            .filter(|node| !inside_deletion(node, figure))
            .collect()
    };
    let caption = current("figcaption").first().and_then(normalized_text_opt);
    if caption.is_none() {
        faults
            .push(ProjectionFault::new("7.2", "a figure has no figcaption").in_unit(id.as_deref()));
    }

    // Section 7.3: kind and source come from the figure's first `pre`.
    let (kind, source) = match current("pre").first() {
        Some(pre) => {
            let kind = match pre_class(pre).as_deref() {
                Some("mermaid") => FigureKind::Mermaid,
                Some("vega-lite") => FigureKind::VegaLite,
                _ => FigureKind::Code,
            };
            (kind, Some(preserved_text(pre)))
        }
        None if !current("math").is_empty() => (FigureKind::Math, None),
        None => (FigureKind::Prose, None),
    };

    Figure {
        id,
        kind,
        caption,
        source,
    }
}

/// The visualization class of a `pre`, when it carries one the table names.
fn pre_class(pre: &NodeRef) -> Option<String> {
    let value = attribute(pre, "class")?;
    value
        .split_whitespace()
        .find(|token| *token == "mermaid" || *token == "vega-lite")
        .map(str::to_owned)
}

// ---------------------------------------------------------------------------
// Sections 6, 8, 9: text, presentational markup, metadata
// ---------------------------------------------------------------------------

/// Where a descendant of a unit stands against the change markup within that unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Standing {
    /// No `del` between it and the unit, so a field rule reads it.
    Current,
    /// Inside a `del` within the unit, so it supplies no field (section 8.6.1).
    Deleted,
}

/// Every element descendant of `unit` that belongs to it rather than to a nested unit, in
/// document order, each with where it stands against the `del` elements within the unit.
///
/// Section 8.1: a field rule reaches through elements with no reserved class, so a paragraph
/// inside `<div class="callout">` still belongs to the enclosing unit. A descendant of a *nested*
/// unit belongs to that unit instead, and the nested unit itself is listed. Section 8.6.1: what
/// sits inside a `del` is still walked, so a unit inside one is still found, and is marked, so no
/// field rule reads it.
fn own_nodes(unit: &NodeRef) -> Vec<(NodeRef, Standing)> {
    let mut found = Vec::new();
    let mut queue: Vec<(NodeRef, Standing)> = unit
        .children()
        .map(|child| (child, Standing::Current))
        .collect();
    queue.reverse();
    while let Some((node, standing)) = queue.pop() {
        if node.as_element().is_none() {
            continue;
        }
        found.push((node.clone(), standing));
        if !reserved_classes(&node).is_empty() {
            // A nested unit's own descendants belong to it, not to this unit.
            continue;
        }
        let within = if is_deletion(&node) {
            Standing::Deleted
        } else {
            standing
        };
        let mut children: Vec<(NodeRef, Standing)> =
            node.children().map(|child| (child, within)).collect();
        children.reverse();
        queue.extend(children);
    }
    found
}

/// Every descendant of `unit` that belongs to it and that a field rule reads: those outside
/// every `del` within the unit (sections 8.1 and 8.6.1).
fn own_descendants(unit: &NodeRef) -> Vec<NodeRef> {
    own_nodes(unit)
        .into_iter()
        .filter(|(_, standing)| *standing == Standing::Current)
        .map(|(node, _)| node)
        .collect()
}

/// Every unit nested in `unit` and not in another nested unit, a `del` around it or not.
///
/// Section 8.6.1: a unit inside a `del` still projects.
fn nested_units(unit: &NodeRef) -> Vec<NodeRef> {
    own_nodes(unit)
        .into_iter()
        .map(|(node, _)| node)
        .filter(|node| !reserved_classes(node).is_empty())
        .collect()
}

/// The entries of a unit's `deletions` (section 8.6.2): the whole text of each `del` that belongs
/// to it and that no other `del` within it encloses, in document order, leaving out one whose text
/// is empty.
///
/// A `del` whose own class names a unit is that unit, and section 8.6.1 has it record no deletion.
/// The walk lists a nested unit's own element as current, so such an element is excluded here by
/// its class rather than by where it stands.
fn own_deletions(unit: &NodeRef) -> Vec<String> {
    own_nodes(unit)
        .into_iter()
        .filter(|(node, standing)| {
            *standing == Standing::Current && is_deletion(node) && reserved_classes(node).is_empty()
        })
        .filter_map(|(node, _)| {
            let mut raw = String::new();
            collect_text(&node, &mut raw, Reading::Whole);
            let text = raw.split_whitespace().collect::<Vec<_>>().join(" ");
            (!text.is_empty()).then_some(text)
        })
        .collect()
}

/// The namespace of HTML's elements. Section 8.6: an element named `del` inside SVG or MathML
/// belongs to that language and is not change markup.
const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// Whether `node` is HTML's `del` element, which marks text an edit removed (section 8.6).
fn is_deletion(node: &NodeRef) -> bool {
    node.as_element().is_some_and(is_deletion_element)
}

fn is_deletion_element(element: &ElementData) -> bool {
    &*element.name.ns == HTML_NAMESPACE && element.name.local.as_ref() == "del"
}

/// Whether a `del` encloses `node` below `within`, so its content supplies nothing to `within`
/// (section 8.6.1).
fn inside_deletion(node: &NodeRef, within: &NodeRef) -> bool {
    node.ancestors()
        .take_while(|ancestor| ancestor != within)
        .any(|ancestor| is_deletion(&ancestor))
}

/// Every descendant element with the given tag name, in document order.
fn descendant_elements(root: &NodeRef, name: &str) -> Vec<NodeRef> {
    root.inclusive_descendants()
        .elements()
        .filter(|element| element.name.local.as_ref() == name)
        .map(|element| element.as_node().clone())
        .collect()
}

/// The first descendant element with the given tag name.
fn first_element(root: &NodeRef, name: &str) -> Option<NodeRef> {
    descendant_elements(root, name).into_iter().next()
}

/// The lowercase tag name of an element node.
fn element_name(node: &NodeRef) -> Option<String> {
    node.as_element()
        .map(|element| element.name.local.as_ref().to_lowercase())
}

/// Normalized text per section 6.1: every whitespace run becomes one space, then
/// the result is trimmed.
///
/// Section 8.5: `script`, `style`, `template`, and `noscript` contribute no text
/// wherever they appear. Section 6.4: a comment contributes no text, so text
/// split by one projects as the concatenation of its parts.
fn normalized_text(node: &NodeRef) -> String {
    let mut raw = String::new();
    collect_text(node, &mut raw, Reading::Current);
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalized text, or `None` when it is empty.
///
/// The schema's `nonEmptyText` requires at least one character, and an absent
/// property is what makes the validator report the missing field rather than an
/// empty string that satisfies nothing.
fn normalized_text_opt(node: &NodeRef) -> Option<String> {
    let text = normalized_text(node);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Preserved text per section 6.2: byte-for-byte, with no trimming and no
/// normalization.
fn preserved_text(node: &NodeRef) -> String {
    let mut raw = String::new();
    collect_text(node, &mut raw, Reading::Current);
    raw
}

/// Which text a walk collects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Reading {
    /// The text as it now reads: a `del`'s content is left out (section 8.6.1). Every field.
    Current,
    /// Every word, a `del`'s content included. Only a deletion's own entry (section 8.6.2).
    Whole,
}

/// Appends the text content of `node` and its descendants to `into`, as `reading` reads it.
fn collect_text(node: &NodeRef, into: &mut String, reading: Reading) {
    match node.data() {
        NodeData::Text(text) => into.push_str(&text.borrow()),
        NodeData::Element(element) => {
            if text_free_element(element.name.local.as_ref()) {
                return;
            }
            if reading == Reading::Current && is_deletion_element(element) {
                return;
            }
            for child in node.children() {
                collect_text(&child, into, reading);
            }
        }
        // A comment contributes nothing (section 6.4). A document or doctype
        // node has no text of its own; its children are walked below.
        NodeData::Comment(_) | NodeData::Doctype(_) => {}
        _ => {
            for child in node.children() {
                collect_text(&child, into, reading);
            }
        }
    }
}

/// Whether an element contributes no text to any field (section 8.5).
fn text_free_element(name: &str) -> bool {
    matches!(name, "script" | "style" | "template" | "noscript")
}

/// The content of a named `meta` element in the head (sections 9.2 and 9.3).
fn meta_content(root: &NodeRef, name: &str) -> Option<String> {
    descendant_elements(root, "meta")
        .into_iter()
        .find(|meta| attribute(meta, "name").as_deref() == Some(name))
        .and_then(|meta| attribute(&meta, "content"))
        .filter(|content| !content.is_empty())
}

/// The document's summary, per section 9.1.1.
///
/// Every `p` inside the document's `header`, in document order. A document with
/// no `header`, or a `header` with no paragraph, projects an empty summary and
/// stays valid: a fragment under review is still a document.
fn header_summary(root: &NodeRef) -> Vec<String> {
    let Some(header) = first_element(root, "header") else {
        return Vec::new();
    };
    // Section 8.6.1: a paragraph inside a `del` supplies no summary paragraph.
    descendant_elements(&header, "p")
        .iter()
        .filter(|paragraph| !inside_deletion(paragraph, root))
        .filter_map(normalized_text_opt)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_document;

    /// The projected model of `body`, as the JSON an independent implementation
    /// reproduces.
    fn model(body: &str) -> serde_json::Value {
        let html = format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <title>T</title></head><body>{body}</body></html>"
        );
        let projection = project(&parse_document(&html).document);
        serde_json::to_value(&projection.model).expect("the model serializes")
    }

    /// A modified requirement with one statement and the scenario `scenario`.
    fn requirement(statement: &str, scenario: &str) -> String {
        format!(
            "<section class=\"requirement\" id=\"req-x\" data-change=\"modified\">\
             <h2>R</h2><p>{statement}</p>{scenario}</section>"
        )
    }

    /// A scenario whose one `when` entry is `entry`.
    fn scenario(entry: &str) -> String {
        format!(
            "<article class=\"scenario\" id=\"scenario-x\" data-change=\"modified\"><h3>S</h3>\
             <dl><dt>When</dt><dd>{entry}</dd><dt>Then</dt><dd>It works.</dd></dl></article>"
        )
    }

    /// Section 8.6.1: normalized text leaves out what a `del` holds, and keeps what
    /// an `ins` holds. Preserved text does too (section 6.2), so a figure's source
    /// is its current text.
    #[test]
    fn text_leaves_out_what_a_del_holds() {
        let html = "<p id=\"p\">A <del>b </del>c <ins>d</ins> e</p>\
                    <pre id=\"pre\">line one\n<del>gone\n</del>line two</pre>";
        let document = parse_document(html).document;
        let p = first_element(&document, "p").expect("a paragraph");
        assert_eq!(normalized_text(&p), "A c d e");
        let pre = first_element(&document, "pre").expect("a pre");
        assert_eq!(preserved_text(&pre), "line one\nline two");
    }

    /// Section 8.6.2: a `del` inside another is part of the outer one's entry, so
    /// nested elements are one entry, holding the outer element's whole content.
    #[test]
    fn nested_dels_are_one_entry() {
        let model = model(&requirement(
            "The system SHALL <del>a <del>b</del> <ins>c</ins></del> hold.",
            &scenario("It happens."),
        ));
        let requirement = &model["requirements"][0];
        assert_eq!(
            requirement["statements"],
            serde_json::json!(["The system SHALL hold."])
        );
        assert_eq!(requirement["deletions"], serde_json::json!(["a b c"]));
    }

    /// Section 8.6.2: a `del` with no text records nothing, and a unit with no
    /// deletion serializes no `deletions`.
    #[test]
    fn an_empty_del_records_nothing() {
        let model = model(&requirement(
            "The system SHALL hold<del></del><del>   </del>.",
            &scenario("It happens."),
        ));
        let requirement = &model["requirements"][0];
        assert_eq!(
            requirement["statements"],
            serde_json::json!(["The system SHALL hold."])
        );
        assert!(requirement.get("deletions").is_none(), "{requirement}");
        assert!(
            requirement["scenarios"][0].get("deletions").is_none(),
            "{requirement}"
        );
    }

    /// Section 8.6.1: a `del` whose own class names a unit is that unit. Nested in a
    /// requirement, it projects as one of the requirement's scenarios, and the requirement
    /// records no deletion for it.
    #[test]
    fn a_del_that_is_a_nested_unit_records_no_deletion() {
        let scenario = "<del class=\"scenario\" id=\"scenario-y\" data-change=\"removed\">\
             <h3>S</h3><dl><dt>When</dt><dd>It happens.</dd><dt>Then</dt><dd>It works.</dd>\
             </dl></del>";
        let model = model(&requirement("The system SHALL hold.", scenario));
        let requirement = &model["requirements"][0];
        assert_eq!(
            requirement["scenarios"][0]["id"],
            serde_json::json!("scenario-y"),
            "{requirement}"
        );
        assert_eq!(
            requirement["scenarios"][0]["when"],
            serde_json::json!(["It happens."])
        );
        assert!(requirement.get("deletions").is_none(), "{requirement}");
    }

    /// Section 8.6.2: a `del` belongs to the nearest unit that encloses it, so one
    /// inside a scenario is the scenario's and not its requirement's.
    #[test]
    fn a_del_in_a_nested_unit_is_that_units() {
        let model = model(&requirement(
            "The system SHALL hold.",
            &scenario("It <del>fails</del> happens."),
        ));
        let requirement = &model["requirements"][0];
        let scenario = &requirement["scenarios"][0];
        assert_eq!(scenario["when"], serde_json::json!(["It happens."]));
        assert_eq!(scenario["deletions"], serde_json::json!(["fails"]));
        assert!(requirement.get("deletions").is_none(), "{requirement}");
    }
}
