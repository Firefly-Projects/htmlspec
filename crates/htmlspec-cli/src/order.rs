//! What must run first: each runbook's prerequisites and their state now, for the listing and the
//! graph.
//!
//! The change `require-a-run-of-another-runbook`, design decision 8. `htmlspec list --runbooks`, the
//! listing `run` prints when it is named with no runbook, and `list --runbooks --graph` answer two
//! questions from one reading of the project: what must run first, and what can start now. This
//! module is that reading. It decides nothing the start path does not decide: a declaration's
//! records are read by [`crate::prerequisites::read_records`], its name is matched byte for byte by
//! [`crate::prerequisites::runbook_names`], and its declarations are read by
//! [`crate::prerequisites::declarations_of`], so the listing and a start cannot disagree about one
//! runbook.
//!
//! What it adds is the view across the project. A start asks about one runbook and refuses a cycle
//! through it. The listing shows every runbook at once, so it names every cycle, and it marks a
//! prerequisite as unavailable where a start would report the check unavailable, rather than reading
//! records it could not act on.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::prerequisites::{declarations_of, read_records, runbook_names, Declarations, NAMED};
use crate::terminal::shown;
use crate::version::RunbookVersion;

/// Which state a prerequisite is in, without what the state holds.
///
/// The listing's `state`. Ordered from best to worst, so two declarations naming one prerequisite
/// merge into one edge of the graph by taking the worse.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Kind {
    /// A finished, passed run of the named runbook satisfies it.
    Met,
    /// The named runbook exists and no record of it satisfies the declaration.
    Unmet,
    /// A runbook its prerequisite reaches cannot have its declarations read, so a start reports the
    /// check unavailable: the word preflight's `run` object uses for the same case.
    Unavailable,
    /// No runbook of the project has the declared name, byte for byte.
    Missing,
    /// The prerequisite leads back to the runbook that declares it.
    Cycle,
    /// The runbook names itself.
    SelfReference,
}

impl Kind {
    /// The word the payload names this state by.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Met => "met",
            Self::Unmet => "unmet",
            Self::Unavailable => "unavailable",
            Self::Missing => "missing",
            Self::Cycle => "cycle",
            Self::SelfReference => "self",
        }
    }
}

/// The state of one prerequisite now, with the records that decided it.
///
/// **Only `Met` and `Unmet` hold records**, because only they were decided from records. A
/// prerequisite in a cycle, one naming its own runbook, one naming no runbook, and one a start would
/// report unavailable are not decided by any record: a start refuses the first two as an invalid
/// contract and reads no record for the last two. So those states carry none, and a listing cannot
/// say a record satisfies a check that preflight refuses outright (PR #89's review).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum State {
    /// A finished, passed run satisfies it.
    Met {
        /// Up to [`NAMED`] records that satisfy it, in name order.
        satisfied_by: Vec<String>,
        /// How many records satisfy it in all, at least one.
        satisfying: usize,
        /// Every record of the named runbook still under way, in name order.
        open_runs: Vec<String>,
    },
    /// No record satisfies it.
    Unmet {
        /// Every record of the named runbook still under way, in name order.
        open_runs: Vec<String>,
    },
    /// A runbook it reaches cannot have its declarations read.
    Unavailable,
    /// No runbook has the name.
    Missing,
    /// It leads back to the runbook that declares it.
    Cycle,
    /// It names its own runbook.
    SelfReference,
}

impl State {
    /// Which state this is.
    pub(crate) fn kind(&self) -> Kind {
        match self {
            Self::Met { .. } => Kind::Met,
            Self::Unmet { .. } => Kind::Unmet,
            Self::Unavailable => Kind::Unavailable,
            Self::Missing => Kind::Missing,
            Self::Cycle => Kind::Cycle,
            Self::SelfReference => Kind::SelfReference,
        }
    }

    /// The records named as satisfying it: some only where it is met.
    pub(crate) fn satisfied_by(&self) -> &[String] {
        match self {
            Self::Met { satisfied_by, .. } => satisfied_by,
            _ => &[],
        }
    }

    /// How many records satisfy it: none unless it is met.
    pub(crate) fn satisfying(&self) -> usize {
        match self {
            Self::Met { satisfying, .. } => *satisfying,
            _ => 0,
        }
    }

