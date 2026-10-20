//! The migration plan, and the check that it accounts for what the survey found.
//!
//! The second of a migration's three exact parts. The survey enumerates, an agent decides, and
//! this compares the two sets. `project-migration`'s "Promotion refuses while any surveyed entity
//! is unaccounted for" states the division: **the agent decides what each entity becomes, and the
//! program checks that a decision was recorded for all of them.** So nothing here reads a
//! disposition and asks whether it is right. A plan sending a specification to the wrong
//! destination passes this check, because that judgment is the thing the plan exists to put in
//! front of a contributor.
//!
//! ## Why the plan is a `tasks` document, and where each field lives
//!
//! A plan is a set of work items with states, which is what the `tasks` kind carries, and
//! `artifact-kinds` states that the set of four kinds is closed. So the plan is a `tasks`
//! document and one task unit accounts for one surveyed entity.
//!
//! The projected model of a task holds `id`, `change`, `title`, `state`, `reason`, `references`,
//! and `notes` — and nothing else. A migration plan needs three more facts per entity: the source
//! path, the destination path, and the disposition. The projection has no room for them:
//! `data-field="source"` names no field of a task and is invalid, and an unknown `data-*`
//! attribute on the unit is ignored by the projection, so a gate reading one would have to parse
//! the HTML a second way and disagree with the model about what the document says.
//!
//! So each fact is a note paragraph opening with its name and a colon:
//!
//! ```html
//! <li class="task" id="task-openspec-specs-project-bootstrap-spec-md"
//!     data-change="added" data-state="todo">
//!   <h3>The project-bootstrap capability's accepted specification</h3>
//!   <p data-field="note">source: openspec/specs/project-bootstrap/spec.md</p>
//!   <p data-field="note">destination: htmlspec/specs/project-bootstrap/spec.html</p>
//!   <p data-field="note">disposition: converted</p>
//!   <p data-field="note">derives: A project has a declared root holding one asset store -&gt; req-a-project-has-a-declared-root-holding-one-asset-store</p>
//! </li>
//! ```
//!
//! A note whose opening word is not one of the five names is ordinary prose. That keeps the
//! convention additive: a contributor annotating a plan line writes a paragraph and nothing
//! breaks.
//!
//! ## The identifier derivations live on the entity that stated them
//!
//! `project-migration` requires the derivation to be "recorded against the source document that
//! stated the heading, rather than against the heading alone", because one continuing obligation
//! can be stated under different headings in different changes. Putting each `derives:` line on
//! the plan task for the file that stated it satisfies that by construction — the line cannot be
//! recorded without the source it came from.
//!
//! It also gives the collision check its grouping for free. A collision is two headings deriving
//! one identifier **within one destination document**, so [`Coverage`] groups derivations by the
//! `destination:` of the item that holds them. An obligation renamed across three changes keeps
//! one identifier and lands in three archived specifications under three headings, which are
//! three destinations, so the three statements of it do not collide — which is the case task 4.8
//! pins and the case a heading-keyed table would have reported wrongly.

use std::collections::BTreeMap;
use std::path::Path;

/// The dispositions the plan's vocabulary holds.
///
/// The four `project-migration` names. Reading the word is not judging it: a word outside this
/// set is reported as a disposition nobody can act on, which is the same class of fault as no
/// disposition at all.
pub const DISPOSITIONS: &[&str] = &["converted", "folded", "carried", "dropped"];

/// The dispositions that owe a reason.
///
/// "A folded or dropped entity SHALL carry the reason it was folded or dropped." Checking that a
/// reason is present is not checking that it is a good one.
const OWES_A_REASON: &[&str] = &["folded", "dropped"];

/// The note key naming the source entity a plan item accounts for.
const SOURCE: &str = "source";

/// The note key naming where the entity is going.
const DESTINATION: &str = "destination";

/// The note key naming what happens to the entity.
const DISPOSITION: &str = "disposition";

/// The note key carrying the reason a folded or dropped entity was folded or dropped.
///
/// Read from a note as well as from the task's own `reason` field, because a task whose
/// `data-state` is `dropped` already carries `data-field="reason"` and the validator already
/// requires it. Both spellings mean the same thing, and refusing one would make the plan's shape
/// depend on which of two equivalent markups an agent chose.
const REASON: &str = "reason";

/// The note key recording one requirement identifier derivation.
const DERIVES: &str = "derives";

/// What separates a source heading from the identifier derived from it.
///
/// Split from the right, because a heading can contain an arrow and an identifier cannot.
const DERIVES_SEPARATOR: &str = " -> ";

/// One requirement identifier, and the heading it was derived from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Derivation {
    /// The source heading, in full.
    pub heading: String,
    /// The identifier derived from it.
    pub identifier: String,
}

