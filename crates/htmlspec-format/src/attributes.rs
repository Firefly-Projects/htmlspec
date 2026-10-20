//! The prescribed order of an element's attributes.
//!
//! Order is fully determined, because two agents that produce the same semantic
//! document with different attribute order must format to byte-identical files.
//!
//! The order is *semantic first, then alphabetical*: the attributes that carry
//! contract meaning lead, in a fixed sequence that reads the way a reviewer scans
//! an element — what it is, what it is called, what state it is in, what it points
//! at — and everything else follows in alphabetical order. Alphabetical alone
//! would put `class` after `aria-label` and bury the reserved class that decides
//! whether the element is a unit at all.

use kuchikiki::ExpandedName;

/// The attributes that lead, in the order they are written.
///
/// These are the ones the contract reads: the reserved class that creates a unit,
/// the identifier, the change state, the per-unit state attributes (a risk's response
/// among them), and the reference list. Anything the projection ignores is not here.
const LEADING: &[&str] = &[
    "class",
    "id",
    "data-change",
    "data-state",
    "data-status",
    "data-outcome",
    "data-response",
    "data-superseded-by",
    "data-field",
    "data-refs",
];

/// The sort key of one attribute name.
///
/// A leading attribute sorts by its position in [`LEADING`]; everything else
/// sorts after all of them, alphabetically. Returning a tuple rather than an
/// index keeps the two groups from ever interleaving.
fn sort_key(name: &str) -> (usize, &str) {
    match LEADING.iter().position(|leading| *leading == name) {
        Some(at) => (at, name),
        None => (LEADING.len(), name),
    }
}

/// One attribute, ready to be written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    /// The attribute's name, lowercased by the parser.
    pub name: String,
    /// Its value, as the parser resolved it.
    pub value: String,
}

/// Reads an element's attributes into prescribed order.
///
/// A namespaced attribute is written with its prefix restored, so that an `xlink`
/// or `xml` attribute inside embedded SVG or MathML survives formatting. Its
/// prefix is part of the name for ordering purposes, which is enough to be
/// deterministic and does not require the formatter to model namespaces.
pub fn ordered(attributes: &kuchikiki::Attributes) -> Vec<Attribute> {
    let mut collected: Vec<Attribute> = attributes
        .map
        .iter()
        .map(|(name, attribute)| Attribute {
            name: qualified_name(name, attribute),
            value: attribute.value.clone(),
        })
        .collect();
    // Sorting by the key alone is a total order because a duplicate attribute
    // name cannot survive parsing: the parser drops the second and reports a
    // recovery, which validation treats as fatal.
    collected.sort_by(|a, b| sort_key(&a.name).cmp(&sort_key(&b.name)));
    collected
}

/// The name as it is written back, prefix included.
fn qualified_name(name: &ExpandedName, attribute: &kuchikiki::Attribute) -> String {
    match &attribute.prefix {
        Some(prefix) => format!("{prefix}:{}", name.local),
        None => name.local.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use htmlspec_parse::parse_document;

    /// Reads the first element matching `selector` and returns its ordered
    /// attribute names.
    fn names_of(html: &str, selector: &str) -> Vec<String> {
        let outcome = parse_document(&format!(
            "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
             <body>{html}</body></html>\n"
        ));
        assert!(
            outcome.is_recovery_free(),
            "the sample did not parse cleanly: {:#?}",
            outcome.recovered
        );
        let element = outcome
            .document
            .select_first(selector)
            .expect("the element parsed");
        let names: Vec<String> = ordered(&element.attributes.borrow())
            .into_iter()
            .map(|a| a.name)
            .collect();
        names
    }

    /// The scenario the specs state: a class, an identifier, and a change state
    /// appear in the prescribed order regardless of how they were written.
    #[test]
    fn the_contract_attributes_lead_in_their_prescribed_order() {
        let written_backwards =
            r#"<section data-change="added" id="req-1" class="requirement"></section>"#;
        assert_eq!(
            names_of(written_backwards, "section"),
            ["class", "id", "data-change"]
        );
    }

    /// Two authoring orders of the same attributes must produce one order, or
    /// two agents cannot converge.
    #[test]
    fn any_authoring_order_produces_one_order() {
        let first = r#"<li class="task" data-refs="req-1" id="task-1" data-state="done"></li>"#;
        let second = r#"<li data-state="done" data-refs="req-1" class="task" id="task-1"></li>"#;
        let expected = ["class", "id", "data-state", "data-refs"];
        assert_eq!(names_of(first, "li"), expected);
        assert_eq!(names_of(second, "li"), expected);
    }

    /// A risk's response is a per-unit state attribute and leads with the others.
    ///
    /// Outside the leading list it would sort after every leading attribute, `data-refs`
    /// among them, and after any ignored attribute that sorts before it alphabetically,
    /// such as `aria-label`. Both are in the sample, because a risk carrying only `class`,
    /// `id`, and `data-change` beside it is written the same either way.
    #[test]
    fn a_risks_response_leads_with_the_other_state_attributes() {
        let html = r#"<section lang="en" data-refs="req-1" data-response="accepted"
             aria-label="a" data-change="added" id="risk-1" class="risk"></section>"#;
        assert_eq!(
            names_of(html, "section"),
            [
                "class",
                "id",
                "data-change",
                "data-response",
                "data-refs",
                "aria-label",
                "lang"
            ]
        );
    }

    /// Everything the projection ignores sorts alphabetically after the
    /// contract attributes, so an unrecognized attribute has a determined place
    /// without the list having to name it.
    #[test]
    fn other_attributes_follow_alphabetically() {
        let html = r#"<section title="t" lang="en" class="risk" role="note"
             aria-label="a" id="risk-1"></section>"#;
        assert_eq!(
            names_of(html, "section"),
            ["class", "id", "aria-label", "lang", "role", "title"]
        );
    }

    /// A `data-*` attribute the contract does not read is an ordinary attribute
    /// and must not be mistaken for a leading one by its prefix.
    #[test]
    fn an_unrecognized_data_attribute_is_not_treated_as_a_contract_attribute() {
        let html = r#"<div data-zebra="z" data-alpha="a" id="d" class="c"></div>"#;
        assert_eq!(
            names_of(html, "div"),
            ["class", "id", "data-alpha", "data-zebra"]
        );
    }

    /// A boolean attribute has an empty value and still holds its place.
    #[test]
    fn a_valueless_attribute_keeps_its_place() {
        let html = r#"<details open id="d"></details>"#;
        assert_eq!(names_of(html, "details"), ["id", "open"]);
    }

    /// The leading list decides an ordering, so a duplicate in it would make two
    /// names claim one position and the order would depend on which was found.
    #[test]
    fn the_leading_list_has_no_duplicate() {
        let mut sorted: Vec<&str> = LEADING.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), LEADING.len(), "LEADING has a duplicate");
    }

    /// Ordering is a pure function of the names, so it does not depend on the
    /// order the parser happened to store them in.
    #[test]
    fn the_order_is_a_function_of_the_names_alone() {
        let mut keys: Vec<(usize, &str)> = LEADING.iter().map(|name| sort_key(name)).collect();
        let before = keys.clone();
        keys.sort_unstable();
        assert_eq!(
            before, keys,
            "the leading list is not in its own sort order"
        );
    }
}
