//! The change markup in a unit's statements and scenario entries, read as runs.
//!
//! The projection drops what a `del` holds and keeps what an `ins` holds, so the model is the
//! current text and nothing in it says where an edit was (sections 8.6.1 and 8.6.2). A view that
//! shows an author's marks needs the positions the projection drops, and this reader keeps them.
//! It walks a unit as the projection does and selects the same statement paragraphs and scenario
//! entries, so which paragraph is a statement stays the projection's rule and is not defined twice.
//!
//! It adds nothing to the model. `mark-the-words-a-modification-changed` kept the marks out of the
//! model on purpose, because runs in a field break every reader of that field to serve one view.

use std::collections::BTreeMap;

use kuchikiki::iter::NodeIterator;
use kuchikiki::{ElementData, NodeData, NodeRef};

use super::{
    attribute, element_name, encloses_requirement, enclosing_unit, is_deletion,
    is_deletion_element, own_descendants, own_nodes, reserved_classes, scenario_label,
    text_free_element, ScenarioField, HTML_NAMESPACE,
};
use crate::model::UnitKind;

/// One stretch of a statement paragraph or a scenario entry, as the change markup marks it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Run {
    /// Text that no `ins` or `del` within the unit encloses.
    Kept(String),
    /// Text an `ins` encloses: words the change added.
    Inserted(String),
    /// Text a `del` encloses: words the change deleted.
    Deleted(String),
}

/// One statement paragraph or scenario entry, as runs in document order.
///
/// Each run's whitespace is collapsed to single spaces and left untrimmed, so the runs of one
/// side, joined and normalized, are that side's text. Never without text: a paragraph holding
/// only whitespace projects nothing and is read as nothing here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkedText(Vec<Run>);

impl MarkedText {
    /// The text of `runs`, or `None` where neither the current text nor the text before the edit
    /// holds a word.
    ///
    /// Each run's whitespace is collapsed, adjacent runs of one kind are joined, and empty runs are
    /// dropped, so two readings of the same words give the same runs.
    pub fn new(runs: Vec<Run>) -> Option<Self> {
        let mut joined: Vec<Run> = Vec::new();
        for run in runs {
            let (kind, text) = run.parts();
            let text = collapsed(text);
            if text.is_empty() {
                continue;
            }
            match joined.last_mut() {
                Some(last) if last.parts().0 == kind => last.push_str(&text),
                _ => joined.push(kind.run(text)),
            }
        }
        let text = Self(joined);
        (!text.current().is_empty() || !text.before().is_empty()).then_some(text)
    }

    /// The runs, in document order.
    pub fn runs(&self) -> &[Run] {
        &self.0
    }

    /// The text as it now reads, which is what the projection holds: each kept and each inserted
    /// run, normalized.
    pub fn current(&self) -> String {
        self.normalized(|kind| kind != Kind::Deleted)
    }

    /// The text as it read before the edit: each kept and each deleted run, normalized.
    pub fn before(&self) -> String {
        self.normalized(|kind| kind != Kind::Inserted)
    }

    /// Whether an inserted or a deleted run holds a word.
    pub fn is_marked(&self) -> bool {
        self.0.iter().any(|run| {
            let (kind, text) = run.parts();
            kind != Kind::Kept && !text.trim().is_empty()
        })
    }

    fn normalized(&self, keep: impl Fn(Kind) -> bool) -> String {
        let text: String = self
            .0
            .iter()
            .filter(|run| keep(run.parts().0))
            .map(|run| run.parts().1)
            .collect();
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// Which of the three a run is, apart from its text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Kept,
    Inserted,
    Deleted,
}

impl Kind {
    fn run(self, text: String) -> Run {
        match self {
            Self::Kept => Run::Kept(text),
            Self::Inserted => Run::Inserted(text),
            Self::Deleted => Run::Deleted(text),
        }
    }

    /// The kind of what an element of kind `element` holds, inside a run of this kind.
    ///
    /// A `del` decides, wherever it is: section 8.6.1 has its content supply no field, so nothing
    /// inside one is current text, an `ins` inside it included. An `ins` makes what it holds
    /// inserted, unless a `del` around it already made it deleted.
    fn within(self, element: Option<Self>) -> Self {
        match (self, element) {
            (_, Some(Self::Deleted)) | (Self::Deleted, _) => Self::Deleted,
            (_, Some(Self::Inserted)) => Self::Inserted,
            (kind, _) => kind,
        }
    }
}

impl Run {
    fn parts(&self) -> (Kind, &str) {
        match self {
            Self::Kept(text) => (Kind::Kept, text),
            Self::Inserted(text) => (Kind::Inserted, text),
            Self::Deleted(text) => (Kind::Deleted, text),
        }
    }

    fn push_str(&mut self, more: &str) {
        match self {
            Self::Kept(text) | Self::Inserted(text) | Self::Deleted(text) => text.push_str(more),
        }
    }
}

/// `text` with each run of whitespace collapsed to one space, and not trimmed.
fn collapsed(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;
    for character in text.chars() {
        if character.is_whitespace() {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(character);
            in_space = false;
        }
    }
    out
}

/// The statements and scenario entries of one unit that carries change markup in them.
///
/// A requirement fills `statements`, and a scenario fills `given`, `when`, and `then`. Each list
/// is in document order and includes a paragraph or an entry wholly inside a `del`, which the
/// projection leaves out.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MarkedUnit {
    pub statements: Vec<MarkedText>,
    pub given: Vec<MarkedText>,
    pub when: Vec<MarkedText>,
    pub then: Vec<MarkedText>,
}

