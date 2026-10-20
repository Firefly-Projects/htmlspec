//! The projected document model.
//!
//! These types hold the model the projection specification defines, in the JSON
//! shape `schema/document.schema.json` validates. They are deliberately
//! *permissive*: an identifier is a `String` and an enumerated value is a
//! `String`, because the projection produces the model and does not judge it
//! (projection specification, section 10.1). A wrong prefix or an unrecognized
//! change state has to reach the runtime validator to be reported as a
//! `schema/*` diagnostic, so it must survive projection.
//!
//! This is why rule code does not use the types generated from the schema by
//! `typify`. Those newtypes validate their pattern in `TryFrom`, so an invalid
//! identifier cannot be represented at all — which would move enforcement into
//! the projection and lose the diagnostic. Generated types remain the
//! convenience the design describes for code that already holds a valid
//! document.
//!
//! A field the schema requires is serialized as an absent property when the
//! projection could not find it, so the runtime validator reports the missing
//! property rather than the projection inventing a default. A required *array*
//! is always serialized, even when empty, so that the schema's `minItems`
//! reports the real fault instead of a missing property.

use serde::Serialize;

/// One of the seven semantic units.
///
/// The reserved class name and the schema's `unit` discriminator are the same
/// string, and this enum is the projection's mapping between an HTML class token
/// and that discriminator. The test
/// `reserved_classes_match_the_schema_discriminators` keeps the enum and the
/// schema's definitions in agreement, so a unit added to one and not the other
/// fails rather than projecting a unit the schema has never heard of.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitKind {
    Requirement,
    Scenario,
    Decision,
    Task,
    Evidence,
    Risk,
    Verification,
}

impl UnitKind {
    /// Every unit, in the order the schema declares them.
    pub const ALL: &'static [Self] = &[
        Self::Requirement,
        Self::Scenario,
        Self::Decision,
        Self::Task,
        Self::Evidence,
        Self::Risk,
        Self::Verification,
    ];

    /// The reserved class name, which is also the schema's `unit` discriminator.
    pub const fn as_class(self) -> &'static str {
        match self {
            Self::Requirement => "requirement",
            Self::Scenario => "scenario",
            Self::Decision => "decision",
            Self::Task => "task",
            Self::Evidence => "evidence",
            Self::Risk => "risk",
            Self::Verification => "verification",
        }
    }

    /// The unit a class token names, or `None` when the token is not reserved.
    ///
    /// Matching is case sensitive and on a whole token, per the projection
    /// specification, section 1.2.
    pub fn from_class(token: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.as_class() == token)
    }
}

impl std::fmt::Display for UnitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_class())
    }
}

/// The change state a unit takes when it declares none.
///
/// Stated by the projection specification, section 2, and by the schema's
/// `changeState.default`. The test `the_default_change_state_matches_the_schema`
/// keeps the two in agreement.
pub const DEFAULT_CHANGE_STATE: &str = "unchanged";

/// The response a risk takes when it declares none.
///
/// Stated by the projection specification, section 4.7.1, and by the schema's
/// `risk.response.default`. The test `the_default_response_matches_the_schema` keeps
/// the two in agreement. Every risk written before the attribute existed had to state a
/// mitigation, so this default keeps each one's meaning and its validity.
pub const DEFAULT_RISK_RESPONSE: &str = "mitigated";

/// The projected document.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentModel {
    /// Projected from `<meta name="htmlspec-version">`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    /// Projected from the `title` element of the head.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Projected from the paragraphs of the document `header`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub summary: Vec<String>,
    /// Which artifact this document is, projected from
    /// `<meta name="htmlspec-kind">`.
    ///
    /// `Option` because the projection produces the model and does not judge it
    /// (projection specification, section 10.1): an absent kind has to survive
    /// projection so the runtime validator reports the missing property against a
    /// schema location. It is *not* optional in the contract — the schema requires
    /// it and states no default, so `None` here means the document declared none
    /// and is invalid, never that the kind is unknown or assumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<Requirement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<Decision>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<Task>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub risks: Vec<Risk>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verifications: Vec<Verification>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub figures: Vec<Figure>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

