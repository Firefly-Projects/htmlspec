//! Where a document sits, and what the places its references reach declare.
//!
//! document-schema's "Cross-references address units by identifier": a `data-refs` entry resolves
//! in the document that makes it, then, for one of a change's documents, in that change's
//! specification, then in the project's accepted specifications. The first place that declares the
//! identifier is the one the reference means.
//!
//! The types are here and the reading is not. The command line reads the other documents and hands
//! a rule the identifiers they declare, for the reason [`crate::Subject`] gives for the asset store:
//! a rule that read the filesystem itself could depend on a path that exists on one machine and not
//! another. So a rule is given sets, and a conformance suite can build a scope at any placement
//! without touching a file.

use std::collections::BTreeSet;

use htmlspec_parse::Projection;

/// Where a document sits in a project, as far as its references are concerned.
///
/// A place, not a kind. A change's specification, an accepted specification, and a runbook's
/// specification all declare the specification kind, and they reach different places.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Placement {
    /// Outside any project, beneath a change's evidence, or anywhere else the layout does not name.
    Loose,
    /// Directly within a change's directory, active or archived, and not its specification.
    ChangeDocument,
    /// A change's `specification.html`, directly within the change's directory.
    ChangeSpecification,
    /// Beneath the place the project keeps its accepted specifications, at any depth.
    AcceptedSpecification,
    /// Beneath the place the project keeps its runbooks, a run's record included.
    RunbookDocument,
}

/// A place beyond a document that its references can reach.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Place {
    /// The specification of the change the document belongs to.
    ChangeSpecification,
    /// Every accepted specification, less the document itself where it is one.
    AcceptedSpecifications,
}

impl Placement {
    /// The places a document here reaches after itself, in the order they are tried.
    ///
    /// The order is stated here and nowhere else. A change's specification comes before the
    /// accepted specifications because it restates, under the accepted identifier, each accepted
    /// requirement the change modifies or withdraws, and for the change's own documents the
    /// change's statement is the one that counts.
    pub const fn reaches(self) -> &'static [Place] {
        match self {
            Self::Loose | Self::RunbookDocument => &[],
            Self::ChangeDocument => &[Place::ChangeSpecification, Place::AcceptedSpecifications],
            Self::ChangeSpecification | Self::AcceptedSpecification => {
                &[Place::AcceptedSpecifications]
            }
        }
    }
}

impl Place {
    /// How a report names the place, after "in".
    fn named(self) -> &'static str {
        match self {
            Self::ChangeSpecification => "its change's specification",
            Self::AcceptedSpecifications => "the accepted specifications",
        }
    }
}

/// A document a place holds that could not be read for what it declares.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Unread {
    /// The document, as a report names it.
    pub document: String,
    /// Why it could not be read.
    pub reason: String,
}

/// What one place declares, as far as resolving a reference needs to know.
///
/// Built from projections, so a figure identifier, which a reference can never name, is never
/// gathered, and the withdrawn state is found the way the reference rule finds it in its own
/// document. A document that could not be read is kept as [`Unread`] rather than read as declaring
/// nothing: a report that no unit declares an identifier would send an author to add a unit that
/// may already be there.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Declared {
    units: BTreeSet<String>,
    withdrawn: BTreeSet<String>,
    unread: Vec<Unread>,
}

impl Declared {
    /// What `projections` declare, and the documents of the place that could not be read.
    ///
    /// An identifier that two of them declare is declared, and it is withdrawn if either marks it
    /// so: two accepted specifications declaring one identifier is a fault in those specifications,
    /// and a reference names a unit, never a capability, so it cannot choose between them.
    pub fn gather<'a>(
        projections: impl IntoIterator<Item = &'a Projection>,
        unread: Vec<Unread>,
    ) -> Self {
        let removed = crate::rules::refs::removed_state();
        let mut declared = Self {
            unread,
            ..Self::default()
        };
        for projection in projections {
            for unit in projection.model.units() {
                let Some(id) = unit.id else {
                    continue;
                };
                declared.units.insert(id.to_owned());
                if removed.as_deref() == Some(unit.change) {
                    declared.withdrawn.insert(id.to_owned());
                }
            }
        }
        declared
    }
}

/// A document's placement, and what each place it reaches declares.
#[derive(Clone, Debug)]
pub struct Scope {
    placement: Placement,
    places: Vec<(Place, Declared)>,
}

