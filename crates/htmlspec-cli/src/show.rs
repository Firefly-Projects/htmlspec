//! `htmlspec show <change> --diff`: each requirement a change modifies, beside the accepted one.
//!
//! **It reads and never writes.** A view that wrote would be a merge under another name, and
//! artifact-kinds reserves the merge for an agent.
//!
//! **It does not compare on its own.** Which accepted requirement a change's requirement is shown
//! beside, whether it differs, where its statements begin to differ, and which scenarios each side
//! holds are the comparison `sync` and archival make, called from here. What this adds is what
//! that comparison leaves out on purpose: which paragraphs differ, how they pair, and which words
//! differ inside a pair.
//!
//! **Nothing here is merge text.** Every text is labelled with the document that holds it, no text
//! is formed from both sides, and the human form is not a patch.
//!
//! The core is pure: [`view`] takes a change's model, its marked units, and the accepted index, and
//! reads nothing. Files are read only in [`command`].

use std::collections::BTreeMap;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use htmlspec_parse::model::{HeadingName, Requirement, Scenario};
use htmlspec_parse::{DocumentModel, MarkedText, MarkedUnit, Run};

use crate::commands::{
    accepted_requirements, compare_content, load_specification, normalized_all,
    resolve_for_comparison, AcceptedRequirements, ContentComparison, ContentDifference,
    ContractEffect, Held, Refusal, ScenarioText,
};
use crate::exit::Outcome;
use crate::output::Streams;

/// One step of an alignment of two sequences, by index into each.
///
/// The left sequence is the accepted side and the right one the change's, so a `Left` step is
/// held only by the accepted specification and a `Right` step is stated only by the change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    /// An item of each that `same` pairs.
    Both(usize, usize),
    /// An item of the left sequence that nothing in the right one is paired with.
    Left(usize),
    /// An item of the right sequence that nothing in the left one is paired with.
    Right(usize),
}

/// The steps that align `left` with `right`, pairing as many items as `same` allows without
/// crossing.
///
/// Every index of each sequence appears in exactly one step, and the steps are in the order of
/// both sequences. The common prefix is taken first, then the common suffix of what remains, then
/// a longest common subsequence of the rest. A tie goes to the left side, so the same inputs
/// always give the same steps.
///
/// Generic over `T` so that it can see what it aligns only through `same`: it cannot pair two
/// items for any reason the caller did not give it.
pub fn align<T>(left: &[T], right: &[T], same: impl Fn(&T, &T) -> bool) -> Vec<Step> {
    let prefix = left
        .iter()
        .zip(right)
        .take_while(|(a, b)| same(a, b))
        .count();
    let (left_rest, right_rest) = (
        left.get(prefix..).unwrap_or_default(),
        right.get(prefix..).unwrap_or_default(),
    );
    let suffix = left_rest
        .iter()
        .rev()
        .zip(right_rest.iter().rev())
        .take_while(|(a, b)| same(a, b))
        .count();
    let left_middle = left_rest
        .get(..left_rest.len() - suffix)
        .unwrap_or_default();
    let right_middle = right_rest
        .get(..right_rest.len() - suffix)
        .unwrap_or_default();

    let mut steps: Vec<Step> = (0..prefix).map(|index| Step::Both(index, index)).collect();
    steps.extend(
        longest_common_subsequence(left_middle, right_middle, &same)
            .into_iter()
            .map(|step| match step {
                Step::Both(i, j) => Step::Both(prefix + i, prefix + j),
                Step::Left(i) => Step::Left(prefix + i),
                Step::Right(j) => Step::Right(prefix + j),
            }),
    );
    let (left_end, right_end) = (prefix + left_middle.len(), prefix + right_middle.len());
    steps.extend((0..suffix).map(|offset| Step::Both(left_end + offset, right_end + offset)));
    steps
}

/// The steps of a longest common subsequence of `left` and `right` under `same`, a tie going to
/// the left side.
fn longest_common_subsequence<T>(
    left: &[T],
    right: &[T],
    same: impl Fn(&T, &T) -> bool,
) -> Vec<Step> {
    let width = right.len() + 1;
    // `table[i * width + j]` is the length of a longest common subsequence of `left[i..]` and
    // `right[j..]`.
    let mut table = vec![0usize; (left.len() + 1) * width];
    let at = |table: &[usize], i: usize, j: usize| table.get(i * width + j).copied().unwrap_or(0);
    for i in (0..left.len()).rev() {
        for j in (0..right.len()).rev() {
            let paired = match (left.get(i), right.get(j)) {
                (Some(a), Some(b)) if same(a, b) => at(&table, i + 1, j + 1) + 1,
                _ => 0,
            };
            let length = paired.max(at(&table, i + 1, j)).max(at(&table, i, j + 1));
            if let Some(cell) = table.get_mut(i * width + j) {
                *cell = length;
            }
        }
    }

    let mut steps = Vec::with_capacity(left.len() + right.len());
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        let pairs = match (left.get(i), right.get(j)) {
            (Some(a), Some(b)) => same(a, b) && at(&table, i, j) == at(&table, i + 1, j + 1) + 1,
            _ => false,
        };
        if pairs {
            steps.push(Step::Both(i, j));
            i += 1;
            j += 1;
        } else if at(&table, i + 1, j) >= at(&table, i, j + 1) {
            steps.push(Step::Left(i));
            i += 1;
        } else {
            steps.push(Step::Right(j));
            j += 1;
        }
    }
    steps.extend((i..left.len()).map(Step::Left));
    steps.extend((j..right.len()).map(Step::Right));
    steps
}

/// Whose marks a statement list or a scenario's entries are shown with.
///
/// One closed type, matched wherever the view is rendered, so a fourth case is a compile error and
/// never a silent default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Marking {
    /// The author's `ins` and `del`, which account for the whole difference.
    Author,
    /// The view's own comparison, where the author marked nothing.
    Comparison,
    /// The view's own comparison, because the author's marks leave part of the difference
    /// unmarked.
    Unreconciled,
}

/// A position in a sequence of paragraphs or entries, counted from one.
pub type Position = NonZeroUsize;

/// What the view shows for one change.
#[derive(Clone, Debug)]
pub struct View {
    /// The change's name.
    pub change: String,
    /// Every requirement the change states, in the order it states them.
    pub requirements: Vec<RequirementShown>,
}

/// One requirement the change states, and what the view shows for it.
#[derive(Clone, Debug)]
pub struct RequirementShown {
    pub id: String,
    /// The change state the change declares on it, as the document declares it.
    pub declared: String,
    /// Its heading name, as the change states it.
    pub heading: Option<HeadingName>,
    pub shown: Shown,
}

/// What the view shows of one requirement.
#[derive(Clone, Debug)]
pub enum Shown {
    /// Listed and not compared: the change withdraws it, or carries it and every scenario forward.
    NotCompared,
    /// Compared, and no accepted specification holds its identifier.
    HeldByNone,
    /// Compared, and the accepted specification named holds its content as the change states it.
    HeldAsStated(Counterpart),
    /// Compared, and the accepted requirement named holds other content.
    Differs(Counterpart, Comparison),
}

/// The accepted requirement a change's requirement is shown beside.
#[derive(Clone, Debug)]
pub struct Counterpart {
    /// The accepted specification that holds it, relative to the project's root.
    pub path: PathBuf,
    /// Its heading name, as the accepted specification holds it.
    pub heading: Option<HeadingName>,
}

/// Where one requirement's content and its counterpart's differ.
#[derive(Clone, Debug)]
pub struct Comparison {
    pub statements: Texts,
    /// The change's scenarios in the order it states them, then those only the accepted
    /// requirement holds, in its order.
    pub scenarios: Vec<ScenarioShown>,
}

/// A sequence of paragraphs or entries, aligned with its counterpart.
#[derive(Clone, Debug)]
pub struct Texts {
    pub marking: Marking,
    pub paragraphs: Vec<Paragraph>,
}

/// One paragraph or entry, as the alignment places it.
#[derive(Clone, Debug)]
pub enum Paragraph {
    /// Held by both, at these positions, and not reprinted.
    Both {
        position: Position,
        accepted: Position,
    },
    /// Paired with its counterpart, each text whole, with the words only one side holds marked on
    /// that side.
    Pair {
        position: Position,
        accepted: Position,
        accepted_holds: Vec<Piece>,
        change_states: Vec<Piece>,
    },
    /// Stated only by the change, whole.
    Change { position: Position, text: String },
    /// Held only by the accepted specification, whole.
    Accepted { accepted: Position, text: String },
}

/// One stretch of a paired text.
///
/// The pieces of one side, joined, are that side's text exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Piece {
    /// Words both sides hold.
    Shared(String),
    /// Words only this side holds.
    OnlyHere(String),
}