    /// The runs of the named runbook still under way: some only where it is met or unmet.
    pub(crate) fn open_runs(&self) -> &[String] {
        match self {
            Self::Met { open_runs, .. } | Self::Unmet { open_runs } => open_runs,
            _ => &[],
        }
    }
}

/// One declaration of a run check, as the listing shows it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Prerequisite {
    /// The requirement that holds the declaration.
    pub requirement: String,
    /// The runbook name it declares, exactly as written.
    pub runbook: String,
    /// The lowest release whose runs count, where declared.
    pub at_least: Option<RunbookVersion>,
    /// Its state now, with the records that decided it.
    pub state: State,
}

/// Whether a runbook's declarations could be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Readable {
    /// Read, and every declaration is one this program reads.
    Yes,
    /// Read, and a declaration is malformed. Holds why.
    Malformed(String),
    /// Not read: the specification is absent, does not validate, or needs recovery. Holds why.
    Unreadable(String),
}

impl Readable {
    /// The word the payload names this by.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Yes => "readable",
            Self::Malformed(_) => "malformed",
            Self::Unreadable(_) => "unreadable",
        }
    }

    /// Why the declarations could not be read, where they could not.
    pub(crate) fn reason(&self) -> Option<&str> {
        match self {
            Self::Yes => None,
            Self::Malformed(reason) | Self::Unreadable(reason) => Some(reason),
        }
    }
}

/// One runbook's prerequisites.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunbookOrder {
    /// The runbook's directory name.
    pub name: String,
    /// Whether its declarations could be read.
    pub declarations: Readable,
    /// Its run checks, in document order. Empty where its declarations could not be read, because
    /// what they declare is not known.
    pub prerequisites: Vec<Prerequisite>,
}

impl RunbookOrder {
    /// Whether every prerequisite is met now: never, where the declarations could not be read, and
    /// always, for a runbook with readable declarations and no run checks.
    pub(crate) fn met(&self) -> bool {
        self.declarations == Readable::Yes
            && self
                .prerequisites
                .iter()
                .all(|prerequisite| prerequisite.state.kind() == Kind::Met)
    }
}

/// The project's order: every runbook, in the order given, and every cycle among them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Order {
    /// One entry per runbook.
    pub runbooks: Vec<RunbookOrder>,
    /// Every cycle of run checks, each as the runbooks in the order they require one another,
    /// starting from the least name, in name order of their first runbook.
    pub cycles: Vec<Vec<String>>,
}

/// Reads the order of the runbooks named `names` under `runbooks`.
///
/// `names` are the listing's runbooks, in its order. Every declaration is decided once, here, so the
/// listing, the bare `run` listing, and the graph show one answer.
pub(crate) fn read(runbooks: &Path, names: &[String]) -> Order {
    let existing: BTreeSet<String> = runbook_names(runbooks).into_iter().collect();
    let declarations: BTreeMap<String, Declarations> = names
        .iter()
        .map(|name| (name.clone(), declarations_of(runbooks, name)))
        .collect();

    // What each readable runbook requires, among runbooks that exist. A declared name that is no
    // runbook leads nowhere, and an unreadable runbook leads nowhere that can be known.
    let requires: BTreeMap<&str, BTreeSet<&str>> = declarations
        .iter()
        .filter_map(|(name, declared)| match declared {
            Declarations::Readable(list) => Some((
                name.as_str(),
                list.iter()
                    .map(|declaration| declaration.runbook.as_str())
                    .filter(|named| existing.contains(*named))
                    .collect(),
            )),
            _ => None,
        })
        .collect();
    let unreadable: BTreeSet<&str> = declarations
        .iter()
        .filter(|(_, declared)| !matches!(declared, Declarations::Readable(_)))
        .map(|(name, _)| name.as_str())
        .collect();

    let mut ordered = Vec::with_capacity(names.len());
    for name in names {
        let entry = match &declarations[name] {
            Declarations::Readable(list) => RunbookOrder {
                name: name.clone(),
                declarations: Readable::Yes,
                prerequisites: list
                    .iter()
                    .map(|declaration| {
                        let named = declaration.runbook.as_str();
                        let reached = reach(&requires, named);
                        // Records are read only where they decide the state: a start reads none
                        // for the other four, so the listing reads none either.
                        let state = if named == name {
                            State::SelfReference
                        } else if !existing.contains(named) {
                            State::Missing
                        } else if reached.contains(name.as_str()) {
                            State::Cycle
                        } else if reached.iter().any(|runbook| unreadable.contains(runbook))
                            || unreadable.contains(named)
                        {
                            State::Unavailable
                        } else {
                            decided(runbooks, named, declaration.at_least)
                        };
                        Prerequisite {
                            requirement: declaration.requirement.to_string(),
                            runbook: named.to_owned(),
                            at_least: declaration.at_least,
                            state,
                        }
                    })
                    .collect(),
            },
            Declarations::Malformed(reason) => RunbookOrder {
                name: name.clone(),
                declarations: Readable::Malformed(reason.clone()),
                prerequisites: Vec::new(),
            },
            Declarations::Unreadable(reason) => RunbookOrder {
                name: name.clone(),
                declarations: Readable::Unreadable(reason.clone()),
                prerequisites: Vec::new(),
            },
        };
        ordered.push(entry);
    }

    Order {
        runbooks: ordered,
        cycles: cycles(&requires),
    }
}