/// One normative obligation and the scenarios that make it checkable.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Requirement {
    pub unit: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub statements: Vec<String>,
    pub scenarios: Vec<Scenario>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Every `data-field="renamed-from"` paragraph the requirement carries, in
    /// document order (projection specification, section 4.1.1).
    ///
    /// The model's `renamedFrom` is the first of them with text, and it is
    /// serialized from this list so that the field and the paragraphs it came from
    /// cannot disagree. The others, an empty paragraph or a second one, reach no
    /// field. They are kept because a rename stated twice, or stated without its
    /// heading, is a fault the validator reports, and a projection that dropped
    /// them would report it as no rename at all.
    #[serde(
        rename = "renamedFrom",
        serialize_with = "serialize_previous_heading",
        skip_serializing_if = "states_no_previous_heading"
    )]
    pub renamed_from: Vec<RenamedFrom>,
    /// The text of each `del` that belongs to this requirement and that no other `del`
    /// encloses, in document order (projection specification, section 8.6.2).
    ///
    /// Words a change deleted from the accepted text. A `del`'s content supplies no other field,
    /// so the statements are the current text and this list is the only place the deleted words
    /// reach the model. Serialized only when it holds an entry, so the model of a document with
    /// no `del` is what it was before the field existed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deletions: Vec<String>,
}

impl Requirement {
    /// The heading this requirement replaced, when it states one.
    ///
    /// The first `renamed-from` paragraph with text, which is what the model's
    /// `renamedFrom` holds.
    pub fn previous_heading(&self) -> Option<&str> {
        previous_heading(&self.renamed_from)
    }

    /// This requirement's heading name (section 4.1.2), absent when it has no title.
    pub fn heading_name(&self) -> Option<HeadingName> {
        self.title
            .as_deref()
            .map(|title| HeadingName::of(title, self.id.as_deref()))
    }
}

/// The `data-field` value that marks the heading a requirement replaced.
///
/// Projection specification, section 4.1.1. An HTML attribute value rather than a
/// schema value: the schema names the field `renamedFrom`, and this is the
/// markup that projects into it.
pub const RENAMED_FROM_FIELD: &str = "renamed-from";

/// The `data-field` value that marks what a task's work did, where that departs from its words.
///
/// Projection specification, section 4.5. The markup that projects into a task's `deviation`.
pub const DEVIATION_FIELD: &str = "deviation";

/// What one `data-field="renamed-from"` paragraph states.
///
/// Two cases rather than an `Option<String>`, because an empty paragraph is not
/// the absence of one. It is a rename stated without its heading, and the
/// validator reports it as that.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenamedFrom {
    /// The heading the unit replaced, as normalized text.
    Heading(String),
    /// A paragraph whose normalized text is empty.
    Empty,
}

/// The first stated heading among `paragraphs`.
fn previous_heading(paragraphs: &[RenamedFrom]) -> Option<&str> {
    paragraphs.iter().find_map(|paragraph| match paragraph {
        RenamedFrom::Heading(heading) => Some(heading.as_str()),
        RenamedFrom::Empty => None,
    })
}

fn states_no_previous_heading(paragraphs: &[RenamedFrom]) -> bool {
    previous_heading(paragraphs).is_none()
}

fn serialize_previous_heading<S: serde::Serializer>(
    paragraphs: &[RenamedFrom],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    // `skip_serializing_if` has already removed the field when there is no stated
    // heading, so the `None` arm serializes a `null` that is never reached.
    previous_heading(paragraphs).serialize(serializer)
}

/// A unit's heading as a reader names it: its title without the identifier shown
/// beside it.
///
/// Projection specification, section 4.1.2. The template writes each heading as
/// `<h3>The name <span class="hs-id">req-x</span></h3>`, and section 3.1 takes a
/// title from all of a heading's text, so the projected title is `The name req-x`.
/// A rename is stated as the name a reader read, so it is compared with this and
/// never with the title. A title and a heading name are both strings, and a
/// distinct type is what keeps a comparison from taking one for the other.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HeadingName(String);