/// Every requirement and scenario in `document` whose statements or scenario entries carry an
/// `ins` or a `del`, by identifier.
///
/// A unit with no identifier is not listed, because nothing could look it up. A unit whose marks
/// sit only in a note or a heading is not listed either: neither is content a comparison reads.
/// Nor is a unit the projection does not place (section 4.3): a requirement nested inside another
/// unit, or a scenario no requirement encloses.
pub fn marked_units(document: &NodeRef) -> BTreeMap<String, MarkedUnit> {
    let mut marked = BTreeMap::new();
    for unit in document
        .descendants()
        .elements()
        .map(|element| element.as_node().clone())
    {
        let kinds = reserved_classes(&unit);
        let Some(id) = attribute(&unit, "id").filter(|id| !id.is_empty()) else {
            continue;
        };
        let read = match kinds.as_slice() {
            [UnitKind::Requirement] if enclosing_unit(&unit).is_none() => statements(&unit),
            [UnitKind::Scenario] if encloses_requirement(&unit) => entries(&unit),
            _ => continue,
        };
        if read.carries_markup {
            marked.insert(id, read.unit);
        }
    }
    marked
}

/// One unit as the reader read it, and whether any of its content carries change markup.
struct Read {
    unit: MarkedUnit,
    carries_markup: bool,
}

/// A requirement's statement paragraphs, as the projection selects them (section 4.0), and those
/// wholly inside a `del`, which it leaves out.
fn statements(unit: &NodeRef) -> Read {
    let mut read = Read {
        unit: MarkedUnit::default(),
        carries_markup: false,
    };
    for (paragraph, _) in own_nodes(unit) {
        if element_name(&paragraph).as_deref() != Some("p") {
            continue;
        }
        if attribute(&paragraph, "data-field").is_some_and(|field| field != "statement") {
            continue;
        }
        let (text, marked) = marked_text(&paragraph, unit);
        read.carries_markup |= marked;
        read.unit.statements.extend(text);
    }
    read
}

/// A scenario's entries, from the description list the projection reads (section 4.2), each under
/// the label before it.
fn entries(unit: &NodeRef) -> Read {
    let mut read = Read {
        unit: MarkedUnit::default(),
        carries_markup: false,
    };
    let Some(list) = own_descendants(unit)
        .into_iter()
        .find(|node| element_name(node).as_deref() == Some("dl"))
    else {
        return read;
    };
    let mut label = None;
    for node in list.children().filter(|node| node.as_element().is_some()) {
        match element_name(&node).as_deref() {
            Some("dt") => label = scenario_label(&node),
            Some("dd") => {
                let (text, marked) = marked_text(&node, unit);
                read.carries_markup |= marked;
                let Some(text) = text else {
                    continue;
                };
                match label {
                    Some(ScenarioField::Given) => read.unit.given.push(text),
                    Some(ScenarioField::When) => read.unit.when.push(text),
                    Some(ScenarioField::Then) => read.unit.then.push(text),
                    None => {}
                }
            }
            _ => {}
        }
    }
    read
}

/// The text of `content`, one paragraph or entry of `unit`, as runs, and whether an `ins` or a
/// `del` inside it or around it below the unit marks it.
fn marked_text(content: &NodeRef, unit: &NodeRef) -> (Option<MarkedText>, bool) {
    let mut marked = false;
    let mut kind = Kind::Kept;
    // Outermost first, so a `del` anywhere around the content decides.
    let around: Vec<NodeRef> = content
        .ancestors()
        .take_while(|ancestor| ancestor != unit)
        .collect();
    for ancestor in around.iter().rev() {
        if let Some(element) = change_markup(ancestor) {
            marked = true;
            kind = kind.within(Some(element));
        }
    }
    let mut runs = Vec::new();
    collect_runs(content, kind, &mut runs, &mut marked);
    (MarkedText::new(runs), marked)
}

/// The kind of change markup `node` is, if it is one: HTML's `ins` or `del`, and nothing of that
/// name in another namespace (section 8.6).
fn change_markup(node: &NodeRef) -> Option<Kind> {
    if is_deletion(node) {
        return Some(Kind::Deleted);
    }
    node.as_element()
        .filter(|element| is_insertion_element(element))
        .map(|_| Kind::Inserted)
}

fn is_insertion_element(element: &ElementData) -> bool {
    &*element.name.ns == HTML_NAMESPACE && element.name.local.as_ref() == "ins"
}

/// Appends the text of `node` to `runs`, each stretch as `kind` or as the change markup around it
/// makes it, skipping what section 8.5 and section 6.4 give no text.
fn collect_runs(node: &NodeRef, kind: Kind, runs: &mut Vec<Run>, marked: &mut bool) {
    match node.data() {
        NodeData::Text(text) => runs.push(kind.run(text.borrow().clone())),
        NodeData::Element(element) => {
            if text_free_element(element.name.local.as_ref()) {
                return;
            }
            let markup = if is_deletion_element(element) {
                Some(Kind::Deleted)
            } else if is_insertion_element(element) {
                Some(Kind::Inserted)
            } else {
                None
            };
            *marked |= markup.is_some();
            let within = kind.within(markup);
            for child in node.children() {
                collect_runs(&child, within, runs, marked);
            }
        }
        NodeData::Comment(_) | NodeData::Doctype(_) => {}
        _ => {
            for child in node.children() {
                collect_runs(&child, kind, runs, marked);
            }
        }
    }
}