/// One scenario, as the view pairs it.
#[derive(Clone, Debug)]
pub enum ScenarioShown {
    /// The accepted requirement holds its text.
    Both {
        id: Option<String>,
        declared: String,
    },
    /// Paired with the accepted scenario carrying its identifier.
    Pair(ScenarioPair),
    /// Stated only by the change, whole.
    Change {
        declared: String,
        scenario: WholeScenario,
    },
    /// Withdrawn by the change, and whether the accepted requirement still holds it.
    Withdrawn {
        id: Option<String>,
        declared: String,
        heading: Option<HeadingName>,
        still_held: bool,
    },
    /// Held only by the accepted specification, whole.
    Accepted(WholeScenario),
}

/// A scenario the change states beside the accepted scenario carrying its identifier.
#[derive(Clone, Debug)]
pub struct ScenarioPair {
    pub id: String,
    pub declared: String,
    pub heading: Option<HeadingName>,
    pub accepted_heading: Option<HeadingName>,
    /// Whose marks its entries are shown with. One for the three lists, because the author's
    /// marks are checked against the scenario as a whole.
    pub marking: Marking,
    pub given: Vec<Paragraph>,
    pub when: Vec<Paragraph>,
    pub then: Vec<Paragraph>,
}

/// A scenario's text, shown whole.
#[derive(Clone, Debug)]
pub struct WholeScenario {
    pub id: Option<String>,
    pub heading: Option<HeadingName>,
    pub given: Vec<String>,
    pub when: Vec<String>,
    pub then: Vec<String>,
}

/// The view of `change`, whose specification projects as `specification` and carries the marks in
/// `marked`, beside the accepted requirements in `accepted`.
///
/// Pure: it reads no file, so the tests build every case from projections in memory.
///
/// Which requirements are compared, which accepted requirement each is compared with, and whether
/// it differs are the comparison's answers, through [`ContractEffect::of_requirement`] and
/// [`compare_content`]. The view adds only how the paragraphs and scenarios of a differing
/// requirement line up.
pub fn view(
    change: &str,
    specification: &DocumentModel,
    marked: &BTreeMap<String, MarkedUnit>,
    accepted: &AcceptedRequirements,
) -> View {
    let requirements = specification
        .requirements
        .iter()
        .filter_map(|requirement| {
            let id = requirement.id.clone()?;
            let shown = match ContractEffect::of_requirement(requirement) {
                ContractEffect::Withdraws | ContractEffect::CarriesForward => Shown::NotCompared,
                ContractEffect::Establishes => match accepted.get(&id) {
                    None => Shown::HeldByNone,
                    Some(holders) => match compare_content(requirement, holders) {
                        ContentComparison::Held(held) => Shown::HeldAsStated(counterpart(held)),
                        ContentComparison::Differs(held, difference) => Shown::Differs(
                            counterpart(held),
                            comparison(requirement, &held.requirement, &difference, marked),
                        ),
                    },
                },
            };
            Some(RequirementShown {
                declared: requirement.change.clone(),
                heading: requirement.heading_name(),
                shown,
                id,
            })
        })
        .collect();
    View {
        change: change.to_owned(),
        requirements,
    }
}

fn counterpart(held: &Held) -> Counterpart {
    Counterpart {
        path: held.path.clone(),
        heading: held.requirement.heading_name(),
    }
}

/// The paragraphs and scenarios of `stated`, a requirement the comparison found differing from
/// `accepted` as `difference` says.
fn comparison(
    stated: &Requirement,
    accepted: &Requirement,
    difference: &ContentDifference,
    marked: &BTreeMap<String, MarkedUnit>,
) -> Comparison {
    let statements = stated
        .id
        .as_ref()
        .and_then(|id| marked.get(id))
        .map(|unit| unit.statements.as_slice());
    let (marking, [paragraphs]) = texts([(&stated.statements, &accepted.statements, statements)]);
    Comparison {
        statements: Texts {
            marking,
            paragraphs,
        },
        scenarios: scenarios(stated, accepted, difference, marked),
    }
}

/// One list the change states, the accepted list it is aligned with, and the author's marks on
/// the change's list, where it carries any.
type Lists<'a> = (&'a [String], &'a [String], Option<&'a [MarkedText]>);

/// Each of `lists` aligned, with one marking for all of them.
///
/// One marking rather than one per list, because the author's marks are checked against the unit
/// as a whole: a requirement's statements are one list, and a scenario's given, when, and then
/// entries are three lists checked together. The marks are the author's only where every list,
/// read with each deletion kept and each insertion left out, equals the accepted one.
fn texts<const N: usize>(lists: [Lists<'_>; N]) -> (Marking, [Vec<Paragraph>; N]) {
    let carries_marks = lists
        .iter()
        .any(|(_, _, marks)| marks.is_some_and(|marks| marks.iter().any(MarkedText::is_marked)));
    let reconciled = lists.iter().all(|(_, accepted, marks)| {
        let before: Vec<String> = marks
            .unwrap_or_default()
            .iter()
            .map(MarkedText::before)
            .filter(|text| !text.is_empty())
            .collect();
        before == normalized_all(accepted)
    });
    let marking = match (carries_marks, reconciled) {
        (false, _) => Marking::Comparison,
        (true, true) => Marking::Author,
        (true, false) => Marking::Unreconciled,
    };
    (
        marking,
        lists.map(|(stated, accepted, marks)| match marking {
            Marking::Author => authored(marks.unwrap_or_default()),
            Marking::Comparison | Marking::Unreconciled => {
                compared(&normalized_all(accepted), &normalized_all(stated))
            }
        }),
    )
}

/// The paragraphs of a list the author's marks account for, as the author marked them.
fn authored(marks: &[MarkedText]) -> Vec<Paragraph> {
    let (mut position, mut accepted) = (0, 0);
    let mut paragraphs = Vec::new();
    for text in marks {
        let (current, before) = (text.current(), text.before());
        let paragraph = match (current.is_empty(), before.is_empty()) {
            (true, true) => continue,
            (false, true) => {
                position += 1;
                Paragraph::Change {
                    position: counted(position),
                    text: current,
                }
            }
            (true, false) => {
                accepted += 1;
                Paragraph::Accepted {
                    accepted: counted(accepted),
                    text: before,
                }
            }
            (false, false) => {
                position += 1;
                accepted += 1;
                if current == before {
                    Paragraph::Both {
                        position: counted(position),
                        accepted: counted(accepted),
                    }
                } else {
                    Paragraph::Pair {
                        position: counted(position),
                        accepted: counted(accepted),
                        accepted_holds: pieces(text.runs().iter().filter_map(|run| match run {
                            Run::Kept(words) => Some((false, words.as_str())),
                            Run::Deleted(words) => Some((true, words.as_str())),
                            Run::Inserted(_) => None,
                        })),
                        change_states: pieces(text.runs().iter().filter_map(|run| match run {
                            Run::Kept(words) => Some((false, words.as_str())),
                            Run::Inserted(words) => Some((true, words.as_str())),
                            Run::Deleted(_) => None,
                        })),
                    }
                }
            }
        };
        paragraphs.push(paragraph);
    }
    paragraphs
}

/// The paragraphs of a list aligned by the view's own comparison: first by equality, then, within
/// each run between two paragraphs held by both, by likeness, then each pair word by word.
fn compared(accepted: &[String], stated: &[String]) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let (mut only_accepted, mut only_stated) = (Vec::new(), Vec::new());
    for step in align(accepted, stated, |a, b| a == b) {
        match step {
            Step::Both(i, j) => {
                pair_run(
                    accepted,
                    stated,
                    &only_accepted,
                    &only_stated,
                    &mut paragraphs,
                );
                only_accepted.clear();
                only_stated.clear();
                paragraphs.push(Paragraph::Both {
                    position: counted(j + 1),
                    accepted: counted(i + 1),
                });
            }
            Step::Left(i) => only_accepted.push(i),
            Step::Right(j) => only_stated.push(j),
        }
    }
    pair_run(
        accepted,
        stated,
        &only_accepted,
        &only_stated,
        &mut paragraphs,
    );
    paragraphs
}

/// One run of paragraphs between two held by both: those alike paired, the rest whole.
fn pair_run(
    accepted: &[String],
    stated: &[String],
    only_accepted: &[usize],
    only_stated: &[usize],
    into: &mut Vec<Paragraph>,
) {
    let text = |list: &[String], index: usize| list.get(index).cloned().unwrap_or_default();
    let left: Vec<String> = only_accepted.iter().map(|&i| text(accepted, i)).collect();
    let right: Vec<String> = only_stated.iter().map(|&j| text(stated, j)).collect();
    let index = |indices: &[usize], at: usize| indices.get(at).copied().unwrap_or(0);
    for step in align(&left, &right, |a, b| alike(a, b)) {
        into.push(match step {
            Step::Both(a, b) => {
                let (accepted_holds, change_states) = words_marked(
                    left.get(a).map_or("", String::as_str),
                    right.get(b).map_or("", String::as_str),
                );
                Paragraph::Pair {
                    position: counted(index(only_stated, b) + 1),
                    accepted: counted(index(only_accepted, a) + 1),
                    accepted_holds,
                    change_states,
                }
            }
            Step::Left(a) => Paragraph::Accepted {
                accepted: counted(index(only_accepted, a) + 1),
                text: left.get(a).cloned().unwrap_or_default(),
            },
            Step::Right(b) => Paragraph::Change {
                position: counted(index(only_stated, b) + 1),
                text: right.get(b).cloned().unwrap_or_default(),
            },
        });
    }
}