impl HeadingName {
    /// The heading name of a unit titled `title` whose identifier is `id`.
    ///
    /// The title is whitespace-normalized first, then the identifier is removed
    /// from its end and the result trimmed. The identifier is removed wherever the
    /// title ends in it, with or without a space before it, because a heading
    /// written `The name<span class="hs-id">req-x</span>` shows the same name.
    /// A previous heading, which carries no identifier, is `HeadingName::of(text, None)`.
    pub fn of(title: &str, id: Option<&str>) -> Self {
        let normalized = title.split_whitespace().collect::<Vec<_>>().join(" ");
        let name = id
            .filter(|id| !id.is_empty())
            .and_then(|id| normalized.strip_suffix(id))
            .map_or(normalized.as_str(), str::trim_end);
        Self(name.to_owned())
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HeadingName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One condition-and-outcome pairing inside a requirement.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Scenario {
    pub unit: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub given: Vec<String>,
    pub when: Vec<String>,
    pub then: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// The text of each `del` that belongs to this scenario and that no other `del` encloses, in
    /// document order (projection specification, section 8.6.2). A `del` in a scenario is the
    /// scenario's and never its requirement's.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deletions: Vec<String>,
}

/// A choice reached, why it was reached, and its standing.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Decision {
    pub unit: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement: Option<String>,
    pub rationale: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(rename = "supersededBy", skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<Reference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// One unit of work and its completion state.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Task {
    pub unit: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// What was done, where that departs from the task's words (section 4.5).
    ///
    /// Absent for a task done as its words say. Only a `done` task may state one, which the
    /// validator checks, because the projection produces the model and does not judge it: a
    /// deviation on a task in another state survives into the model so the validator can name
    /// the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deviation: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<Reference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// A measurement or observation supporting a claim elsewhere.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Evidence {
    pub unit: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub summary: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<Reference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// A way the change can fail, and how the change settled it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Risk {
    pub unit: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement: Option<String>,
    /// What holds the risk down. Required of a `mitigated` risk by the validator, and
    /// optional on an `accepted` one, where it states a partial measure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitigation: Option<String>,
    /// How the change settled the risk, from `data-response` or
    /// [`DEFAULT_RISK_RESPONSE`] (projection specification, section 4.7.1).
    ///
    /// Always serialized, as `change` is, so a consumer never applies the default
    /// itself. A `String` for the reason this module's documentation gives: a value
    /// outside the enumeration survives projection, so the validator reports it with
    /// the permitted values rather than the projection losing it.
    pub response: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<Reference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// The recorded outcome of checking a requirement or scenario.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Verification {
    pub unit: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub change: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Each part of what the verification names that its check did not reach, from every
    /// `data-field="unchecked"` paragraph, in document order (projection specification,
    /// section 4.8). Repeatable, because a check can miss more than one clause and a dropped
    /// paragraph would understate what it missed. Serialized only when it holds an entry, so
    /// the model of a document with no such paragraph is what it was before the field existed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unchecked: Vec<String>,
    pub references: Vec<Reference>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// One unit addressing another by identifier.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Reference {
    pub target: String,
    pub origin: ReferenceOrigin,
}

/// How a reference was written in the HTML.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceOrigin {
    /// From the `data-refs` attribute.
    Attribute,
    /// From an anchor whose `href` is a same-document fragment.
    Link,
}

/// An internal fragment link, with the fragment's leading `#` removed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Link {
    pub target: String,
    pub text: String,
}

/// A figure and its caption. A figure is not a semantic unit.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Figure {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: FigureKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// What a figure carries, decided by its first `pre` element.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FigureKind {
    Mermaid,
    VegaLite,
    Math,
    Code,
    Prose,
}

/// A read-only view of one semantic unit, whatever its type.
///
/// Every relational rule needs the same four facts about a unit — its type, its
/// declared identifier, its change state, and the references it makes — and none
/// of them needs the unit's own fields. One view over all seven types means a
/// rule is written once rather than seven times, and a unit added to the model
/// reaches every rule as soon as [`DocumentModel::units`] yields it.
#[derive(Clone, Copy, Debug)]
pub struct UnitView<'a> {
    /// Which of the seven units this is.
    pub kind: UnitKind,
    /// The declared identifier, absent when the unit declared none. The schema
    /// reports the missing property; a relational rule has nothing to relate.
    pub id: Option<&'a str>,
    /// The declared change state, or the default when none was declared.
    pub change: &'a str,
    /// The references the unit makes, in projection order.
    pub references: &'a [Reference],
}