/// A prerequisite decided from the records of the runbook it names: met or unmet.
///
/// The same reading a start makes, [`read_records`], naming at most [`NAMED`] satisfying records and
/// every open run, as preflight's `run` object does.
fn decided(runbooks: &Path, named: &str, at_least: Option<RunbookVersion>) -> State {
    let records = read_records(&runbooks.join(named).join("runs"), at_least);
    let open_runs: Vec<String> = records.open.iter().map(|run| run.record.clone()).collect();
    if records.satisfying.is_empty() {
        return State::Unmet { open_runs };
    }
    State::Met {
        satisfied_by: records
            .satisfying
            .iter()
            .take(NAMED)
            .map(|record| record.record.clone())
            .collect(),
        satisfying: records.satisfying.len(),
        open_runs,
    }
}

/// Every runbook reachable from `start` through readable run checks, `start` included.
fn reach<'a>(requires: &BTreeMap<&'a str, BTreeSet<&'a str>>, start: &'a str) -> BTreeSet<&'a str> {
    let mut reached = BTreeSet::new();
    let mut pending = vec![start];
    while let Some(at) = pending.pop() {
        if !reached.insert(at) {
            continue;
        }
        if let Some(next) = requires.get(at) {
            pending.extend(next.iter().copied());
        }
    }
    reached
}

/// One representative cycle of run checks for each strongly connected component that holds one.
///
/// A component holds a cycle when it has more than one runbook, or one that requires itself. Its
/// representative is the shortest cycle through its least runbook, found breadth first over names
/// in order, so the same project gives the same cycles everywhere; it starts from that least name
/// and follows the order the runbooks require one another. Components are listed by their least
/// runbook. A runbook that requires itself is a cycle of one.
///
/// **One per component, not every cycle.** Every elementary cycle grows factorially: nine runbooks
/// that each require all the others hold 125,664 of them, a 14 MB payload, for an answer a person
/// acts on the same way, by breaking the component (the pre-archival review of 2026-10-27). Two
/// runbooks are in the same listed cycle's component exactly when each leads back to the other,
/// which is what a person needs to know.
fn cycles(requires: &BTreeMap<&str, BTreeSet<&str>>) -> Vec<Vec<String>> {
    let component = components(requires);
    let mut members: BTreeMap<usize, BTreeSet<&str>> = BTreeMap::new();
    for (name, id) in &component {
        members.entry(*id).or_default().insert(name);
    }
    let mut found: Vec<Vec<String>> = members
        .values()
        .filter_map(|names| {
            let least = *names.first()?;
            let loops = requires.get(least).is_some_and(|next| next.contains(least));
            (names.len() > 1 || loops).then(|| shortest_cycle(requires, names, least))
        })
        .flatten()
        .collect();
    found.sort();
    found
}

/// The shortest cycle from `start` back to itself through `names` only, found breadth first over
/// names in order, as the runbook names along it.
fn shortest_cycle(
    requires: &BTreeMap<&str, BTreeSet<&str>>,
    names: &BTreeSet<&str>,
    start: &str,
) -> Option<Vec<String>> {
    let mut came_from: BTreeMap<&str, &str> = BTreeMap::new();
    let mut queue = std::collections::VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for &named in requires.get(at).into_iter().flatten() {
            if named == start {
                let mut path = vec![at.to_owned()];
                let mut step = at;
                while step != start {
                    step = came_from.get(step)?;
                    path.push(step.to_owned());
                }
                path.reverse();
                return Some(path);
            }
            if names.contains(named) && !came_from.contains_key(named) && named != start {
                came_from.insert(named, at);
                queue.push_back(named);
            }
        }
    }
    None
}