/// One plan item: an entity, where it is going, and what happens to it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Item {
    /// The task unit's identifier, for naming the item in a report.
    pub id: String,
    /// The task unit's title.
    pub title: String,
    /// The source entity this item accounts for.
    pub source: Option<String>,
    /// Where the entity is going.
    pub destination: Option<String>,
    /// What happens to it, as the plan wrote it.
    pub disposition: Option<String>,
    /// Why it was folded or dropped.
    pub reason: Option<String>,
    /// The requirement identifiers this source document's headings derive.
    pub derivations: Vec<Derivation>,
    /// Every `derives:` note whose two halves could not be separated.
    ///
    /// Kept rather than discarded. A line the reader cannot parse is a derivation the plan states
    /// and the collision check cannot see, so silently dropping it means a plan that misspells the
    /// separator on all 120 of its requirement lines records no derivations, reports no collisions,
    /// and reads clean — the systematic-fault shape the rest of this change exists to surface. A
    /// review of the pull request found it.
    pub unreadable_derivations: Vec<String>,
}

impl Item {
    /// How to name this item in a report.
    ///
    /// The identifier where it has one, because that is what a contributor searches the plan
    /// for. The title otherwise.
    fn named(&self) -> &str {
        if self.id.is_empty() {
            &self.title
        } else {
            &self.id
        }
    }
}

/// A migration plan, as its document records it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Plan {
    /// One item per task unit, in document order.
    pub items: Vec<Item>,
}

impl Plan {
    /// Reads the plan at `path`.
    ///
    /// Reads it through the same projection every other operation reads a document through. A
    /// second reader would let the plan say one thing to the gate and another to `validate`.
    pub fn read(path: &Path) -> Result<Self, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|error| format!("{} could not be read: {error}", path.display()))?;
        let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);

        match projection.model.kind.as_deref() {
            Some("tasks") => {}
            Some(other) => {
                return Err(format!(
                    "{} declares kind {other}. A migration plan is a tasks document: it is a set \
                     of work items with states, and the set of artifact kinds is closed",
                    path.display()
                ))
            }
            None => {
                return Err(format!(
                    "{} declares no artifact kind, so it is not a plan this check can read",
                    path.display()
                ))
            }
        }

        Ok(Self {
            items: projection
                .model
                .tasks
                .iter()
                .map(|task| {
                    let mut item = Item {
                        id: task.id.clone().unwrap_or_default(),
                        title: task.title.clone().unwrap_or_default(),
                        reason: task.reason.clone(),
                        ..Item::default()
                    };
                    for note in &task.notes {
                        let Some((key, value)) = note.split_once(':') else {
                            continue;
                        };
                        let value = value.trim().to_owned();
                        if value.is_empty() {
                            continue;
                        }
                        match key.trim().to_ascii_lowercase().as_str() {
                            SOURCE => item.source = Some(value),
                            DESTINATION => item.destination = Some(value),
                            // Lowercased, because the vocabulary is four words and a plan
                            // written with a capital should not read as a word nobody knows.
                            DISPOSITION => {
                                item.disposition = Some(value.to_ascii_lowercase());
                            }
                            REASON => item.reason = item.reason.take().or(Some(value)),
                            DERIVES => match value.rsplit_once(DERIVES_SEPARATOR) {
                                Some((heading, identifier)) => item.derivations.push(Derivation {
                                    heading: heading.trim().to_owned(),
                                    identifier: identifier.trim().to_owned(),
                                }),
                                // Reported rather than skipped. See `unreadable_derivations`.
                                None => item.unreadable_derivations.push(value),
                            },
                            // Any other note is prose. The convention is additive on purpose.
                            _ => {}
                        }
                    }
                    item
                })
                .collect(),
        })
    }
}

/// Two headings deriving one identifier inside one destination document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Collision {
    /// The destination document both would land in.
    pub destination: String,
    /// The identifier both derive.
    pub identifier: String,
    /// The source path and heading of each, sorted.
    pub sources: Vec<(String, String)>,
}

/// What a plan failed to account for.
///
/// Every field is a class of omission rather than a judgment. A plan that names every entity and
/// records a usable disposition for each produces an empty [`Coverage`], whatever a reader would
/// think of the dispositions themselves.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Coverage {
    /// Surveyed entities no plan item names.
    pub unaccounted: Vec<String>,
    /// Plan items naming a source with no disposition recorded.
    pub without_disposition: Vec<String>,
    /// Plan items whose disposition is not one of the four, with the word they wrote.
    pub unknown_disposition: Vec<(String, String)>,
    /// Plan items whose disposition owes a reason and records none.
    pub without_reason: Vec<String>,
    /// Sources named by more than one plan item.
    pub duplicated: Vec<String>,
    /// Plan items naming a source the survey never found.
    pub unsurveyed: Vec<String>,
    /// Identifier collisions within one destination document.
    pub collisions: Vec<Collision>,
    /// Plan items carrying no source at all.
    pub sourceless: Vec<String>,
    /// Plan items naming a source with no destination recorded.
    ///
    /// The requirement says the plan records "its destination and its disposition" for each entity,
    /// and only the disposition was checked. The omission was not merely a missing report: the
    /// collision check groups derivations by destination, so an item without one had **every** one
    /// of its `derives:` lines excluded from collision detection. Two distinct headings deriving one
    /// identifier inside one collapsed specification then reached the staged tree, where it surfaces
    /// as a duplicate-identifier error in a document that has to be re-collapsed by hand.
    pub without_destination: Vec<String>,
    /// `derives:` notes whose heading and identifier could not be separated, with the line.
    pub unreadable_derivations: Vec<(String, String)>,
}