/// Whether two paragraphs are alike: their words align with at least half of each one's words in
/// common. A pair that shares fewer is marked more than it is kept, and reads better as two whole
/// paragraphs.
fn alike(accepted: &str, stated: &str) -> bool {
    let (left, right): (Vec<&str>, Vec<&str>) = (
        accepted.split_whitespace().collect(),
        stated.split_whitespace().collect(),
    );
    let common = align(&left, &right, |a, b| a == b)
        .iter()
        .filter(|step| matches!(step, Step::Both(..)))
        .count();
    2 * common >= left.len() && 2 * common >= right.len()
}

/// The two sides of a pair, word by word: the words only one side holds marked on that side.
fn words_marked(accepted: &str, stated: &str) -> (Vec<Piece>, Vec<Piece>) {
    let (left, right): (Vec<&str>, Vec<&str>) = (
        accepted.split_whitespace().collect(),
        stated.split_whitespace().collect(),
    );
    let steps = align(&left, &right, |a, b| a == b);
    let side = |words: &[&str], only: &dyn Fn(&Step) -> Option<(bool, usize)>| -> Vec<Piece> {
        let tokens: Vec<(bool, String)> = steps
            .iter()
            .filter_map(only)
            .filter_map(|(only_here, index)| {
                words.get(index).map(|word| (only_here, format!("{word} ")))
            })
            .collect();
        pieces(
            tokens
                .iter()
                .map(|(only_here, word)| (*only_here, word.as_str())),
        )
    };
    let accepted_holds = side(&left, &|step| match *step {
        Step::Both(i, _) => Some((false, i)),
        Step::Left(i) => Some((true, i)),
        Step::Right(_) => None,
    });
    let change_states = side(&right, &|step| match *step {
        Step::Both(_, j) => Some((false, j)),
        Step::Right(j) => Some((true, j)),
        Step::Left(_) => None,
    });
    (accepted_holds, change_states)
}

/// The pieces of one side, from stretches of its text each marked as held only here or not.
///
/// The text is normalized across the stretches as the projection normalizes a field, so the
/// pieces joined are the side's text exactly. A space between two words goes with the word before
/// it, and stretches of one kind are joined. Words a mark splits are kept apart, with no space
/// between them, as the author marked them.
fn pieces<'a>(stretches: impl Iterator<Item = (bool, &'a str)>) -> Vec<Piece> {
    // Each maximal run of non-space characters of one kind, and whether space came before it.
    let mut tokens: Vec<(bool, String, bool)> = Vec::new();
    let mut space = false;
    for (only_here, text) in stretches {
        for character in text.chars() {
            if character.is_whitespace() {
                space = true;
                continue;
            }
            match tokens.last_mut() {
                Some((kind, word, _)) if !space && *kind == only_here => word.push(character),
                _ => tokens.push((only_here, character.to_string(), space)),
            }
            space = false;
        }
    }
    let mut pieces: Vec<Piece> = Vec::new();
    let count = tokens.len();
    for (index, (only_here, mut word, _)) in tokens.clone().into_iter().enumerate() {
        if index + 1 < count && tokens.get(index + 1).is_some_and(|next| next.2) {
            word.push(' ');
        }
        match pieces.last_mut() {
            Some(Piece::OnlyHere(text)) if only_here => text.push_str(&word),
            Some(Piece::Shared(text)) if !only_here => text.push_str(&word),
            _ => pieces.push(if only_here {
                Piece::OnlyHere(word)
            } else {
                Piece::Shared(word)
            }),
        }
    }
    pieces
}

/// The scenarios of a differing requirement, as design decision 6's table places them.
fn scenarios(
    stated: &Requirement,
    accepted: &Requirement,
    difference: &ContentDifference,
    marked: &BTreeMap<String, MarkedUnit>,
) -> Vec<ScenarioShown> {
    let mut unpaired: Vec<usize> = difference.scenarios_not_kept.clone();
    // The accepted scenario among those not kept that is `stated`'s counterpart, taken so nothing
    // shows it twice. By identifier, which is what makes a stated scenario the same scenario as an
    // accepted one. Where `by_text` and no identifier finds one, by the comparison's own text: a
    // withdrawn scenario with no identifier, or with another one, is still held where the accepted
    // requirement holds its text, and saying otherwise would list that scenario twice.
    let mut take = |stated: &Scenario, by_text: bool| -> Option<&Scenario> {
        let carries_its_id = |position: &usize| {
            stated.id.as_deref().is_some_and(|id| {
                accepted
                    .scenarios
                    .get(*position)
                    .is_some_and(|scenario| scenario.id.as_deref() == Some(id))
            })
        };
        let holds_its_text = |position: &usize| {
            accepted
                .scenarios
                .get(*position)
                .is_some_and(|scenario| ScenarioText::of(scenario) == ScenarioText::of(stated))
        };
        let at = unpaired.iter().position(carries_its_id).or_else(|| {
            by_text
                .then(|| unpaired.iter().position(holds_its_text))
                .flatten()
        })?;
        let position = unpaired.remove(at);
        accepted.scenarios.get(position)
    };

    let mut shown = Vec::new();
    for (position, scenario) in stated.scenarios.iter().enumerate() {
        let declared = scenario.change.clone();
        if ContractEffect::of(&scenario.change) == ContractEffect::Withdraws {
            shown.push(ScenarioShown::Withdrawn {
                id: scenario.id.clone(),
                heading: heading_of(scenario),
                still_held: take(scenario, true).is_some(),
                declared,
            });
        } else if !difference.scenarios_not_held.contains(&position) {
            shown.push(ScenarioShown::Both {
                id: scenario.id.clone(),
                declared,
            });
        } else if let Some(counterpart) = take(scenario, false) {
            shown.push(ScenarioShown::Pair(scenario_pair(
                scenario,
                counterpart,
                marked,
            )));
        } else {
            shown.push(ScenarioShown::Change {
                declared,
                scenario: whole(scenario),
            });
        }
    }
    shown.extend(
        unpaired
            .iter()
            .filter_map(|&position| accepted.scenarios.get(position))
            .map(|scenario| ScenarioShown::Accepted(whole(scenario))),
    );
    shown
}

fn scenario_pair(
    stated: &Scenario,
    accepted: &Scenario,
    marked: &BTreeMap<String, MarkedUnit>,
) -> ScenarioPair {
    let marks = stated.id.as_ref().and_then(|id| marked.get(id));
    let (marking, [given, when, then]) = texts([
        (
            &stated.given,
            &accepted.given,
            marks.map(|unit| unit.given.as_slice()),
        ),
        (
            &stated.when,
            &accepted.when,
            marks.map(|unit| unit.when.as_slice()),
        ),
        (
            &stated.then,
            &accepted.then,
            marks.map(|unit| unit.then.as_slice()),
        ),
    ]);
    ScenarioPair {
        id: stated.id.clone().unwrap_or_default(),
        declared: stated.change.clone(),
        heading: heading_of(stated),
        accepted_heading: heading_of(accepted),
        marking,
        given,
        when,
        then,
    }
}

fn heading_of(scenario: &Scenario) -> Option<HeadingName> {
    scenario
        .title
        .as_deref()
        .map(|title| HeadingName::of(title, scenario.id.as_deref()))
}

fn whole(scenario: &Scenario) -> WholeScenario {
    WholeScenario {
        id: scenario.id.clone(),
        heading: heading_of(scenario),
        given: normalized_all(&scenario.given),
        when: normalized_all(&scenario.when),
        then: normalized_all(&scenario.then),
    }
}

/// The position `n`, counted from one. `n` is at least one wherever it is called, and zero would
/// read as the first.
fn counted(n: usize) -> Position {
    Position::MIN.saturating_add(n.saturating_sub(1))
}