/// The strongly connected component of every runbook the run checks mention, by Kosaraju's method.
///
/// Iterative rather than recursive, so a long chain of runbooks cannot exhaust the stack. Two
/// runbooks share a component exactly when each requires, directly or not, a run of the other.
fn components<'a>(requires: &BTreeMap<&'a str, BTreeSet<&'a str>>) -> BTreeMap<&'a str, usize> {
    let mut nodes: BTreeSet<&str> = requires.keys().copied().collect();
    nodes.extend(requires.values().flatten().copied());
    let empty = BTreeSet::new();
    let next = |node: &str| requires.get(node).unwrap_or(&empty);

    // The order each node finishes in a depth-first walk of the run checks.
    let mut finished: Vec<&str> = Vec::with_capacity(nodes.len());
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for &root in &nodes {
        if !seen.insert(root) {
            continue;
        }
        let mut stack: Vec<(&str, std::collections::btree_set::Iter<'_, &str>)> =
            vec![(root, next(root).iter())];
        while let Some((node, children)) = stack.last_mut() {
            match children.next() {
                Some(&child) => {
                    if seen.insert(child) {
                        stack.push((child, next(child).iter()));
                    }
                }
                None => {
                    finished.push(node);
                    stack.pop();
                }
            }
        }
    }

    // The checks reversed, walked from the last node to finish: each walk is one component.
    let mut reverse: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (&node, required) in requires {
        for &named in required {
            reverse.entry(named).or_default().push(node);
        }
    }
    let mut component: BTreeMap<&str, usize> = BTreeMap::new();
    for (id, &root) in finished.iter().rev().enumerate() {
        if component.contains_key(root) {
            continue;
        }
        let mut stack = vec![root];
        component.insert(root, id);
        while let Some(node) = stack.pop() {
            for &before in reverse.get(node).map(Vec::as_slice).unwrap_or(&[]) {
                if !component.contains_key(before) {
                    component.insert(before, id);
                    stack.push(before);
                }
            }
        }
    }
    component
}

/// The text a prerequisite's state reads as in the human listing.
pub(crate) fn describe(prerequisite: &Prerequisite) -> String {
    match &prerequisite.state {
        State::Met {
            satisfied_by,
            satisfying,
            open_runs,
        } => {
            let named: Vec<String> = satisfied_by
                .iter()
                .map(|record| crate::terminal::printable(record))
                .collect();
            let rest = satisfying.saturating_sub(named.len());
            let met = if rest > 0 {
                format!("met by {} and {rest} more", named.join(", "))
            } else {
                format!("met by {}", named.join(", "))
            };
            // A run still under way beside a passed one is named, as preflight names it, so a
            // dependent run is not started mid-way through a newer prerequisite run unseen.
            match open_runs.len() {
                0 => met,
                1 => format!("{met}, 1 run still open"),
                open => format!("{met}, {open} runs still open"),
            }
        }
        State::Unmet { open_runs } => match open_runs.len() {
            0 => String::from("unmet, no finished and passed run"),
            1 => String::from("unmet, 1 run still open"),
            open => format!("unmet, {open} runs still open"),
        },
        State::Unavailable => {
            String::from("unavailable, a runbook it reaches has declarations that cannot be read")
        }
        State::Missing => String::from("missing, this project has no runbook of that name"),
        State::Cycle => String::from("cycle, it leads back to this runbook"),
        State::SelfReference => String::from("self, the runbook requires itself"),
    }
}

/// The lines the human listing shows beneath one runbook's row.
pub(crate) fn lines(runbook: &RunbookOrder) -> Vec<String> {
    if let Some(reason) = runbook.declarations.reason() {
        return vec![format!(
            "declarations cannot be read: {}",
            crate::terminal::printable(reason)
        )];
    }
    runbook
        .prerequisites
        .iter()
        .map(|prerequisite| {
            let at_least = prerequisite
                .at_least
                .map(|release| format!(" at least {release}"))
                .unwrap_or_default();
            format!(
                "requires {}{at_least}: {}",
                crate::terminal::printable(&prerequisite.runbook),
                describe(prerequisite)
            )
        })
        .collect()
}

/// The style a graph edge is drawn in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Style {
    Met,
    Unmet,
    Cycle,
}