impl Coverage {
    /// Compares `plan` against the entity paths `surveyed`.
    ///
    /// `surveyed` comes from the survey's payload rather than from a fresh enumeration, because
    /// the requirement is that the plan is checked "against the list the plan was written from"
    /// rather than against a second opinion taken later.
    pub fn of(plan: &Plan, surveyed: &[String]) -> Self {
        let mut coverage = Self::default();
        let mut accounted: BTreeMap<&str, usize> = BTreeMap::new();

        for item in &plan.items {
            let Some(source) = item.source.as_deref() else {
                coverage.sourceless.push(item.named().to_owned());
                continue;
            };
            *accounted.entry(source).or_insert(0) += 1;

            if item.destination.is_none() {
                coverage.without_destination.push(source.to_owned());
            }
            for line in &item.unreadable_derivations {
                coverage
                    .unreadable_derivations
                    .push((source.to_owned(), line.clone()));
            }

            match item.disposition.as_deref() {
                None => coverage.without_disposition.push(source.to_owned()),
                Some(word) if !DISPOSITIONS.contains(&word) => coverage
                    .unknown_disposition
                    .push((source.to_owned(), word.to_owned())),
                Some(word) => {
                    if OWES_A_REASON.contains(&word) && item.reason.is_none() {
                        coverage.without_reason.push(source.to_owned());
                    }
                }
            }
        }

        for (source, count) in &accounted {
            if *count > 1 {
                coverage.duplicated.push((*source).to_owned());
            }
        }

        for entity in surveyed {
            if !accounted.contains_key(entity.as_str()) {
                coverage.unaccounted.push(entity.clone());
            }
        }
        for source in accounted.keys() {
            if !surveyed.iter().any(|entity| entity == source) {
                coverage.unsurveyed.push((*source).to_owned());
            }
        }

        coverage.collisions = collisions(plan);
        coverage
    }

    /// Whether the plan accounts for everything.
    pub fn is_complete(&self) -> bool {
        self.unaccounted.is_empty()
            && self.without_disposition.is_empty()
            && self.unknown_disposition.is_empty()
            && self.without_reason.is_empty()
            && self.duplicated.is_empty()
            && self.unsurveyed.is_empty()
            && self.collisions.is_empty()
            && self.sourceless.is_empty()
            && self.without_destination.is_empty()
            && self.unreadable_derivations.is_empty()
    }

    /// The report a contributor reads.
    ///
    /// **Every occurrence of every class, together.** Naming the first unaccounted entity would
    /// turn a systematic omission — a whole entity kind the plan has no rows for — into a
    /// sequence of individual repairs, which is the same fault the check-last rule exists to
    /// prevent one stage later.
    pub fn report(&self) -> String {
        let mut report = String::new();

        if !self.unaccounted.is_empty() {
            report.push_str(&format!(
                "{} surveyed {} no disposition in the plan:\n",
                self.unaccounted.len(),
                plural(self.unaccounted.len(), "entity has", "entities have")
            ));
            for entity in &self.unaccounted {
                report.push_str(&format!("  {entity}\n"));
            }
        }

        if !self.sourceless.is_empty() {
            report.push_str(&format!(
                "\n{} plan {} no source entity, so nothing is accounted for by them:\n",
                self.sourceless.len(),
                plural(self.sourceless.len(), "item names", "items name")
            ));
            for item in &self.sourceless {
                report.push_str(&format!("  {item}\n"));
            }
        }

        if !self.without_destination.is_empty() {
            report.push_str(&format!(
                "\n{} plan {} a source and record no destination:\n",
                self.without_destination.len(),
                plural(self.without_destination.len(), "item names", "items name")
            ));
            for source in &self.without_destination {
                report.push_str(&format!("  {source}\n"));
            }
        }

        if !self.unreadable_derivations.is_empty() {
            report.push_str(&format!(
                "\n{} derivation {} its heading from its identifier. The separator is ` -> `, and \
                 a line the check cannot read is a derivation no collision check can see:\n",
                self.unreadable_derivations.len(),
                plural(
                    self.unreadable_derivations.len(),
                    "note does not separate",
                    "notes do not separate"
                )
            ));
            for (source, line) in &self.unreadable_derivations {
                report.push_str(&format!("  {source}  wrote: {line}\n"));
            }
        }

        if !self.without_disposition.is_empty() {
            report.push_str(&format!(
                "\n{} plan {} a source and record no disposition:\n",
                self.without_disposition.len(),
                plural(self.without_disposition.len(), "item names", "items name")
            ));
            for source in &self.without_disposition {
                report.push_str(&format!("  {source}\n"));
            }
        }

        if !self.unknown_disposition.is_empty() {
            report.push_str(&format!(
                "\n{} plan {} a disposition outside the four ({}):\n",
                self.unknown_disposition.len(),
                plural(
                    self.unknown_disposition.len(),
                    "item records",
                    "items record"
                ),
                DISPOSITIONS.join(", ")
            ));
            for (source, word) in &self.unknown_disposition {
                report.push_str(&format!("  {source}  wrote {word}\n"));
            }
        }

        if !self.without_reason.is_empty() {
            report.push_str(&format!(
                "\n{} folded or dropped {} no reason. An entity dropped with a reason is \
                 accounted for; one dropped without a reason records that something is gone and \
                 not what the decision was:\n",
                self.without_reason.len(),
                plural(
                    self.without_reason.len(),
                    "entity carries",
                    "entities carry"
                )
            ));
            for source in &self.without_reason {
                report.push_str(&format!("  {source}\n"));
            }
        }

        if !self.duplicated.is_empty() {
            report.push_str(&format!(
                "\n{} {} named by more than one plan item, so which disposition applies is \
                 undecided:\n",
                self.duplicated.len(),
                plural(self.duplicated.len(), "entity is", "entities are")
            ));
            for source in &self.duplicated {
                report.push_str(&format!("  {source}\n"));
            }
        }

        if !self.unsurveyed.is_empty() {
            report.push_str(&format!(
                "\n{} plan {} a source the survey did not find. The plan was written against a \
                 different survey, so re-survey and reconcile before promoting:\n",
                self.unsurveyed.len(),
                plural(self.unsurveyed.len(), "item names", "items name")
            ));
            for source in &self.unsurveyed {
                report.push_str(&format!("  {source}\n"));
            }
        }

        if !self.collisions.is_empty() {
            report.push_str(&format!(
                "\n{} {} derived by two source headings within one destination document. A \
                 suffix would make two obligations read as versions of one, so this is reported \
                 rather than resolved:\n",
                self.collisions.len(),
                plural(self.collisions.len(), "identifier is", "identifiers are")
            ));
            for collision in &self.collisions {
                report.push_str(&format!(
                    "  {} in {}\n",
                    collision.identifier, collision.destination
                ));
                for (source, heading) in &collision.sources {
                    report.push_str(&format!("      {heading}  ({source})\n"));
                }
            }
        }

        report
    }
}