/// What a scope says about an identifier its own document does not declare.
pub(crate) enum Found<'a> {
    /// A place declares it. `withdrawn` says whether that place marks it removed.
    In { place: Place, withdrawn: bool },
    /// No place declares it. `searched` names every place tried, and `unread` every document
    /// among them that could not be read.
    Nowhere {
        searched: Vec<Place>,
        unread: Vec<&'a Unread>,
    },
}

impl Scope {
    /// A document that reaches only itself: outside any project, or validated on its own.
    pub fn loose() -> Self {
        Self {
            placement: Placement::Loose,
            places: Vec::new(),
        }
    }

    /// The scope of a document at `placement`, asking `declared` once for each place it reaches.
    ///
    /// So a caller cannot supply a place the placement does not reach, and cannot leave one out.
    pub fn gather(placement: Placement, mut declared: impl FnMut(Place) -> Declared) -> Self {
        Self {
            placement,
            places: placement
                .reaches()
                .iter()
                .map(|&place| (place, declared(place)))
                .collect(),
        }
    }

    /// Where the document sits.
    pub fn placement(&self) -> Placement {
        self.placement
    }

    /// The first place that declares `target`, tried in order, or every place searched.
    pub(crate) fn find(&self, target: &str) -> Found<'_> {
        for (place, declared) in &self.places {
            if declared.units.contains(target) {
                return Found::In {
                    place: *place,
                    withdrawn: declared.withdrawn.contains(target),
                };
            }
        }
        Found::Nowhere {
            searched: self.places.iter().map(|(place, _)| *place).collect(),
            unread: self
                .places
                .iter()
                .flat_map(|(_, declared)| &declared.unread)
                .collect(),
        }
    }
}

/// "this document", then each place searched, as a report lists them after "in".
///
/// "in this document, in its change's specification, or in the accepted specifications".
pub(crate) fn searched(places: &[Place]) -> String {
    let mut named: Vec<String> = vec![String::from("in this document")];
    named.extend(places.iter().map(|place| format!("in {}", place.named())));
    match named.len() {
        0 | 1 => named.concat(),
        2 => named.join(" or "),
        count => format!("{}, or {}", named[..count - 1].join(", "), named[count - 1]),
    }
}

/// How a report names `place` when a reference resolves there to a withdrawn unit.
pub(crate) fn withdrawing(place: Place) -> &'static str {
    match place {
        Place::ChangeSpecification => "its change's specification withdraws",
        Place::AcceptedSpecifications => "the accepted specifications mark removed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// design decision 1's table, read row by row.
    #[test]
    fn each_placement_reaches_the_places_of_the_design_in_order() {
        assert_eq!(Placement::Loose.reaches(), &[] as &[Place]);
        assert_eq!(Placement::RunbookDocument.reaches(), &[] as &[Place]);
        assert_eq!(
            Placement::ChangeDocument.reaches(),
            &[Place::ChangeSpecification, Place::AcceptedSpecifications]
        );
        assert_eq!(
            Placement::ChangeSpecification.reaches(),
            &[Place::AcceptedSpecifications]
        );
        assert_eq!(
            Placement::AcceptedSpecification.reaches(),
            &[Place::AcceptedSpecifications]
        );
    }

    /// `gather` asks once for each place the placement reaches, in its order, and for no other.
    #[test]
    fn gather_asks_once_for_each_place_reached() {
        for placement in [
            Placement::Loose,
            Placement::ChangeDocument,
            Placement::ChangeSpecification,
            Placement::AcceptedSpecification,
            Placement::RunbookDocument,
        ] {
            let mut asked = Vec::new();
            let scope = Scope::gather(placement, |place| {
                asked.push(place);
                Declared::default()
            });
            assert_eq!(asked, placement.reaches(), "{placement:?}");
            assert_eq!(scope.placement(), placement);
        }
    }

    #[test]
    fn a_loose_scope_reaches_nothing() {
        let scope = Scope::loose();
        assert_eq!(scope.placement(), Placement::Loose);
        match scope.find("req-x") {
            Found::Nowhere { searched, unread } => {
                assert!(searched.is_empty());
                assert!(unread.is_empty());
            }
            Found::In { .. } => panic!("a loose scope found something"),
        }
    }

    #[test]
    fn the_places_searched_read_as_a_list() {
        assert_eq!(searched(&[]), "in this document");
        assert_eq!(
            searched(&[Place::AcceptedSpecifications]),
            "in this document or in the accepted specifications"
        );
        assert_eq!(
            searched(&[Place::ChangeSpecification, Place::AcceptedSpecifications]),
            "in this document, in its change's specification, or in the accepted specifications"
        );
    }
}