impl Style {
    /// The style of an edge in state `kind`.
    fn of(kind: Kind) -> Self {
        match kind {
            Kind::Met => Self::Met,
            Kind::Unmet | Kind::Unavailable | Kind::Missing => Self::Unmet,
            Kind::Cycle | Kind::SelfReference => Self::Cycle,
        }
    }

    /// The Mermaid `linkStyle` declarations for this style.
    fn declaration(self) -> &'static str {
        match self {
            Self::Met => "stroke:#2e6b3a,stroke-width:2px",
            Self::Unmet => "stroke:#a33333,stroke-width:2px,stroke-dasharray:5 4",
            Self::Cycle => "stroke:#7a3fa3,stroke-width:3px",
        }
    }
}

/// A runbook name as a Mermaid label's text.
///
/// Two steps, as design decision 8 sets out: every character a terminal would not show as itself
/// is written in [`shown`]'s code-point form, so a newline or an escape can never reach the label,
/// and then each character Mermaid or HTML would read is written as a Mermaid entity code. Done one
/// character at a time, so an entity code's own `#` is never escaped twice.
///
/// Two kinds of character Mermaid reads even inside a quoted label, both found by PR #89's review.
/// `%` is written `#37;`, because `%%` begins a directive or a comment wherever it stands, and a
/// directive in a name could restyle or break the whole diagram. And the characters Mermaid uses
/// internally to carry an entity code through its parser, `ﬂ`, `°`, `¶`, and `ß`, are written in
/// [`shown`]'s code-point form: Mermaid decodes them back into `&`, `&#`, and `;`, so a name holding
/// them would render as a different name.
pub(crate) fn label(text: &str) -> String {
    shown(text)
        .chars()
        .map(|character| match character {
            '"' => String::from("#quot;"),
            '#' => String::from("#35;"),
            '%' => String::from("#37;"),
            '`' => String::from("#96;"),
            '<' => String::from("#lt;"),
            '>' => String::from("#gt;"),
            '&' => String::from("#amp;"),
            '\u{FB02}' | '\u{B0}' | '\u{B6}' | '\u{DF}' => {
                format!("#lt;U+{:04X}#gt;", u32::from(character))
            }
            other => other.to_string(),
        })
        .collect()
}