/// Whichever of `one` and `many` fits `count`.
fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 {
        one
    } else {
        many
    }
}

/// The identifiers two distinct source headings derive within one destination document.
fn collisions(plan: &Plan) -> Vec<Collision> {
    // Keyed on (destination, identifier), holding the distinct (source, heading) pairs that
    // derived it. Distinct, because one plan repeating a line states one derivation twice and
    // that is not two obligations sharing an identifier.
    let mut derived: BTreeMap<(&str, &str), Vec<(String, String)>> = BTreeMap::new();
    for item in &plan.items {
        let Some(destination) = item.destination.as_deref() else {
            continue;
        };
        let source = item.source.as_deref().unwrap_or(item.named());
        for derivation in &item.derivations {
            let pairs = derived
                .entry((destination, derivation.identifier.as_str()))
                .or_default();
            let pair = (source.to_owned(), derivation.heading.clone());
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }

    derived
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|((destination, identifier), mut sources)| {
            sources.sort();
            Collision {
                destination: destination.to_owned(),
                identifier: identifier.to_owned(),
                sources,
            }
        })
        .collect()
}

/// The entity paths a survey payload reports.
///
/// Read from the payload rather than from a fresh survey. A coverage check that re-enumerated
/// the source would compare the plan against a second opinion rather than against the list the
/// plan was written from.
pub fn surveyed_paths(payload: &serde_json::Value) -> Result<Vec<String>, String> {
    let Some(entities) = payload["entities"].as_array() else {
        return Err(String::from(
            "the survey payload has no entities array. Produce it with `htmlspec migrate survey \
             --json <source>`",
        ));
    };
    Ok(entities
        .iter()
        .filter_map(|entity| entity["path"].as_str())
        .map(str::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plan document holding `items` as task units.
    fn document(items: &str) -> String {
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"tasks\" name=\"htmlspec-kind\">\
             <title>Migration plan</title></head><body><main>{items}</main></body></html>"
        )
    }

    /// One plan item, spelled the way the module documentation spells it.
    fn item(id: &str, notes: &[&str]) -> String {
        let notes: String = notes
            .iter()
            .map(|note| format!("<p data-field=\"note\">{note}</p>"))
            .collect();
        format!(
            "<li class=\"task\" id=\"task-{id}\" data-change=\"added\" data-state=\"todo\">\
             <h3>{id}</h3>{notes}</li>"
        )
    }

    /// Writes `html` to a scratch file and reads it back as a plan.
    fn plan_of(html: &str) -> Plan {
        let path = std::env::temp_dir().join(format!(
            "htmlspec-plan-{}-{:p}.html",
            std::process::id(),
            html
        ));
        std::fs::write(&path, html).expect("the plan is written");
        let plan = Plan::read(&path).expect("the plan reads");
        let _ = std::fs::remove_file(&path);
        plan
    }

    /// A complete plan for two entities.
    fn complete() -> (Plan, Vec<String>) {
        let plan = plan_of(&document(&format!(
            "{}{}",
            item(
                "a",
                &[
                    "source: openspec/specs/a/spec.md",
                    "destination: htmlspec/specs/a/spec.html",
                    "disposition: converted",
                ]
            ),
            item(
                "b",
                &[
                    "source: openspec/config.yaml",
                    "destination: none",
                    "disposition: dropped",
                    "reason: the declaration carries no authoring rules or project context",
                ]
            )
        )));
        let surveyed = vec![
            String::from("openspec/specs/a/spec.md"),
            String::from("openspec/config.yaml"),
        ];
        (plan, surveyed)
    }

    /// A plan naming every entity with a usable disposition is complete.
    #[test]
    fn a_plan_accounting_for_every_entity_is_complete() {
        let (plan, surveyed) = complete();
        let coverage = Coverage::of(&plan, &surveyed);
        assert!(coverage.is_complete(), "{coverage:?}");
        assert_eq!(coverage.report(), "");
    }

    /// The refusal names **every** unaccounted entity. Naming the first turns a systematic
    /// omission — a whole entity kind the plan has no rows for — into a sequence of individual
    /// repairs.
    #[test]
    fn every_unaccounted_entity_is_named() {
        let (plan, mut surveyed) = complete();
        for missing in [
            "openspec/changes/archive/x/VERSION",
            "openspec/x/.openspec.yaml",
        ] {
            surveyed.push(String::from(missing));
        }
        let coverage = Coverage::of(&plan, &surveyed);

        assert!(!coverage.is_complete());
        assert_eq!(coverage.unaccounted.len(), 2, "{coverage:?}");
        let report = coverage.report();
        for missing in [
            "openspec/changes/archive/x/VERSION",
            "openspec/x/.openspec.yaml",
        ] {
            assert!(report.contains(missing), "{report}");
        }
    }

    /// Removing one entity from a complete plan fails the check, so its subject is proven rather
    /// than assumed. A gate that passed everything would pass every test above.
    #[test]
    fn removing_one_entity_from_a_complete_plan_fails_the_check() {
        let (plan, surveyed) = complete();
        assert!(Coverage::of(&plan, &surveyed).is_complete());

        let one_fewer = Plan {
            items: plan.items[1..].to_vec(),
        };
        let coverage = Coverage::of(&one_fewer, &surveyed);
        assert!(!coverage.is_complete());
        assert_eq!(coverage.unaccounted, vec!["openspec/specs/a/spec.md"]);
    }

    /// An entity dropped with a stated reason satisfies the check. Being accounted for is what
    /// the check tests, not being converted.
    #[test]
    fn a_deliberate_drop_with_a_reason_is_covered() {
        let plan = plan_of(&document(&item(
            "dropped",
            &[
                "source: openspec/config.yaml",
                "destination: none",
                "disposition: dropped",
                "reason: the destination's declaration has nowhere to put project context",
            ],
        )));
        let coverage = Coverage::of(&plan, &[String::from("openspec/config.yaml")]);
        assert!(coverage.is_complete(), "{coverage:?}");
    }

    /// A dropped entity with no reason is reported. The reason is the difference between
    /// recording a decision and recording that something is gone.
    #[test]
    fn a_drop_with_no_reason_is_reported() {
        let plan = plan_of(&document(&item(
            "dropped",
            &[
                "source: openspec/config.yaml",
                "destination: none",
                "disposition: dropped",
            ],
        )));
        let coverage = Coverage::of(&plan, &[String::from("openspec/config.yaml")]);
        assert_eq!(coverage.without_reason, vec!["openspec/config.yaml"]);
    }

    /// The reason may come from the task's own `reason` field rather than from a note, because a
    /// task whose state is `dropped` already carries one and the validator already requires it.
    #[test]
    fn a_reason_in_the_tasks_own_field_counts() {
        let plan = plan_of(&document(
            "<li class=\"task\" id=\"task-a\" data-change=\"added\" data-state=\"dropped\">\
             <h3>a</h3>\
             <p data-field=\"note\">source: openspec/config.yaml</p>\
             <p data-field=\"note\">destination: none</p>\
             <p data-field=\"note\">disposition: dropped</p>\
             <p data-field=\"reason\">Nowhere in the destination holds project context.</p></li>",
        ));
        let coverage = Coverage::of(&plan, &[String::from("openspec/config.yaml")]);
        assert!(coverage.is_complete(), "{coverage:?}");
    }

    /// The check does not judge a disposition. A plan sending a specification to a destination a
    /// reader would disagree with passes, because whether a disposition is right is what a
    /// contributor reviews the plan for.
    #[test]
    fn a_disposition_a_reader_would_disagree_with_passes() {
        let plan = plan_of(&document(&item(
            "wrong",
            &[
                "source: openspec/specs/project-bootstrap/spec.md",
                // An accepted capability specification sent into a change's evidence directory,
                // which is plainly the wrong destination and plainly not this check's business.
                "destination: htmlspec/changes/archive/2026-01-01-x/evidence/spec.md",
                "disposition: carried",
            ],
        )));
        let coverage = Coverage::of(
            &plan,
            &[String::from("openspec/specs/project-bootstrap/spec.md")],
        );
        assert!(
            coverage.is_complete(),
            "the check judged a disposition: {coverage:?}"
        );
    }

    /// A plan item recording no disposition is reported, because the requirement is that a
    /// decision was recorded rather than that one was correct.
    #[test]
    fn an_item_with_no_disposition_is_reported() {
        let plan = plan_of(&document(&item(
            "a",
            &[
                "source: openspec/specs/a/spec.md",
                "destination: htmlspec/specs/a/spec.html",
            ],
        )));
        let coverage = Coverage::of(&plan, &[String::from("openspec/specs/a/spec.md")]);
        assert_eq!(
            coverage.without_disposition,
            vec!["openspec/specs/a/spec.md"]
        );
    }

    /// A word outside the four is reported with the word, because a contributor reading
    /// "unrecognized disposition" has to guess which one and what they wrote.
    #[test]
    fn a_disposition_outside_the_vocabulary_is_reported_with_the_word() {
        let plan = plan_of(&document(&item(
            "a",
            &[
                "source: openspec/specs/a/spec.md",
                "destination: htmlspec/specs/a/spec.html",
                "disposition: migrated",
            ],
        )));
        let coverage = Coverage::of(&plan, &[String::from("openspec/specs/a/spec.md")]);
        assert_eq!(
            coverage.unknown_disposition,
            vec![(
                String::from("openspec/specs/a/spec.md"),
                String::from("migrated")
            )]
        );
        assert!(coverage.report().contains("wrote migrated"));
    }

    /// One entity accounted for twice leaves it undecided which disposition applies.
    #[test]
    fn one_entity_named_by_two_items_is_reported() {
        let plan = plan_of(&document(&format!(
            "{}{}",
            item(
                "a",
                &[
                    "source: openspec/specs/a/spec.md",
                    "destination: htmlspec/specs/a/spec.html",
                    "disposition: converted",
                ]
            ),
            item(
                "again",
                &[
                    "source: openspec/specs/a/spec.md",
                    "destination: none",
                    "disposition: dropped",
                    "reason: a second opinion about the same file",
                ]
            )
        )));
        let coverage = Coverage::of(&plan, &[String::from("openspec/specs/a/spec.md")]);
        assert_eq!(coverage.duplicated, vec!["openspec/specs/a/spec.md"]);
    }

    /// A plan naming a source the survey never found was written against a different survey.
    #[test]
    fn a_plan_naming_an_unsurveyed_source_is_reported() {
        let (plan, _) = complete();
        let coverage = Coverage::of(&plan, &[String::from("openspec/specs/a/spec.md")]);
        assert_eq!(coverage.unsurveyed, vec!["openspec/config.yaml"]);
    }

    /// Two headings deriving one identifier inside one destination document is a collision the
    /// plan reports rather than resolving by suffix.
    #[test]
    fn two_headings_deriving_one_identifier_in_one_document_collide() {
        let plan = plan_of(&document(&format!(
            "{}{}",
            item(
                "one",
                &[
                    "source: openspec/changes/archive/x/specs/deterministic-formatting/spec.md",
                    "destination: htmlspec/changes/archive/x/specification.html",
                    "disposition: folded",
                    "reason: a change directory holds at most one specification",
                    "derives: Formatting is checked without writing -> req-formatting-is-checked-without-writing",
                ]
            ),
            item(
                "two",
                &[
                    "source: openspec/changes/archive/x/specs/toolchain-packaging/spec.md",
                    "destination: htmlspec/changes/archive/x/specification.html",
                    "disposition: folded",
                    "reason: a change directory holds at most one specification",
                    "derives: Formatting is checked without writing -> req-formatting-is-checked-without-writing",
                ]
            )
        )));
        let coverage = Coverage::of(&plan, &[]);
        assert_eq!(coverage.collisions.len(), 1, "{coverage:?}");
        let collision = &coverage.collisions[0];
        assert_eq!(
            collision.identifier,
            "req-formatting-is-checked-without-writing"
        );
        assert_eq!(collision.sources.len(), 2);
        assert!(coverage.report().contains("two source headings"));
    }

    /// One obligation renamed across changes holds one identifier under each heading, and that is
    /// not a collision.
    ///
    /// `project-migration` — "Identifiers are derived once and every reference resolves through
    /// that derivation" — keeps a renamed requirement's identifier: a heading named as a RENAMED
    /// `TO:` resolves to the identifier its `FROM:` heading already held. So each archived
    /// specification states the heading its own change used, and all of them carry the identifier
    /// derived from the heading the obligation was first established under. Each is a different
    /// destination, so one identifier under several headings is not two obligations sharing one.
    ///
    /// The chain is this repository's own, task 7.2 of `record-a-renamed-requirement`: the
    /// `toolchain-packaging` requirement established in `2026-08-10-html-spec`, renamed by
    /// `2026-08-17-project-bootstrap-cli` and again by `2026-08-30-finish-the-archival`, restated
    /// under its latest heading by `2026-08-31-sync-a-change-into-the-accepted-contract`, and held
    /// by the accepted specification.
    #[test]
    fn one_obligation_renamed_across_changes_is_not_a_collision() {
        const IDENTIFIER: &str = "req-a-single-command-exposes-validate-format-and-init";
        let first = "A single command exposes validate, format, and init";
        let second = "A single command exposes validate, format, bootstrap, and create";
        let third = "The command surface names its operations as OpenSpec names them";
        let statements = [
            ("openspec/changes/archive/2026-08-10-html-spec/specs/toolchain-packaging/spec.md",
             "htmlspec/changes/archive/2026-08-10-html-spec/specification.html", first),
            ("openspec/changes/archive/2026-08-17-project-bootstrap-cli/specs/toolchain-packaging/spec.md",
             "htmlspec/changes/archive/2026-08-17-project-bootstrap-cli/specification.html", second),
            ("openspec/changes/archive/2026-08-30-finish-the-archival/specs/toolchain-packaging/spec.md",
             "htmlspec/changes/archive/2026-08-30-finish-the-archival/specification.html", third),
            ("openspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract/specs/toolchain-packaging/spec.md",
             "htmlspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract/specification.html", third),
            ("openspec/specs/toolchain-packaging/spec.md",
             "htmlspec/specs/toolchain-packaging/spec.html", third),
        ];
        let items: String = statements
            .iter()
            .enumerate()
            .map(|(index, (source, destination, heading))| {
                item(
                    &format!("rename-{index}"),
                    &[
                        &format!("source: {source}"),
                        &format!("destination: {destination}"),
                        "disposition: converted",
                        &format!("derives: {heading} -> {IDENTIFIER}"),
                    ],
                )
            })
            .collect();
        let plan = plan_of(&document(&items));

        assert_eq!(plan.items.len(), statements.len());
        let identifiers: Vec<&str> = plan
            .items
            .iter()
            .flat_map(|item| item.derivations.iter().map(|d| d.identifier.as_str()))
            .collect();
        assert_eq!(
            identifiers,
            vec![IDENTIFIER; statements.len()],
            "one obligation, one identifier"
        );
        let coverage = Coverage::of(&plan, &[]);
        assert!(
            coverage.collisions.is_empty(),
            "a renamed obligation was reported as a collision: {:?}",
            coverage.collisions
        );
    }

    /// One heading stated by several changes derives one identifier in several destination
    /// documents, and that is not a collision.
    ///
    /// **This is the case the destination keying is load-bearing for**, and it is the ordinary
    /// case rather than an edge one: this repository holds 17 headings that more than one archived
    /// change states, because a change that modifies an obligation restates its heading. Five
    /// changes each state "Bootstrapping installs agent instructions for the tools a project
    /// uses", so the slug is identical five times and lands in five archived specifications.
    ///
    /// A derivation table keyed on the identifier alone reports five collisions here and sends a
    /// contributor to reconcile a conflict the source never contained. `artifact-kinds` — "A
    /// specification states the settled contract as built" — requires each archived specification
    /// to state what *its* change established, so five documents stating one heading is what a
    /// correct migration produces.
    ///
    /// The rename test above now catches a key that drops the destination as well, because a rename
    /// keeps one identifier under several headings. Before renames kept their identifier it could
    /// not: a rename gave distinct identifiers and a wrong key found no duplicate among them, which a
    /// mutation confirmed by keying on the identifier alone and leaving every other test passing.
    #[test]
    fn one_heading_stated_by_several_changes_is_not_a_collision() {
        let heading = "Bootstrapping installs agent instructions for the tools a project uses";
        let identifier =
            "req-bootstrapping-installs-agent-instructions-for-the-tools-a-project-uses";
        let changes = [
            "2026-08-17-project-bootstrap-cli",
            "2026-08-18-conform-generated-frontmatter-names",
            "2026-08-18-install-the-full-workflow-set",
            "2026-08-21-distinguish-command-names-from-skill-names",
            "2026-08-24-pass-the-description-through",
        ];
        let items: String = changes
            .iter()
            .enumerate()
            .map(|(index, change)| {
                item(
                    &format!("modified-{index}"),
                    &[
                        &format!(
                            "source: openspec/changes/archive/{change}/specs/project-bootstrap/spec.md"
                        ),
                        &format!("destination: htmlspec/changes/archive/{change}/specification.html"),
                        "disposition: folded",
                        "reason: a change directory holds at most one specification",
                        &format!("derives: {heading} -> {identifier}"),
                    ],
                )
            })
            .collect();
        let plan = plan_of(&document(&items));

        assert_eq!(plan.items.len(), 5);
        // One identifier, five times, which is the condition an identifier-keyed table trips on.
        let derived: Vec<&str> = plan
            .items
            .iter()
            .flat_map(|item| item.derivations.iter().map(|d| d.identifier.as_str()))
            .collect();
        assert_eq!(derived, vec![identifier; 5]);

        let coverage = Coverage::of(&plan, &[]);
        assert!(
            coverage.collisions.is_empty(),
            "one obligation modified across five changes was reported as a collision: {:?}",
            coverage.collisions
        );
    }

    /// A plan item with no destination is reported, and its derivations are not silently skipped.
    ///
    /// **Two failures in one omission.** The requirement says the plan records "its destination and
    /// its disposition" for each entity, and only the disposition was checked. Worse, the collision
    /// check groups derivations by destination and opened by skipping an item that had none — so
    /// every `derives:` line on such an item was excluded from collision detection.
    ///
    /// The concrete case: an agent collapsing sixteen changes' delta specifications omits the note
    /// on a few items. Coverage reads complete, the collision check silently skips them, and two
    /// distinct headings deriving one identifier inside one collapsed specification reach the staged
    /// tree — surfacing at stage 6 as a duplicate-identifier error in a document that must then be
    /// re-collapsed by hand. A review of the pull request found it.
    #[test]
    fn an_item_with_no_destination_is_reported() {
        let plan = plan_of(&document(&item(
            "a",
            &[
                "source: openspec/specs/a/spec.md",
                "disposition: converted",
                "derives: A theme is selectable -> req-a-theme-is-selectable",
            ],
        )));
        let coverage = Coverage::of(&plan, &[String::from("openspec/specs/a/spec.md")]);
        assert_eq!(
            coverage.without_destination,
            vec!["openspec/specs/a/spec.md"],
            "{coverage:?}"
        );
        assert!(!coverage.is_complete());
        assert!(coverage.report().contains("record no destination"));
    }

    /// A `derives:` note whose separator is wrong is reported rather than discarded.
    ///
    /// The separator is ` -> `, and a line the reader cannot split recorded nothing and reported
    /// nothing. So a plan misspelling it on all 120 of its requirement lines produced zero
    /// derivations, zero collisions, and a clean report — the systematic-fault shape the rest of
    /// this change exists to surface. The accepted requirement says the derivation SHALL be recorded
    /// in the plan, and a line nothing can read records nothing.
    #[test]
    fn a_derivation_note_with_the_wrong_separator_is_reported() {
        // Three spellings an author reaches for: unspaced, an em dash, and a colon.
        for wrote in [
            "derives: A theme is selectable->req-a-theme-is-selectable",
            "derives: A theme is selectable — req-a-theme-is-selectable",
            "derives: A theme is selectable: req-a-theme-is-selectable",
        ] {
            let plan = plan_of(&document(&item(
                "a",
                &[
                    "source: openspec/specs/a/spec.md",
                    "destination: htmlspec/specs/a/spec.html",
                    "disposition: converted",
                    wrote,
                ],
            )));
            assert!(
                plan.items[0].derivations.is_empty(),
                "{wrote} parsed as a derivation"
            );
            let coverage = Coverage::of(&plan, &[String::from("openspec/specs/a/spec.md")]);
            assert_eq!(
                coverage.unreadable_derivations.len(),
                1,
                "{wrote} was discarded silently: {coverage:?}"
            );
            assert!(!coverage.is_complete());
            // The report names the separator, because a contributor told only "unreadable" has to
            // guess which of three plausible spellings is wanted.
            assert!(coverage.report().contains(" -> "), "{}", coverage.report());
        }
    }

    /// One derivation line written twice states one derivation, not two obligations sharing an
    /// identifier.
    #[test]
    fn a_repeated_derivation_line_is_not_a_collision() {
        let plan = plan_of(&document(&item(
            "a",
            &[
                "source: openspec/specs/a/spec.md",
                "destination: htmlspec/specs/a/spec.html",
                "disposition: converted",
                "derives: A theme is selectable -> req-a-theme-is-selectable",
                "derives: A theme is selectable -> req-a-theme-is-selectable",
            ],
        )));
        assert!(Coverage::of(&plan, &[]).collisions.is_empty());
    }

    /// A note that is not one of the five keys is prose, so annotating a plan line breaks
    /// nothing.
    #[test]
    fn an_unrecognized_note_is_prose() {
        let plan = plan_of(&document(&item(
            "a",
            &[
                "source: openspec/specs/a/spec.md",
                "destination: htmlspec/specs/a/spec.html",
                "disposition: converted",
                "Worth reading first: this capability defines the project's root",
            ],
        )));
        let coverage = Coverage::of(&plan, &[String::from("openspec/specs/a/spec.md")]);
        assert!(coverage.is_complete(), "{coverage:?}");
    }

    /// A document of another kind is refused rather than read as an empty plan, because an empty
    /// plan accounts for nothing and would report every entity as unaccounted — a diagnostic
    /// about the source when the fault is the file that was named.
    #[test]
    fn a_document_of_another_kind_is_refused() {
        let path =
            std::env::temp_dir().join(format!("htmlspec-plan-kind-{}.html", std::process::id()));
        std::fs::write(
            &path,
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta content=\"0.1.0\" name=\"htmlspec-version\">\
             <meta content=\"proposal\" name=\"htmlspec-kind\">\
             <title>Not a plan</title></head><body><main><p>Prose.</p></main></body></html>",
        )
        .expect("written");
        let error = Plan::read(&path).expect_err("a proposal is not a plan");
        let _ = std::fs::remove_file(&path);
        assert!(error.contains("declares kind proposal"), "{error}");
        assert!(error.contains("tasks document"), "{error}");
    }

    /// The surveyed paths come out of the payload the survey emitted.
    #[test]
    fn the_surveyed_paths_are_read_from_the_payload() {
        let payload = serde_json::json!({
            "entities": [
                { "path": "openspec/config.yaml", "kind": "configuration" },
                { "path": "openspec/specs/a/spec.md", "kind": "accepted-specification" },
            ]
        });
        assert_eq!(
            surveyed_paths(&payload).expect("read"),
            vec!["openspec/config.yaml", "openspec/specs/a/spec.md"]
        );
    }

    /// A payload with no entities array is refused, naming the command that produces one.
    #[test]
    fn a_payload_that_is_not_a_survey_is_refused() {
        let error = surveyed_paths(&serde_json::json!({ "counts": {} })).expect_err("refused");
        assert!(error.contains("migrate survey --json"), "{error}");
    }
}