/// `htmlspec show <change> --diff`.
///
/// Refuses as `sync` does, in its order and for its reasons, through the same two functions, and
/// refuses an archived change between them: before reading anything of it, because what it would
/// show is not what the change modified.
pub fn command<O: Write, E: Write>(
    change: &str,
    store: Option<&str>,
    streams: &mut Streams<O, E>,
) -> Outcome {
    let (project, directory) = match resolve_for_comparison(change, store, Refusal::View, streams) {
        Ok(resolved) => resolved,
        Err(outcome) => return outcome,
    };
    if project.is_archived_change(&directory) {
        let _ = streams.failure(&format!(
            "{} is an archived change. The accepted specifications hold its edit together with \
             every later change's, so a comparison with them shows a later change's edit, \
             reversed, where there is one, and nothing where there is none. `show --diff` \
             compares a change before its merge. What an archived change modified is in its \
             specification's marks, where it has them, and in the repository's history",
            directory.display()
        ));
        return Outcome::OperationalFailure;
    }
    let projection = match load_specification(&directory, streams) {
        Ok(projection) => projection,
        Err(outcome) => return outcome,
    };

    let marked = projection
        .document
        .as_ref()
        .map(htmlspec_parse::marked_units)
        .unwrap_or_default();
    let name = directory.file_name().map_or_else(
        || change.to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let shown = view(
        &name,
        &projection.model,
        &marked,
        &accepted_requirements(&project),
    );

    if streams.form().is_structured() {
        if let Ok(json) = serde_json::to_string_pretty(&payload(&shown)) {
            let _ = streams.payload(&json);
        }
    } else {
        let _ = streams.report(&human(&shown));
    }
    Outcome::Success
}

/// What a missing `--diff` is told: the one view `show` has, and how to ask for it.
pub const ONE_VIEW: &str = "show has one view today, `htmlspec show <change> --diff`, which \
                            shows each requirement the change adds or modifies beside the \
                            accepted one. Pass --diff";

// ---------------------------------------------------------------------------
// The structured form
// ---------------------------------------------------------------------------

/// The view as a payload, with the same facts as the human form.
///
/// Every text is a list of runs labelled by the side that holds it, and no key names a result, a
/// merge, or anything to apply.
pub fn payload(view: &View) -> serde_json::Value {
    serde_json::json!({
        "change": view.change,
        "requirements": view.requirements.iter().map(requirement_payload).collect::<Vec<_>>(),
    })
}

fn requirement_payload(requirement: &RequirementShown) -> serde_json::Value {
    serde_json::json!({
        "id": requirement.id,
        "declared": requirement.declared,
        "heading": requirement.heading.as_ref().map(HeadingName::as_str),
        "comparison": match &requirement.shown {
            Shown::NotCompared => serde_json::json!({ "shown": "not-compared" }),
            Shown::HeldByNone => serde_json::json!({ "shown": "held-by-none" }),
            Shown::HeldAsStated(counterpart) => serde_json::json!({
                "shown": "held-as-stated",
                "acceptedIn": counterpart.path.to_string_lossy(),
                "acceptedHeading": counterpart.heading.as_ref().map(HeadingName::as_str),
            }),
            Shown::Differs(counterpart, comparison) => serde_json::json!({
                "shown": "differs",
                "acceptedIn": counterpart.path.to_string_lossy(),
                "acceptedHeading": counterpart.heading.as_ref().map(HeadingName::as_str),
                "statements": {
                    "marks": marking_name(comparison.statements.marking),
                    "paragraphs": comparison
                        .statements
                        .paragraphs
                        .iter()
                        .map(paragraph_payload)
                        .collect::<Vec<_>>(),
                },
                "scenarios": comparison.scenarios.iter().map(scenario_payload).collect::<Vec<_>>(),
            }),
        },
    })
}

fn marking_name(marking: Marking) -> &'static str {
    match marking {
        Marking::Author => "author",
        Marking::Comparison => "comparison",
        Marking::Unreconciled => "unreconciled",
    }
}

fn paragraph_payload(paragraph: &Paragraph) -> serde_json::Value {
    match paragraph {
        Paragraph::Both { position, accepted } => serde_json::json!({
            "kind": "both",
            "position": position.get(),
            "acceptedPosition": accepted.get(),
        }),
        Paragraph::Pair {
            position,
            accepted,
            accepted_holds,
            change_states,
        } => serde_json::json!({
            "kind": "pair",
            "position": position.get(),
            "acceptedPosition": accepted.get(),
            "acceptedHolds": runs_payload(accepted_holds),
            "changeStates": runs_payload(change_states),
        }),
        Paragraph::Change { position, text } => serde_json::json!({
            "kind": "change",
            "position": position.get(),
            "changeStates": [{ "text": text }],
        }),
        Paragraph::Accepted { accepted, text } => serde_json::json!({
            "kind": "accepted",
            "acceptedPosition": accepted.get(),
            "acceptedHolds": [{ "text": text }],
        }),
    }
}

fn runs_payload(pieces: &[Piece]) -> Vec<serde_json::Value> {
    pieces
        .iter()
        .map(|piece| match piece {
            Piece::Shared(text) => serde_json::json!({ "text": text }),
            Piece::OnlyHere(text) => serde_json::json!({ "text": text, "onlyThisSide": true }),
        })
        .collect()
}

fn scenario_payload(scenario: &ScenarioShown) -> serde_json::Value {
    match scenario {
        ScenarioShown::Both { id, declared } => serde_json::json!({
            "kind": "both",
            "id": id,
            "declared": declared,
        }),
        ScenarioShown::Pair(pair) => {
            let entries =
                |list: &[Paragraph]| list.iter().map(paragraph_payload).collect::<Vec<_>>();
            serde_json::json!({
                "kind": "pair",
                "id": pair.id,
                "declared": pair.declared,
                "heading": pair.heading.as_ref().map(HeadingName::as_str),
                "acceptedHeading": pair.accepted_heading.as_ref().map(HeadingName::as_str),
                "marks": marking_name(pair.marking),
                "given": entries(&pair.given),
                "when": entries(&pair.when),
                "then": entries(&pair.then),
            })
        }
        ScenarioShown::Change { declared, scenario } => serde_json::json!({
            "kind": "change",
            "id": scenario.id,
            "declared": declared,
            "changeStates": whole_payload(scenario),
        }),
        ScenarioShown::Withdrawn {
            id,
            declared,
            heading,
            still_held,
        } => serde_json::json!({
            "kind": "withdrawn",
            "id": id,
            "declared": declared,
            "heading": heading.as_ref().map(HeadingName::as_str),
            "stillHeld": still_held,
        }),
        ScenarioShown::Accepted(scenario) => serde_json::json!({
            "kind": "accepted",
            "id": scenario.id,
            "acceptedHolds": whole_payload(scenario),
        }),
    }
}

fn whole_payload(scenario: &WholeScenario) -> serde_json::Value {
    serde_json::json!({
        "heading": scenario.heading.as_ref().map(HeadingName::as_str),
        "given": scenario.given,
        "when": scenario.when,
        "then": scenario.then,
    })
}

// ---------------------------------------------------------------------------
// The human form
// ---------------------------------------------------------------------------

const ACCEPTED_HOLDS: &str = "the accepted specification holds:";
const CHANGE_STATES: &str = "the change states:";

/// The view for a reader: what it is, then each requirement it compares, then the list.
///
/// Each text is one line, quoted whole and labelled with the document that holds it. Words only
/// one side holds are marked `{+…+}` or `[-…-]`, `git diff --word-diff=plain`'s notation, and only
/// on the side that holds them. There is no hunk header, no file header, and no line marked whole
/// as added or removed, so neither `patch` nor `git apply` can take it.
pub fn human(view: &View) -> String {
    let compared: Vec<&RequirementShown> = view
        .requirements
        .iter()
        .filter(|requirement| !matches!(requirement.shown, Shown::NotCompared))
        .collect();
    let mut out = format!(
        "{} states {} requirement(s), and compares {} of them.\n\
         Each text is quoted, as projected, from the document named beside it. {{+ +}} marks \
         words only the change states, and [- -] words only the accepted specification holds.\n",
        view.change,
        view.requirements.len(),
        compared.len()
    );
    for requirement in compared {
        out.push('\n');
        out.push_str(&format!("{}, {}\n", requirement.id, requirement.declared));
        let stated_heading = requirement.heading.as_ref().map_or("", HeadingName::as_str);
        match &requirement.shown {
            Shown::NotCompared => {}
            Shown::HeldByNone => {
                out.push_str(&format!("  {stated_heading}\n"));
                out.push_str("  held by no accepted specification, so nothing is compared\n");
            }
            Shown::HeldAsStated(counterpart) => {
                headings(&mut out, "  ", stated_heading, counterpart.heading.as_ref());
                out.push_str(&format!(
                    "  held as stated in {}\n",
                    counterpart.path.display()
                ));
            }
            Shown::Differs(counterpart, comparison) => {
                headings(&mut out, "  ", stated_heading, counterpart.heading.as_ref());
                out.push_str(&format!("  compared with {}\n", counterpart.path.display()));
                statements(&mut out, &comparison.statements);
                scenarios_human(&mut out, &comparison.scenarios);
            }
        }
    }
    let listed: Vec<&RequirementShown> = view
        .requirements
        .iter()
        .filter(|requirement| matches!(requirement.shown, Shown::NotCompared))
        .collect();
    if !listed.is_empty() {
        out.push_str("\nlisted, not compared:\n");
        for requirement in listed {
            out.push_str(&format!(
                "  {:<9} {}\n",
                requirement.declared, requirement.id
            ));
        }
    }
    out
}