/// The order as a Mermaid flowchart, laid out as design decision 8 sets out.
///
/// Nodes are the project's runbooks and every declared name that matches none, in byte order,
/// numbered `r1` onwards, so a name never becomes an identifier. One edge from each prerequisite to
/// the runbook that declares it, sorted by prerequisite and then by runbook, with two declarations
/// of one prerequisite merged into one edge. Then the classes the diagram uses and the styles of
/// its edges, each in a fixed order.
pub(crate) fn graph(order: &Order) -> String {
    let runbooks: BTreeSet<&str> = order
        .runbooks
        .iter()
        .map(|runbook| runbook.name.as_str())
        .collect();
    let missing: BTreeSet<&str> = order
        .runbooks
        .iter()
        .flat_map(|runbook| runbook.prerequisites.iter())
        .filter(|prerequisite| prerequisite.state.kind() == Kind::Missing)
        .map(|prerequisite| prerequisite.runbook.as_str())
        .filter(|name| !runbooks.contains(name))
        .collect();
    let malformed: BTreeSet<&str> = order
        .runbooks
        .iter()
        .filter(|runbook| runbook.declarations != Readable::Yes)
        .map(|runbook| runbook.name.as_str())
        .collect();
    let nodes: Vec<&str> = runbooks.union(&missing).copied().collect();
    let identifier: BTreeMap<&str, String> = nodes
        .iter()
        .enumerate()
        .map(|(index, name)| (*name, format!("r{}", index + 1)))
        .collect();

    // One edge per (prerequisite, runbook), merging declarations: the worse state, and every
    // distinct lowest release.
    let mut edges: BTreeMap<(&str, &str), (Kind, BTreeSet<RunbookVersion>)> = BTreeMap::new();
    for runbook in &order.runbooks {
        for prerequisite in &runbook.prerequisites {
            let edge = edges
                .entry((prerequisite.runbook.as_str(), runbook.name.as_str()))
                .or_insert((Kind::Met, BTreeSet::new()));
            edge.0 = edge.0.max(prerequisite.state.kind());
            if let Some(release) = prerequisite.at_least {
                edge.1.insert(release);
            }
        }
    }

    let mut out = String::from("flowchart LR\n");
    for name in &nodes {
        let suffix = if missing.contains(name) {
            " (no such runbook)"
        } else if malformed.contains(name) {
            " (declarations cannot be read)"
        } else {
            ""
        };
        out.push_str(&format!(
            "  {}[\"{}{suffix}\"]\n",
            identifier[name],
            label(name)
        ));
    }
    let mut styles: Vec<(Style, Vec<usize>)> = vec![
        (Style::Met, Vec::new()),
        (Style::Unmet, Vec::new()),
        (Style::Cycle, Vec::new()),
    ];
    for (index, ((from, to), (kind, releases))) in edges.iter().enumerate() {
        let releases: Vec<String> = releases.iter().map(ToString::to_string).collect();
        let text = match kind {
            Kind::SelfReference => String::from("requires itself"),
            Kind::Cycle => String::from("cycle"),
            Kind::Unavailable => String::from("cannot be checked"),
            Kind::Met | Kind::Unmet | Kind::Missing => {
                let word = if *kind == Kind::Met { "met" } else { "unmet" };
                if releases.is_empty() {
                    word.to_owned()
                } else {
                    format!("{word}, at least {}", releases.join(" and "))
                }
            }
        };
        out.push_str(&format!(
            "  {} -->|\"{text}\"| {}\n",
            identifier[from], identifier[to]
        ));
        let style = Style::of(*kind);
        if let Some((_, indices)) = styles.iter_mut().find(|(candidate, _)| *candidate == style) {
            indices.push(index);
        }
    }
    let classes: Vec<(&str, &str, Vec<&String>)> = vec![
        (
            "missing",
            "stroke-dasharray:5 4",
            missing.iter().map(|name| &identifier[name]).collect(),
        ),
        (
            "malformed",
            "stroke:#a33333,stroke-width:2px",
            malformed.iter().map(|name| &identifier[name]).collect(),
        ),
    ];
    for (class, declaration, members) in &classes {
        if !members.is_empty() {
            out.push_str(&format!("  classDef {class} {declaration}\n"));
        }
    }
    for (class, _, members) in &classes {
        if !members.is_empty() {
            // Ascending already: identifiers are numbered in name order, and each class's
            // members come from a set kept in name order.
            let joined: Vec<&str> = members
                .iter()
                .map(|identifier| identifier.as_str())
                .collect();
            out.push_str(&format!("  class {} {class}\n", joined.join(",")));
        }
    }
    for (style, indices) in &styles {
        if !indices.is_empty() {
            let joined: Vec<String> = indices.iter().map(ToString::to_string).collect();
            out.push_str(&format!(
                "  linkStyle {} {}\n",
                joined.join(","),
                style.declaration()
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A met state with one satisfying record and no open run, for the graph tests.
    fn met() -> State {
        State::Met {
            satisfied_by: vec![String::from("nightly")],
            satisfying: 1,
            open_runs: Vec::new(),
        }
    }

    /// An unmet state with no open run, for the graph tests.
    fn unmet() -> State {
        State::Unmet {
            open_runs: Vec::new(),
        }
    }

    fn prerequisite(runbook: &str, state: State, at_least: Option<&str>) -> Prerequisite {
        Prerequisite {
            requirement: format!("req-{runbook}"),
            runbook: runbook.to_owned(),
            at_least: at_least.map(|text| RunbookVersion::parse(text).expect("a version")),
            state,
        }
    }

    fn runbook(name: &str, prerequisites: Vec<Prerequisite>) -> RunbookOrder {
        RunbookOrder {
            name: name.to_owned(),
            declarations: Readable::Yes,
            prerequisites,
        }
    }

    /// The design's example, drawn from the order alone.
    #[test]
    fn the_designs_example_is_drawn_line_for_line() {
        let order = Order {
            runbooks: vec![
                runbook("backup", vec![]),
                runbook("create-account", vec![]),
                runbook(
                    "deploy-service",
                    vec![prerequisite("provision-database", unmet(), None)],
                ),
                runbook(
                    "migrate",
                    vec![prerequisite("backup", met(), Some("1.4.0"))],
                ),
                runbook(
                    "provision-database",
                    vec![prerequisite("create-account", unmet(), None)],
                ),
            ],
            cycles: Vec::new(),
        };
        assert_eq!(
            graph(&order),
            "flowchart LR\n  r1[\"backup\"]\n  r2[\"create-account\"]\n  r3[\"deploy-service\"]\n  \
             r4[\"migrate\"]\n  r5[\"provision-database\"]\n  r1 -->|\"met, at least 1.4.0\"| r4\n  \
             r2 -->|\"unmet\"| r5\n  r5 -->|\"unmet\"| r3\n  linkStyle 0 \
             stroke:#2e6b3a,stroke-width:2px\n  linkStyle 1,2 \
             stroke:#a33333,stroke-width:2px,stroke-dasharray:5 4\n"
        );
    }

    /// Every character Mermaid or HTML would read is an entity code, and every hidden one a code
    /// point, so no raw one and no line break reaches a label.
    #[test]
    fn a_label_holds_no_raw_character_mermaid_reads() {
        assert_eq!(
            label("a\"b#c`d<e>f&g"),
            "a#quot;b#35;c#96;d#lt;e#gt;f#amp;g"
        );
        assert_eq!(label("%%{init}%%"), "#37;#37;{init}#37;#37;");
        assert_eq!(
            label("\u{FB02}\u{B0}x\u{B6}\u{DF}"),
            "#lt;U+FB02#gt;#lt;U+00B0#gt;x#lt;U+00B6#gt;#lt;U+00DF#gt;"
        );
        assert_eq!(label("x\u{1b}y"), "x#lt;U+001B#gt;y");
        assert_eq!(label("x\ny"), "x#lt;U+000A#gt;y");
        assert_eq!(label("x\u{202e}y"), "x#lt;U+202E#gt;y");
        for hostile in [
            "a\"b",
            "c#d",
            "`e`",
            "f<g",
            "h&i",
            "j\u{1b}k",
            "l\nm",
            "n\u{202e}o",
        ] {
            let written = label(hostile);
            assert!(!written.contains('"'), "{written}");
            assert!(!written.contains('<'), "{written}");
            assert!(!written.contains('&'), "{written}");
            assert!(!written.contains('`'), "{written}");
            assert!(!written.contains('\n'), "{written}");
        }
    }

    /// Two declarations of one prerequisite make one edge, met only when both are, listing each
    /// distinct lowest release.
    #[test]
    fn two_declarations_of_one_prerequisite_merge_into_one_edge() {
        let order = Order {
            runbooks: vec![
                runbook("backup", vec![]),
                runbook(
                    "migrate",
                    vec![
                        prerequisite("backup", met(), Some("1.4.0")),
                        prerequisite("backup", unmet(), Some("1.2.0")),
                    ],
                ),
            ],
            cycles: Vec::new(),
        };
        let drawn = graph(&order);
        assert!(
            drawn.contains("  r1 -->|\"unmet, at least 1.2.0 and 1.4.0\"| r2\n"),
            "{drawn}"
        );
        assert_eq!(drawn.matches("-->").count(), 1, "{drawn}");
    }

    /// The edge cases: an isolated runbook, a missing one, a self-reference, a cycle, and
    /// declarations that cannot be read.
    #[test]
    fn each_edge_case_is_drawn_as_the_design_says() {
        let order = Order {
            runbooks: vec![
                runbook("alone", vec![]),
                runbook("ask", vec![prerequisite("nowhere", State::Missing, None)]),
                runbook(
                    "loop",
                    vec![prerequisite("loop", State::SelfReference, None)],
                ),
                runbook("ping", vec![prerequisite("pong", State::Cycle, None)]),
                runbook("pong", vec![prerequisite("ping", State::Cycle, None)]),
                RunbookOrder {
                    name: String::from("broken"),
                    declarations: Readable::Malformed(String::from("bad")),
                    prerequisites: Vec::new(),
                },
            ],
            cycles: Vec::new(),
        };
        let drawn = graph(&order);
        let expected = "flowchart LR\n  r1[\"alone\"]\n  r2[\"ask\"]\n  \
                        r3[\"broken (declarations cannot be read)\"]\n  r4[\"loop\"]\n  \
                        r5[\"nowhere (no such runbook)\"]\n  r6[\"ping\"]\n  r7[\"pong\"]\n  \
                        r4 -->|\"requires itself\"| r4\n  r5 -->|\"unmet\"| r2\n  \
                        r6 -->|\"cycle\"| r7\n  r7 -->|\"cycle\"| r6\n  \
                        classDef missing stroke-dasharray:5 4\n  \
                        classDef malformed stroke:#a33333,stroke-width:2px\n  \
                        class r5 missing\n  class r3 malformed\n  \
                        linkStyle 1 stroke:#a33333,stroke-width:2px,stroke-dasharray:5 4\n  \
                        linkStyle 0,2,3 stroke:#7a3fa3,stroke-width:3px\n";
        assert_eq!(drawn, expected);
    }

    /// A layered project with no cycle is searched in linear time, and a cycle inside one still is
    /// found exactly as before.
    ///
    /// Fifty-one runbooks in seventeen layers, each requiring all three of the next layer, hold
    /// three to the sixteenth paths. Each layer's names sort after the one requiring it, so the
    /// search, which follows only greater names, can walk every path. The first version did, and
    /// took seconds with no cycle to find (PR #89's review). This one finishes in milliseconds.
    #[test]
    fn a_large_acyclic_project_is_searched_quickly() {
        let names: Vec<String> = (1..=17)
            .flat_map(|layer| ["a", "b", "c"].map(|which| format!("l{layer:02}{which}")))
            .collect();
        let mut requires: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for (index, name) in names.iter().enumerate() {
            let layer = index / 3;
            let next: BTreeSet<&str> = names
                .get((layer + 1) * 3..(layer + 2) * 3)
                .unwrap_or(&[])
                .iter()
                .map(String::as_str)
                .collect();
            requires.insert(name.as_str(), next);
        }
        let started = std::time::Instant::now();
        assert!(cycles(&requires).is_empty());
        assert!(
            started.elapsed() < std::time::Duration::from_millis(500),
            "an acyclic project took {:?}",
            started.elapsed()
        );

        // The same layers with a cycle at the bottom and a self-reference at the top.
        requires.insert("l01a", ["l01b"].into_iter().collect());
        requires.insert("l01b", ["l01a"].into_iter().collect());
        requires.get_mut("l17c").expect("the top").insert("l17c");
        let started = std::time::Instant::now();
        assert_eq!(
            cycles(&requires),
            vec![
                vec![String::from("l01a"), String::from("l01b")],
                vec![String::from("l17c")],
            ]
        );
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
    }

    /// Nine runbooks that each require all the others hold 125,664 elementary cycles. The listing
    /// names one, the shortest through the least runbook, at once.
    #[test]
    fn mutually_requiring_runbooks_list_one_cycle_quickly() {
        let names: Vec<String> = (1..=9).map(|index| format!("r{index}")).collect();
        let mut requires: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for name in &names {
            let others: BTreeSet<&str> = names
                .iter()
                .filter(|other| *other != name)
                .map(String::as_str)
                .collect();
            requires.insert(name.as_str(), others);
        }
        let started = std::time::Instant::now();
        assert_eq!(
            cycles(&requires),
            vec![vec![String::from("r1"), String::from("r2")]]
        );
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
    }

    /// A project with no runbooks is the header alone.
    #[test]
    fn an_empty_order_is_the_header_alone() {
        assert_eq!(graph(&Order::default()), "flowchart LR\n");
    }

    /// Cycles are found once each, from their least runbook, in the order runbooks require one
    /// another, and a self-reference is a cycle of one.
    #[test]
    fn every_cycle_is_found_once_from_its_least_runbook() {
        let mut requires: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        requires.insert("c", ["a"].into_iter().collect());
        requires.insert("a", ["b"].into_iter().collect());
        requires.insert("b", ["c"].into_iter().collect());
        requires.insert("x", ["y"].into_iter().collect());
        requires.insert("y", ["x"].into_iter().collect());
        requires.insert("s", ["s"].into_iter().collect());
        assert_eq!(
            cycles(&requires),
            vec![
                vec![String::from("a"), String::from("b"), String::from("c")],
                vec![String::from("s")],
                vec![String::from("x"), String::from("y")],
            ]
        );
    }
}