impl DocumentModel {
    /// Every semantic unit in the document, in document order.
    ///
    /// Order is requirements first — each immediately followed by its own
    /// scenarios, so a scenario is adjacent to the requirement that contains it —
    /// then decisions, tasks, evidence, risks, and verifications.
    pub fn units(&self) -> Vec<UnitView<'_>> {
        const NO_REFERENCES: &[Reference] = &[];
        let mut found: Vec<UnitView<'_>> = Vec::new();
        for requirement in &self.requirements {
            found.push(UnitView {
                kind: requirement.unit,
                id: requirement.id.as_deref(),
                change: &requirement.change,
                references: NO_REFERENCES,
            });
            for scenario in &requirement.scenarios {
                found.push(UnitView {
                    kind: scenario.unit,
                    id: scenario.id.as_deref(),
                    change: &scenario.change,
                    references: NO_REFERENCES,
                });
            }
        }
        for decision in &self.decisions {
            found.push(UnitView {
                kind: decision.unit,
                id: decision.id.as_deref(),
                change: &decision.change,
                references: &decision.references,
            });
        }
        for task in &self.tasks {
            found.push(UnitView {
                kind: task.unit,
                id: task.id.as_deref(),
                change: &task.change,
                references: &task.references,
            });
        }
        for evidence in &self.evidence {
            found.push(UnitView {
                kind: evidence.unit,
                id: evidence.id.as_deref(),
                change: &evidence.change,
                references: &evidence.references,
            });
        }
        for risk in &self.risks {
            found.push(UnitView {
                kind: risk.unit,
                id: risk.id.as_deref(),
                change: &risk.change,
                references: &risk.references,
            });
        }
        for verification in &self.verifications {
            found.push(UnitView {
                kind: verification.unit,
                id: verification.id.as_deref(),
                change: &verification.change,
                references: &verification.references,
            });
        }
        found
    }

    /// Every reference in the document, paired with the unit that made it.
    ///
    /// A unit with no identifier of its own still contributes its references, so
    /// a dangling reference is not hidden by a second fault in the same unit.
    pub fn references(&self) -> Vec<(UnitView<'_>, &Reference)> {
        self.units()
            .into_iter()
            .flat_map(|unit| unit.references.iter().map(move |r| (unit, r)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> serde_json::Value {
        serde_json::from_str(htmlspec_schema::schema_text::DOCUMENT).expect("schema parses")
    }

    /// The projection maps a class name onto a unit; the schema owns the unit's
    /// discriminator. A class name with no matching discriminator would project
    /// a unit the schema has never heard of.
    #[test]
    fn reserved_classes_match_the_schema_discriminators() {
        let schema = schema();
        for kind in UnitKind::ALL {
            let class = kind.as_class();
            assert_eq!(
                schema["definitions"][class]["properties"]["unit"]["const"],
                serde_json::Value::from(class),
                "the schema has no unit whose discriminator is {class}"
            );
            assert_eq!(
                serde_json::to_value(kind).expect("a unit kind serializes"),
                serde_json::Value::from(class),
                "{class} serializes as something other than its discriminator"
            );
        }
        let declared = schema["definitions"]
            .as_object()
            .expect("definitions is an object")
            .values()
            .filter(|d| d["properties"]["unit"]["const"].is_string())
            .count();
        assert_eq!(
            declared,
            UnitKind::ALL.len(),
            "the schema declares {declared} units and the projection knows {}",
            UnitKind::ALL.len()
        );
    }

    #[test]
    fn a_class_token_outside_the_vocabulary_names_no_unit() {
        assert_eq!(
            UnitKind::from_class("requirement"),
            Some(UnitKind::Requirement)
        );
        // Section 1.2: whole-token, case-sensitive matching.
        assert_eq!(UnitKind::from_class("requirements"), None);
        assert_eq!(UnitKind::from_class("Requirement"), None);
        assert_eq!(UnitKind::from_class("note"), None);
    }

    #[test]
    fn the_default_change_state_matches_the_schema() {
        assert_eq!(
            schema()["definitions"]["changeState"]["default"],
            serde_json::Value::from(DEFAULT_CHANGE_STATE)
        );
    }

    /// Section 4.7.1: a risk that declares no response projects the schema's default, and
    /// the default is one of the values the schema permits.
    #[test]
    fn the_default_response_matches_the_schema() {
        let response = &schema()["definitions"]["risk"]["properties"]["response"];
        assert_eq!(
            response["default"],
            serde_json::Value::from(DEFAULT_RISK_RESPONSE)
        );
        assert!(
            response["enum"].as_array().is_some_and(
                |values| values.contains(&serde_json::Value::from(DEFAULT_RISK_RESPONSE))
            ),
            "the default response is not a value the schema permits: {response}"
        );
    }

    /// A required array serializes even when empty, so `minItems` reports the
    /// fault rather than a missing property. This is the property task 4.7's
    /// fixtures rest on.
    #[test]
    fn an_empty_required_array_is_serialized_rather_than_omitted() {
        let requirement = Requirement {
            unit: UnitKind::Requirement,
            id: None,
            change: DEFAULT_CHANGE_STATE.to_owned(),
            title: None,
            statements: Vec::new(),
            scenarios: Vec::new(),
            notes: Vec::new(),
            renamed_from: Vec::new(),
            deletions: Vec::new(),
        };
        let json = serde_json::to_value(&requirement).expect("serializes");
        assert!(json.get("statements").is_some(), "{json}");
        assert!(json.get("scenarios").is_some(), "{json}");
        assert!(json.get("id").is_none(), "an absent id must not serialize");
    }

    /// Section 4.1.2: the identifier comes off the end, with or without the space
    /// the template writes before it, and nothing else about the title changes.
    #[test]
    fn a_heading_name_leaves_out_only_the_units_own_identifier_at_its_end() {
        let name = |title: &str, id: Option<&str>| HeadingName::of(title, id).as_str().to_owned();
        assert_eq!(name("The name req-x", Some("req-x")), "The name");
        assert_eq!(name("The namereq-x", Some("req-x")), "The name");
        assert_eq!(name("The name", Some("req-x")), "The name");
        assert_eq!(name("The name req-x", None), "The name req-x");
        // Another unit's identifier is part of the name, not an annotation of it.
        assert_eq!(name("The name req-y", Some("req-x")), "The name req-y");
        // Only at the end: an identifier the title merely mentions stays.
        assert_eq!(
            name("How req-x is renamed", Some("req-x")),
            "How req-x is renamed"
        );
        assert_eq!(name("  The\n  name   req-x ", Some("req-x")), "The name");
        assert_eq!(name("req-x", Some("req-x")), "");
    }

    /// A previous heading is compared as a heading with no identifier, and the
    /// comparison is case-sensitive: OpenSpec allows a case-only rename, and so
    /// does this.
    #[test]
    fn heading_names_compare_exactly_after_whitespace_normalization() {
        assert_eq!(
            HeadingName::of("The  old\nname", None),
            HeadingName::of("The old name req-x", Some("req-x"))
        );
        assert_ne!(
            HeadingName::of("the old name", None),
            HeadingName::of("The old name", None)
        );
    }

    /// The model's `renamedFrom` is the first paragraph with text, and it is
    /// absent rather than null when no paragraph states one.
    #[test]
    fn renamed_from_serializes_the_first_stated_heading_or_nothing() {
        let mut requirement = Requirement {
            unit: UnitKind::Requirement,
            id: Some("req-x".to_owned()),
            change: DEFAULT_CHANGE_STATE.to_owned(),
            title: Some("The new name".to_owned()),
            statements: Vec::new(),
            scenarios: Vec::new(),
            notes: Vec::new(),
            renamed_from: vec![RenamedFrom::Empty],
            deletions: Vec::new(),
        };
        let json = serde_json::to_value(&requirement).expect("serializes");
        assert!(json.get("renamedFrom").is_none(), "{json}");

        requirement
            .renamed_from
            .push(RenamedFrom::Heading("The old name".to_owned()));
        let json = serde_json::to_value(&requirement).expect("serializes");
        assert_eq!(json["renamedFrom"], "The old name", "{json}");
    }
}