/// The heading, or both headings, each labelled, where they differ.
fn headings(out: &mut String, indent: &str, stated: &str, accepted: Option<&HeadingName>) {
    let accepted = accepted.map_or("", HeadingName::as_str);
    if accepted == stated {
        out.push_str(&format!("{indent}{stated}\n"));
    } else {
        out.push_str(&format!(
            "{indent}the change states the heading: {stated}\n"
        ));
        out.push_str(&format!(
            "{indent}the accepted specification holds the heading: {accepted}\n"
        ));
    }
}

fn marking_said(marking: Marking) -> &'static str {
    match marking {
        Marking::Author => "words marked as the author marked them",
        Marking::Comparison => "words marked by comparison",
        Marking::Unreconciled => {
            "words marked by comparison, because the author's marks leave part of the difference \
             unmarked"
        }
    }
}

fn statements(out: &mut String, texts: &Texts) {
    let both = texts
        .paragraphs
        .iter()
        .filter(|paragraph| matches!(paragraph, Paragraph::Both { .. }))
        .count();
    let stated = texts
        .paragraphs
        .iter()
        .filter(|paragraph| !matches!(paragraph, Paragraph::Accepted { .. }))
        .count();
    let accepted = texts
        .paragraphs
        .iter()
        .filter(|paragraph| !matches!(paragraph, Paragraph::Change { .. }))
        .count();
    let also = if accepted == stated {
        String::new()
    } else {
        format!(", and the accepted specification holds {accepted}")
    };
    out.push_str(&format!(
        "  statements: {both} of {stated} paragraphs held by both{also}, {}\n",
        marking_said(texts.marking)
    ));
    paragraphs(out, "    ", "paragraph", &texts.paragraphs);
}

/// Each paragraph or entry that is not held by both, with its texts.
fn paragraphs(out: &mut String, indent: &str, noun: &str, list: &[Paragraph]) {
    for paragraph in list {
        match paragraph {
            Paragraph::Both { .. } => {}
            Paragraph::Pair {
                position,
                accepted,
                accepted_holds,
                change_states,
            } => {
                out.push_str(&format!(
                    "{indent}{noun} {position}, and {noun} {accepted} in the accepted specification\n"
                ));
                quoted(
                    out,
                    indent,
                    ACCEPTED_HOLDS,
                    &marked_text(accepted_holds, "[-", "-]"),
                );
                quoted(
                    out,
                    indent,
                    CHANGE_STATES,
                    &marked_text(change_states, "{+", "+}"),
                );
            }
            Paragraph::Change { position, text } => {
                out.push_str(&format!(
                    "{indent}{noun} {position}, stated only by the change\n"
                ));
                quoted(out, indent, CHANGE_STATES, text);
            }
            Paragraph::Accepted { accepted, text } => {
                out.push_str(&format!(
                    "{indent}{noun} {accepted} in the accepted specification, held only by it\n"
                ));
                quoted(out, indent, ACCEPTED_HOLDS, text);
            }
        }
    }
}

fn quoted(out: &mut String, indent: &str, label: &str, text: &str) {
    out.push_str(&format!("{indent}  {label}\n{indent}    {text}\n"));
}

/// A side's pieces as one line, the words only this side holds between `open` and `close`, and
/// the spaces around them outside the marks.
fn marked_text(pieces: &[Piece], open: &str, close: &str) -> String {
    pieces
        .iter()
        .map(|piece| match piece {
            Piece::Shared(text) => text.clone(),
            Piece::OnlyHere(text) => {
                let trimmed = text.trim();
                let lead = &text[..text.len() - text.trim_start().len()];
                let trail = &text[text.trim_end().len()..];
                format!("{lead}{open}{trimmed}{close}{trail}")
            }
        })
        .collect()
}

fn scenarios_human(out: &mut String, scenarios: &[ScenarioShown]) {
    let both = scenarios
        .iter()
        .filter(|scenario| matches!(scenario, ScenarioShown::Both { .. }))
        .count();
    let stated = scenarios
        .iter()
        .filter(|scenario| !matches!(scenario, ScenarioShown::Accepted(_)))
        .count();
    out.push_str(&format!("  scenarios: {both} of {stated} held by both\n"));
    for scenario in scenarios {
        match scenario {
            ScenarioShown::Both { .. } => {}
            ScenarioShown::Pair(pair) => {
                out.push_str(&format!(
                    "    {}, {}, beside the accepted scenario carrying its identifier, {}\n",
                    pair.id,
                    pair.declared,
                    marking_said(pair.marking)
                ));
                headings(
                    out,
                    "      ",
                    pair.heading.as_ref().map_or("", HeadingName::as_str),
                    pair.accepted_heading.as_ref(),
                );
                for (label, list) in [
                    ("given", &pair.given),
                    ("when", &pair.when),
                    ("then", &pair.then),
                ] {
                    paragraphs(out, "      ", label, list);
                }
            }
            ScenarioShown::Change { declared, scenario } => {
                out.push_str(&format!(
                    "    {}, {declared}, stated only by the change\n",
                    scenario
                        .id
                        .as_deref()
                        .unwrap_or("a scenario with no identifier")
                ));
                whole_human(out, CHANGE_STATES, scenario);
            }
            ScenarioShown::Withdrawn {
                id,
                declared,
                still_held,
                ..
            } => {
                out.push_str(&format!(
                    "    {}, {declared}, withdrawn by the change, and {}\n",
                    id.as_deref().unwrap_or("a scenario with no identifier"),
                    if *still_held {
                        "still held by the accepted specification"
                    } else {
                        "no longer held by the accepted specification"
                    }
                ));
            }
            ScenarioShown::Accepted(scenario) => {
                out.push_str(&format!(
                    "    {}, held only by the accepted specification\n",
                    scenario
                        .id
                        .as_deref()
                        .unwrap_or("a scenario with no identifier")
                ));
                whole_human(out, ACCEPTED_HOLDS, scenario);
            }
        }
    }
}

