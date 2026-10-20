//! Which attribute values in a document are asset references.
//!
//! Two readers ask, and they must give one answer. The asset rules read a document's tree
//! and judge each reference against the store. The formatter reads its source and fills
//! each reference whose bytes are absent. When the two kept separate lists, the formatter
//! filled values the rules never judged: an author's quotation of `href="…"` in prose, and a
//! hyperlink. So the list lives here, below both, and each reads it.

/// An attribute through which an element loads an asset, named by the element and the
/// attribute.
///
/// Both names are lowercase, which is how the parser reports them whatever case a
/// document writes them in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferencingAttribute {
    /// The element's name, such as `link`.
    pub element: &'static str,
    /// The attribute's name, such as `href`.
    pub attribute: &'static str,
}

impl ReferencingAttribute {
    const fn new(element: &'static str, attribute: &'static str) -> Self {
        Self { element, attribute }
    }

    /// The pair `element` and `attribute` name, if it loads an asset.
    ///
    /// Both names are compared as given, so a caller passes them lowercased, as the parser
    /// does.
    pub fn find(element: &str, attribute: &str) -> Option<Self> {
        REFERENCING_ATTRIBUTES
            .iter()
            .copied()
            .find(|pair| pair.element == element && pair.attribute == attribute)
    }
}

/// The attributes that reference a stored asset.
///
/// The loading attributes, which is the same set `safety/*` reads for a different
/// question — that rule asks whether a reference points *outward*, this one asks
/// whether it points at the bytes it claims. `href` on `a` is absent for the same
/// reason it is there: a hyperlink is a citation, not a dependency, and a document
/// that links to another document is not claiming anything about its bytes.
///
/// `srcset` is absent although `safety/*` reads it, because its value lists several URLs
/// with their sizes, and no one of them is a reference the store can resolve as written.
pub const REFERENCING_ATTRIBUTES: &[ReferencingAttribute] = &[
    ReferencingAttribute::new("script", "src"),
    ReferencingAttribute::new("link", "href"),
    ReferencingAttribute::new("img", "src"),
    ReferencingAttribute::new("source", "src"),
    ReferencingAttribute::new("iframe", "src"),
    ReferencingAttribute::new("embed", "src"),
    ReferencingAttribute::new("object", "data"),
    ReferencingAttribute::new("video", "src"),
    ReferencingAttribute::new("audio", "src"),
    ReferencingAttribute::new("track", "src"),
    ReferencingAttribute::new("input", "src"),
    ReferencingAttribute::new("use", "href"),
    ReferencingAttribute::new("image", "href"),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// A hyperlink is a citation, so it is not on the list however its path reads.
    #[test]
    fn a_hyperlink_is_not_a_referencing_attribute() {
        assert_eq!(ReferencingAttribute::find("a", "href"), None);
    }

    /// The parser reports names in lowercase, and the list is written to match.
    #[test]
    fn every_name_on_the_list_is_lowercase() {
        for pair in REFERENCING_ATTRIBUTES {
            assert_eq!(pair.element, pair.element.to_ascii_lowercase());
            assert_eq!(pair.attribute, pair.attribute.to_ascii_lowercase());
        }
    }

    /// Each pair is found by its own names.
    #[test]
    fn every_pair_is_found_by_its_names() {
        for pair in REFERENCING_ATTRIBUTES {
            assert_eq!(
                ReferencingAttribute::find(pair.element, pair.attribute),
                Some(*pair)
            );
        }
    }
}