fn whole_human(out: &mut String, label: &str, scenario: &WholeScenario) {
    out.push_str(&format!("      {label}\n"));
    if let Some(heading) = &scenario.heading {
        out.push_str(&format!("        {heading}\n"));
    }
    for (name, entries) in [
        ("Given", &scenario.given),
        ("When", &scenario.when),
        ("Then", &scenario.then),
    ] {
        for entry in entries {
            out.push_str(&format!("        {name}: {entry}\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letters(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    /// Every index of each side appears once, in order, and the `Both` steps pair items `same`
    /// pairs.
    fn assert_well_formed<T>(
        left: &[T],
        right: &[T],
        steps: &[Step],
        same: impl Fn(&T, &T) -> bool,
    ) {
        let (mut next_left, mut next_right) = (0, 0);
        for step in steps {
            match *step {
                Step::Both(i, j) => {
                    assert_eq!((i, j), (next_left, next_right), "{steps:?}");
                    assert!(
                        same(&left[i], &right[j]),
                        "{steps:?} pairs items that differ"
                    );
                    next_left += 1;
                    next_right += 1;
                }
                Step::Left(i) => {
                    assert_eq!(i, next_left, "{steps:?}");
                    next_left += 1;
                }
                Step::Right(j) => {
                    assert_eq!(j, next_right, "{steps:?}");
                    next_right += 1;
                }
            }
        }
        assert_eq!(
            (next_left, next_right),
            (left.len(), right.len()),
            "{steps:?}"
        );
    }

    fn pairs(steps: &[Step]) -> usize {
        steps
            .iter()
            .filter(|step| matches!(step, Step::Both(..)))
            .count()
    }

    /// The longest common subsequence's length, by trying every subsequence of `left`.
    fn brute_force_lcs(left: &[u8], right: &[u8]) -> usize {
        let mut longest = 0;
        for mask in 0u32..(1 << left.len()) {
            let chosen: Vec<u8> = (0..left.len())
                .filter(|bit| mask & (1 << bit) != 0)
                .map(|bit| left[bit])
                .collect();
            if chosen.len() <= longest {
                continue;
            }
            let mut rest = right.iter();
            if chosen.iter().all(|item| rest.any(|other| other == item)) {
                longest = chosen.len();
            }
        }
        longest
    }

    /// Every sequence of up to `longest` items from an alphabet of three.
    fn every_sequence(longest: usize) -> Vec<Vec<u8>> {
        let mut all = vec![Vec::new()];
        let mut frontier = vec![Vec::new()];
        for _ in 0..longest {
            frontier = frontier
                .iter()
                .flat_map(|sequence: &Vec<u8>| {
                    (0..3).map(move |item| {
                        let mut longer = sequence.clone();
                        longer.push(item);
                        longer
                    })
                })
                .collect();
            all.extend(frontier.iter().cloned());
        }
        all
    }

    #[test]
    fn a_common_prefix_and_suffix_are_held_by_both() {
        let (left, right) = (letters("xay"), letters("xby"));
        let steps = align(&left, &right, |a, b| a == b);
        assert_eq!(
            steps,
            vec![
                Step::Both(0, 0),
                Step::Left(1),
                Step::Right(1),
                Step::Both(2, 2)
            ]
        );
    }

    #[test]
    fn a_shorter_side_that_agrees_to_its_end_leaves_the_rest_unpaired() {
        // The prefix is taken before the suffix, so the unpaired item is the last one, at the
        // position the comparison names as where the statements begin to differ.
        let (left, right) = (letters("abb"), letters("ab"));
        assert_eq!(
            align(&left, &right, |a, b| a == b),
            vec![Step::Both(0, 0), Step::Both(1, 1), Step::Left(2)]
        );
    }

    #[test]
    fn a_tie_goes_to_the_left_side() {
        let (left, right) = (letters("ab"), letters("ba"));
        assert_eq!(
            align(&left, &right, |a, b| a == b),
            vec![Step::Left(0), Step::Both(1, 0), Step::Right(1)]
        );
    }

    #[test]
    fn the_steps_pair_as_many_as_a_longest_common_subsequence_and_never_cross() {
        let sequences = every_sequence(6);
        for left in &sequences {
            for right in &sequences {
                let steps = align(left, right, |a, b| a == b);
                assert_well_formed(left, right, &steps, |a, b| a == b);
                assert_eq!(
                    pairs(&steps),
                    brute_force_lcs(left, right),
                    "{left:?} and {right:?} gave {steps:?}"
                );
            }
        }
    }

    #[test]
    fn the_same_inputs_give_the_same_steps() {
        let sequences = every_sequence(4);
        for left in &sequences {
            for right in &sequences {
                assert_eq!(
                    align(left, right, |a, b| a == b),
                    align(left, right, |a, b| a == b),
                    "{left:?} and {right:?}"
                );
            }
        }
    }

    #[test]
    fn a_predicate_other_than_equality_pairs_by_it() {
        let left = ["Alpha", "beta", "gamma"];
        let right = ["ALPHA", "delta", "GAMMA"];
        let steps = align(&left, &right, |a, b| a.eq_ignore_ascii_case(b));
        assert_well_formed(&left, &right, &steps, |a, b| a.eq_ignore_ascii_case(b));
        assert_eq!(
            steps,
            vec![
                Step::Both(0, 0),
                Step::Left(1),
                Step::Right(1),
                Step::Both(2, 2)
            ]
        );
    }

    #[test]
    fn two_empty_sequences_align_to_nothing() {
        let empty: [u8; 0] = [];
        assert!(align(&empty, &empty, |a, b| a == b).is_empty());
        assert_eq!(align(&[1u8], &empty, |a, b| a == b), vec![Step::Left(0)]);
    }
}

/// The view builder, one test per scenario of toolchain-packaging's "The command surface shows a
/// modified requirement beside the accepted one" that needs no command line. Each case is built
/// from documents projected in memory.
#[cfg(test)]
mod view_tests {
    use super::*;
    use crate::commands::Held;

    const ACCEPTED_PATH: &str = "htmlspec/specs/surface/spec.html";

    /// The model of a document holding `units`, and its marked units.
    fn projected(units: &str) -> (DocumentModel, BTreeMap<String, MarkedUnit>) {
        let source = format!(
            "<!DOCTYPE html><html lang=\"en\"><head><title>t</title></head><body><main>{units}\
             </main></body></html>"
        );
        let parsed = htmlspec_parse::parse_document(&source);
        let projection = htmlspec_parse::project(&parsed.document);
        (
            projection.model,
            htmlspec_parse::marked_units(&parsed.document),
        )
    }

    /// The accepted index of one accepted specification holding `units`.
    fn accepted(units: &str) -> AcceptedRequirements {
        let (model, _) = projected(units);
        AcceptedRequirements::of(
            model
                .requirements
                .into_iter()
                .map(|requirement| Held {
                    path: PathBuf::from(ACCEPTED_PATH),
                    requirement,
                })
                .collect(),
        )
    }

    fn requirement(id: &str, change: &str, heading: &str, body: &str) -> String {
        format!(
            "<section class=\"requirement\" id=\"{id}\" data-change=\"{change}\"><h3>{heading} \
             <span class=\"hs-id\">{id}</span></h3>{body}</section>"
        )
    }

    fn scenario(id: &str, change: &str, heading: &str, when: &str, then: &str) -> String {
        format!(
            "<article class=\"scenario\" id=\"{id}\" data-change=\"{change}\"><h4>{heading}</h4>\
             <dl><dt>When</dt><dd>{when}</dd><dt>Then</dt><dd>{then}</dd></dl></article>"
        )
    }

    fn paragraphs(texts: &[&str]) -> String {
        texts.iter().map(|text| format!("<p>{text}</p>")).collect()
    }

    /// The view of a change stating `change_units` beside an accepted specification holding
    /// `accepted_units`.
    fn shown(change_units: &str, accepted_units: &str) -> View {
        let (model, marked) = projected(change_units);
        view("a-change", &model, &marked, &accepted(accepted_units))
    }

    fn requirement_shown<'a>(view: &'a View, id: &str) -> &'a RequirementShown {
        view.requirements
            .iter()
            .find(|requirement| requirement.id == id)
            .unwrap_or_else(|| panic!("the view shows no {id}: {view:#?}"))
    }

    fn differs<'a>(view: &'a View, id: &str) -> (&'a Counterpart, &'a Comparison) {
        match &requirement_shown(view, id).shown {
            Shown::Differs(counterpart, comparison) => (counterpart, comparison),
            other => panic!("{id} is shown as {other:?}, not as differing"),
        }
    }

    /// The words only one side holds, each run trimmed, in order.
    fn marked(pieces: &[Piece]) -> Vec<&str> {
        pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::OnlyHere(text) => Some(text.trim()),
                Piece::Shared(_) => None,
            })
            .collect()
    }

    /// A side's text, from its pieces.
    fn joined(pieces: &[Piece]) -> String {
        pieces
            .iter()
            .map(|piece| match piece {
                Piece::Shared(text) | Piece::OnlyHere(text) => text.as_str(),
            })
            .collect()
    }

    fn held_by_both(paragraphs: &[Paragraph]) -> usize {
        paragraphs
            .iter()
            .filter(|paragraph| matches!(paragraph, Paragraph::Both { .. }))
            .count()
    }

    fn position(n: usize) -> Position {
        Position::new(n).expect("a position counts from one")
    }

    const SPELLINGS: &str = "The spellings are fixed: validate, archive, init, new, list, status, \
                             instructions, and store name the operations OpenSpec also provides.";
    const SPELLINGS_WITH_SYNC: &str = "The spellings are fixed: validate, archive, init, new, \
                                       list, status, sync, instructions, and store name the \
                                       operations OpenSpec also provides.";

    /// Ten statements, the third of them `third`, and eighteen scenarios marked `change`.
    fn ten_and_eighteen(third: &str, scenario_change: &str) -> String {
        let mut body = String::new();
        for n in 1..=10 {
            if n == 3 {
                body.push_str(&format!("<p>{third}</p>"));
            } else {
                body.push_str(&format!(
                    "<p>Paragraph {n} states an obligation of its own in plain words.</p>"
                ));
            }
        }
        for n in 1..=18 {
            body.push_str(&scenario(
                &format!("scenario-{n}"),
                scenario_change,
                &format!("Case {n}"),
                &format!("Case {n} happens."),
                &format!("Case {n} holds."),
            ));
        }
        body
    }

    #[test]
    fn a_one_word_modification_is_one_pair_with_one_marked_word() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The names",
                &ten_and_eighteen(SPELLINGS_WITH_SYNC, "unchanged"),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The names",
                &ten_and_eighteen(SPELLINGS, "unchanged"),
            ),
        );
        let (counterpart, comparison) = differs(&view, "req-x");
        assert_eq!(counterpart.path, PathBuf::from(ACCEPTED_PATH));
        assert_eq!(comparison.statements.marking, Marking::Comparison);
        assert_eq!(held_by_both(&comparison.statements.paragraphs), 9);
        let pairs: Vec<&Paragraph> = comparison
            .statements
            .paragraphs
            .iter()
            .filter(|paragraph| !matches!(paragraph, Paragraph::Both { .. }))
            .collect();
        let [Paragraph::Pair {
            position: at,
            accepted: accepted_at,
            accepted_holds,
            change_states,
        }] = pairs.as_slice()
        else {
            panic!("the edit is not one pair: {pairs:#?}");
        };
        assert_eq!((*at, *accepted_at), (position(3), position(3)));
        assert_eq!(marked(change_states), vec!["sync,"]);
        assert!(marked(accepted_holds).is_empty(), "{accepted_holds:?}");
        assert_eq!(
            joined(change_states),
            SPELLINGS_WITH_SYNC
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        );
        assert_eq!(
            joined(accepted_holds),
            SPELLINGS.split_whitespace().collect::<Vec<_>>().join(" ")
        );
        assert_eq!(comparison.scenarios.len(), 18);
        assert!(comparison
            .scenarios
            .iter()
            .all(|scenario| matches!(scenario, ScenarioShown::Both { .. })));
    }

    #[test]
    fn a_paragraph_rewritten_throughout_is_shown_whole_on_each_side() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                &paragraphs(&[
                    "The system SHALL hold the first obligation.",
                    "Every archived change records the version it shipped in, once.",
                ]),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &paragraphs(&[
                    "The system SHALL hold the first obligation.",
                    "A runbook names its prerequisites before any step starts.",
                ]),
            ),
        );
        let (_, comparison) = differs(&view, "req-x");
        let paragraphs = &comparison.statements.paragraphs;
        assert_eq!(held_by_both(paragraphs), 1);
        assert!(
            paragraphs.iter().any(|paragraph| matches!(
                paragraph,
                Paragraph::Accepted { accepted, text }
                    if *accepted == position(2) && text.starts_with("A runbook names")
            )),
            "{paragraphs:#?}"
        );
        assert!(
            paragraphs.iter().any(|paragraph| matches!(
                paragraph,
                Paragraph::Change { position: at, text }
                    if *at == position(2) && text.starts_with("Every archived change")
            )),
            "{paragraphs:#?}"
        );
        assert!(
            !paragraphs
                .iter()
                .any(|paragraph| matches!(paragraph, Paragraph::Pair { .. })),
            "a paragraph sharing fewer than half its words was paired: {paragraphs:#?}"
        );
    }

    #[test]
    fn a_paragraph_added_beside_an_edited_one_is_shown_apart_from_the_pair() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                &paragraphs(&[
                    "The system SHALL hold the first obligation.",
                    "Nothing here resembles anything that came before it at all.",
                    "Each change SHALL state its whole requirement once merged.",
                ]),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &paragraphs(&[
                    "The system SHALL hold the first obligation.",
                    "Each change SHALL state its requirement once merged.",
                ]),
            ),
        );
        let (_, comparison) = differs(&view, "req-x");
        let paragraphs = &comparison.statements.paragraphs;
        assert!(
            paragraphs.iter().any(|paragraph| matches!(
                paragraph,
                Paragraph::Change { position: at, text }
                    if *at == position(2) && text.starts_with("Nothing here")
            )),
            "the added paragraph is not shown whole: {paragraphs:#?}"
        );
        let pair = paragraphs
            .iter()
            .find_map(|paragraph| match paragraph {
                Paragraph::Pair {
                    position: at,
                    accepted,
                    change_states,
                    accepted_holds,
                } => Some((*at, *accepted, change_states, accepted_holds)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("the edited paragraph is not paired: {paragraphs:#?}"));
        assert_eq!((pair.0, pair.1), (position(3), position(2)));
        assert_eq!(marked(pair.2), vec!["whole"]);
        assert!(marked(pair.3).is_empty());
    }

    #[test]
    fn the_authors_marks_are_shown_where_they_account_for_the_difference() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                "<p>The merge SHALL keep <del>the whole list</del><ins>every list</ins> of \
                 scenarios, <ins>exactly </ins>as stated<del> today</del>.</p>",
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                "<p>The merge SHALL keep the whole list of scenarios, as stated today.</p>",
            ),
        );
        let (_, comparison) = differs(&view, "req-x");
        assert_eq!(comparison.statements.marking, Marking::Author);
        let [Paragraph::Pair {
            accepted_holds,
            change_states,
            ..
        }] = comparison.statements.paragraphs.as_slice()
        else {
            panic!("{:#?}", comparison.statements.paragraphs);
        };
        // As the author marked them, and not as a computed alignment would, which keeps "list".
        assert_eq!(marked(accepted_holds), vec!["the whole list", "today"]);
        assert_eq!(marked(change_states), vec!["every list", "exactly"]);
        assert_eq!(
            joined(change_states).trim(),
            "The merge SHALL keep every list of scenarios, exactly as stated."
        );
        assert_eq!(
            joined(accepted_holds).trim(),
            "The merge SHALL keep the whole list of scenarios, as stated today."
        );
    }

    #[test]
    fn the_authors_marks_are_set_aside_where_they_leave_part_unmarked() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                "<p>The merge SHALL keep <ins>every </ins>scenario as stated.</p>",
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                "<p>The merge SHALL always keep scenario as stated.</p>",
            ),
        );
        let (_, comparison) = differs(&view, "req-x");
        assert_eq!(comparison.statements.marking, Marking::Unreconciled);
        let [Paragraph::Pair {
            accepted_holds,
            change_states,
            ..
        }] = comparison.statements.paragraphs.as_slice()
        else {
            panic!("{:#?}", comparison.statements.paragraphs);
        };
        assert_eq!(marked(change_states), vec!["every"]);
        assert_eq!(marked(accepted_holds), vec!["always"]);
    }

    #[test]
    fn a_scenario_the_change_alters_is_paired_by_identifier() {
        // The altered scenario moves to the front and takes a new heading, so neither its
        // position nor its heading would find its counterpart.
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                &format!(
                    "<p>The system SHALL do x.</p>{}{}{}",
                    scenario(
                        "scenario-c",
                        "modified",
                        "Case C, renamed",
                        "C happens.",
                        "Case C holds for every archived change."
                    ),
                    scenario(
                        "scenario-a",
                        "unchanged",
                        "Case A",
                        "A happens.",
                        "A holds."
                    ),
                    scenario(
                        "scenario-b",
                        "unchanged",
                        "Case B",
                        "B happens.",
                        "B holds."
                    ),
                ),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &format!(
                    "<p>The system SHALL do x.</p>{}{}{}",
                    scenario(
                        "scenario-a",
                        "unchanged",
                        "Case A",
                        "A happens.",
                        "A holds."
                    ),
                    scenario(
                        "scenario-b",
                        "unchanged",
                        "Case B",
                        "B happens.",
                        "B holds."
                    ),
                    scenario(
                        "scenario-c",
                        "unchanged",
                        "Case C",
                        "C happens.",
                        "Case C holds for every change."
                    ),
                ),
            ),
        );
        let (_, comparison) = differs(&view, "req-x");
        assert_eq!(held_by_both(&comparison.statements.paragraphs), 1);
        let pairs: Vec<&ScenarioPair> = comparison
            .scenarios
            .iter()
            .filter_map(|scenario| match scenario {
                ScenarioShown::Pair(pair) => Some(pair),
                _ => None,
            })
            .collect();
        let [pair] = pairs.as_slice() else {
            panic!("{:#?}", comparison.scenarios);
        };
        assert_eq!(pair.id, "scenario-c");
        assert_eq!(pair.declared, "modified");
        assert_eq!(
            pair.heading.as_ref().map(HeadingName::as_str),
            Some("Case C, renamed")
        );
        assert_eq!(
            pair.accepted_heading.as_ref().map(HeadingName::as_str),
            Some("Case C")
        );
        assert_eq!(held_by_both(&pair.when), 1);
        let [Paragraph::Pair {
            accepted_holds,
            change_states,
            ..
        }] = pair.then.as_slice()
        else {
            panic!("{:#?}", pair.then);
        };
        assert_eq!(marked(change_states), vec!["archived"]);
        assert!(marked(accepted_holds).is_empty(), "{accepted_holds:?}");
        let both: Vec<Option<&str>> = comparison
            .scenarios
            .iter()
            .filter_map(|scenario| match scenario {
                ScenarioShown::Both { id, .. } => Some(id.as_deref()),
                _ => None,
            })
            .collect();
        assert_eq!(both, vec![Some("scenario-a"), Some("scenario-b")]);
    }

    #[test]
    fn scenarios_added_withdrawn_and_left_out_are_each_shown_as_such() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                &format!(
                    "<p>The system SHALL do x.</p>{}{}{}",
                    scenario(
                        "scenario-a",
                        "unchanged",
                        "Case A",
                        "A happens.",
                        "A holds."
                    ),
                    scenario("scenario-n", "added", "Case N", "N happens.", "N holds."),
                    scenario("scenario-b", "removed", "Case B", "B happens.", "B holds."),
                ),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &format!(
                    "<p>The system SHALL do x.</p>{}{}{}",
                    scenario(
                        "scenario-a",
                        "unchanged",
                        "Case A",
                        "A happens.",
                        "A holds."
                    ),
                    scenario(
                        "scenario-b",
                        "unchanged",
                        "Case B",
                        "B happens.",
                        "B holds."
                    ),
                    scenario(
                        "scenario-l",
                        "unchanged",
                        "Case L",
                        "L happens.",
                        "L holds."
                    ),
                ),
            ),
        );
        let (_, comparison) = differs(&view, "req-x");
        let kinds: Vec<String> = comparison
            .scenarios
            .iter()
            .map(|scenario| match scenario {
                ScenarioShown::Both { id, .. } => format!("both {}", id.as_deref().unwrap_or("")),
                ScenarioShown::Pair(pair) => format!("pair {}", pair.id),
                ScenarioShown::Change { scenario, .. } => {
                    format!("change {}", scenario.id.as_deref().unwrap_or(""))
                }
                ScenarioShown::Withdrawn { id, still_held, .. } => format!(
                    "withdrawn {} {}",
                    id.as_deref().unwrap_or(""),
                    if *still_held { "still held" } else { "gone" }
                ),
                ScenarioShown::Accepted(scenario) => {
                    format!("accepted {}", scenario.id.as_deref().unwrap_or(""))
                }
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                "both scenario-a",
                "change scenario-n",
                "withdrawn scenario-b still held",
                "accepted scenario-l",
            ]
        );
    }

    #[test]
    fn a_modification_already_merged_is_held_as_stated() {
        let body = format!(
            "<p>The system SHALL do x.</p>{}",
            scenario(
                "scenario-a",
                "unchanged",
                "Case A",
                "A happens.",
                "A holds."
            )
        );
        let view = shown(
            &requirement("req-x", "modified", "The x requirement", &body),
            &requirement("req-x", "unchanged", "The x requirement", &body),
        );
        match &requirement_shown(&view, "req-x").shown {
            Shown::HeldAsStated(counterpart) => {
                assert_eq!(counterpart.path, PathBuf::from(ACCEPTED_PATH));
            }
            other => panic!("{other:?}"),
        }
    }

    /// A withdrawn scenario with no identifier is still held where the accepted requirement holds
    /// its text, and the view says so once, not as "no longer held" and again as held only by the
    /// accepted specification. Found by the code review of PR #47.
    #[test]
    fn a_withdrawn_scenario_with_no_identifier_is_found_by_its_text() {
        let gone = "<article class=\"scenario\" data-change=\"removed\"><h4>Case G</h4><dl>\
                    <dt>When</dt><dd>G happens.</dd><dt>Then</dt><dd>G holds.</dd></dl></article>";
        let held = gone.replace("removed", "unchanged");
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                &format!("<p>The system SHALL do x at once.</p>{gone}"),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &format!("<p>The system SHALL do x.</p>{held}"),
            ),
        );
        let (_, comparison) = differs(&view, "req-x");
        assert!(
            matches!(
                comparison.scenarios.as_slice(),
                [ScenarioShown::Withdrawn {
                    id: None,
                    still_held: true,
                    ..
                }]
            ),
            "{:#?}",
            comparison.scenarios
        );
    }

    /// The comparison reads scenarios as a collection, so a merge that put them in another order
    /// took them. The view says what the comparison says, and does not decide "differs" by
    /// comparing them in order.
    #[test]
    fn a_modification_merged_with_its_scenarios_reordered_is_held_as_stated() {
        let a = scenario(
            "scenario-a",
            "unchanged",
            "Case A",
            "A happens.",
            "A holds.",
        );
        let b = scenario("scenario-b", "added", "Case B", "B happens.", "B holds.");
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                &format!("<p>The system SHALL do x.</p>{a}{b}"),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &format!("<p>The system SHALL do x.</p>{b}{a}"),
            ),
        );
        assert!(
            matches!(
                requirement_shown(&view, "req-x").shown,
                Shown::HeldAsStated(_)
            ),
            "{view:#?}"
        );
    }

    #[test]
    fn a_modified_requirement_no_accepted_specification_holds_is_held_by_none() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The x requirement",
                "<p>The system SHALL do x.</p>",
            ),
            &requirement(
                "req-y",
                "unchanged",
                "The y requirement",
                "<p>The system SHALL do y.</p>",
            ),
        );
        assert!(matches!(
            requirement_shown(&view, "req-x").shown,
            Shown::HeldByNone
        ));
    }

    #[test]
    fn a_renamed_requirement_shows_both_headings() {
        let view = shown(
            &requirement(
                "req-x",
                "modified",
                "The new name",
                "<p data-field=\"renamed-from\">The old name</p><p>The system SHALL do x \
                 quickly.</p>",
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The old name",
                "<p>The system SHALL do x.</p>",
            ),
        );
        let stated = requirement_shown(&view, "req-x");
        assert_eq!(
            stated.heading.as_ref().map(HeadingName::as_str),
            Some("The new name")
        );
        let (counterpart, _) = differs(&view, "req-x");
        assert_eq!(
            counterpart.heading.as_ref().map(HeadingName::as_str),
            Some("The old name")
        );
    }

    #[test]
    fn an_added_requirement_held_otherwise_is_compared() {
        let view = shown(
            &requirement(
                "req-x",
                "added",
                "The x requirement",
                "<p>The system SHALL do x quickly at once.</p>",
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                "<p>The system SHALL do x at once.</p>",
            ),
        );
        assert_eq!(requirement_shown(&view, "req-x").declared, "added");
        let (_, comparison) = differs(&view, "req-x");
        let [Paragraph::Pair { change_states, .. }] = comparison.statements.paragraphs.as_slice()
        else {
            panic!("{:#?}", comparison.statements.paragraphs);
        };
        assert_eq!(marked(change_states), vec!["quickly"]);
    }

    #[test]
    fn a_scenario_changed_inside_an_unchanged_requirement_is_compared() {
        let view = shown(
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &format!(
                    "<p>The system SHALL do x.</p>{}",
                    scenario(
                        "scenario-a",
                        "modified",
                        "Case A",
                        "A happens.",
                        "A holds at once."
                    )
                ),
            ),
            &requirement(
                "req-x",
                "unchanged",
                "The x requirement",
                &format!(
                    "<p>The system SHALL do x.</p>{}",
                    scenario(
                        "scenario-a",
                        "unchanged",
                        "Case A",
                        "A happens.",
                        "A holds."
                    )
                ),
            ),
        );
        assert_eq!(requirement_shown(&view, "req-x").declared, "unchanged");
        let (_, comparison) = differs(&view, "req-x");
        assert!(
            comparison.scenarios.iter().any(
                |scenario| matches!(scenario, ScenarioShown::Pair(pair) if pair.id == "scenario-a")
            ),
            "{:#?}",
            comparison.scenarios
        );
    }

    #[test]
    fn requirements_the_comparison_does_not_compare_are_listed() {
        let view = shown(
            &format!(
                "{}{}",
                requirement(
                    "req-gone",
                    "removed",
                    "The withdrawn requirement",
                    "<p>The system SHALL do what is withdrawn.</p>"
                ),
                requirement(
                    "req-kept",
                    "unchanged",
                    "The kept requirement",
                    &format!(
                        "<p>The system SHALL do what is kept, in other words.</p>{}",
                        scenario(
                            "scenario-k",
                            "unchanged",
                            "Case K",
                            "K happens.",
                            "K holds."
                        )
                    )
                ),
            ),
            &format!(
                "{}{}",
                requirement(
                    "req-gone",
                    "unchanged",
                    "The withdrawn requirement",
                    "<p>The system SHALL do what is withdrawn.</p>"
                ),
                requirement(
                    "req-kept",
                    "unchanged",
                    "The kept requirement",
                    &format!(
                        "<p>The system SHALL do what is kept.</p>{}",
                        scenario(
                            "scenario-k",
                            "unchanged",
                            "Case K",
                            "K happens.",
                            "K holds."
                        )
                    )
                ),
            ),
        );
        let listed: Vec<(&str, &str, Option<&str>)> = view
            .requirements
            .iter()
            .map(|requirement| {
                assert!(
                    matches!(requirement.shown, Shown::NotCompared),
                    "{} was compared: {:?}",
                    requirement.id,
                    requirement.shown
                );
                (
                    requirement.id.as_str(),
                    requirement.declared.as_str(),
                    requirement.heading.as_ref().map(HeadingName::as_str),
                )
            })
            .collect();
        assert_eq!(
            listed,
            vec![
                ("req-gone", "removed", Some("The withdrawn requirement")),
                ("req-kept", "unchanged", Some("The kept requirement")),
            ]
        );
    }
}
