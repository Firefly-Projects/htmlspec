//! How an element's name decides the way it is serialized.
//!
//! Three questions, and the answer to each is a property of the *name* alone, so
//! the serializer never inspects content to decide layout. That is what keeps the
//! canonical form a function of the tree rather than of the source it came from.
//!
//! The defaults are chosen so that an unrecognized name cannot change how a
//! document renders. An unknown element is inline, because treating an inline
//! element as a block inserts line breaks where whitespace is significant and
//! changes what a reader sees, whereas treating a block as inline only produces a
//! longer line. The safe direction is the default.

/// How an element's children are serialized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentMode {
    /// The element has no children and no closing tag.
    Void,
    /// Children are laid out by the serializer: indented blocks, or inline
    /// content on the element's own line.
    Laid,
    /// Children are text, and every byte of it survives — indentation, newlines,
    /// and trailing spaces alike. This is `pre`, `code`, and `textarea`, the three
    /// the projection specification calls preserved text (section 6.2).
    Preserved,
    /// Content is raw text that is not HTML and is never escaped: `script` and
    /// `style`. The projection reads no text from either one (section 8.5).
    Raw,
}

/// Elements with no closing tag, per the HTML specification.
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Elements laid out as blocks: each on its own line, children indented.
///
/// The list is closed and explicit rather than derived from a stylesheet's notion
/// of display, because the canonical form must not change when a stylesheet does.
const BLOCK: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "caption",
    "colgroup",
    "dd",
    "details",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "li",
    "link",
    "main",
    "meta",
    "nav",
    "ol",
    "p",
    "pre",
    "script",
    "section",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "ul",
];

/// Elements whose text content is preserved byte-for-byte.
///
/// These are exactly the three the projection specification names, so the
/// formatter and the projection agree about which content is whitespace
/// sensitive.
const PRESERVED: &[&str] = &["code", "pre", "textarea"];

/// Elements whose content is raw text rather than markup.
const RAW: &[&str] = &["script", "style"];

/// Elements the HTML parser strips one leading newline from.
///
/// The parser discards a newline immediately after the open tag of these two, so
/// a serializer that writes preserved content beginning with a newline would lose
/// that newline on the next parse and the formatter would not be a fixed point.
/// The serializer writes an extra newline for exactly these elements.
const NEWLINE_STRIPPED: &[&str] = &["pre", "textarea"];

/// Whether the element has no closing tag.
pub fn is_void(name: &str) -> bool {
    VOID.contains(&name)
}

/// Whether the element is laid out as a block.
pub fn is_block(name: &str) -> bool {
    BLOCK.contains(&name)
}

/// Whether the parser strips one leading newline after this element's open tag.
pub fn strips_leading_newline(name: &str) -> bool {
    NEWLINE_STRIPPED.contains(&name)
}

/// How this element's children are serialized.
pub fn content_mode(name: &str) -> ContentMode {
    if is_void(name) {
        ContentMode::Void
    } else if RAW.contains(&name) {
        ContentMode::Raw
    } else if PRESERVED.contains(&name) {
        ContentMode::Preserved
    } else {
        ContentMode::Laid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A void element cannot have content to lay out, so the two answers must not
    /// disagree.
    #[test]
    fn every_void_element_reports_void_content() {
        for name in VOID {
            assert_eq!(
                content_mode(name),
                ContentMode::Void,
                "{name} is void but its content mode says otherwise"
            );
        }
    }

    /// The three preserved elements are the three the projection specification
    /// names in section 6.2. A fourth would mean the formatter preserves content
    /// the projection normalizes.
    #[test]
    fn preserved_elements_are_the_three_the_projection_names() {
        let mut preserved: Vec<&str> = PRESERVED.to_vec();
        preserved.sort_unstable();
        assert_eq!(preserved, ["code", "pre", "textarea"]);
    }

    /// Raw content is never escaped, so a raw element must not also be preserved:
    /// the two rules would prescribe different bytes for the same character.
    #[test]
    fn no_element_is_both_raw_and_preserved() {
        for name in RAW {
            assert!(
                !PRESERVED.contains(name),
                "{name} is both raw and preserved"
            );
        }
    }

    #[test]
    fn an_unknown_element_is_inline_and_laid_out() {
        assert!(!is_block("custom-widget"));
        assert!(!is_void("custom-widget"));
        assert_eq!(content_mode("custom-widget"), ContentMode::Laid);
    }

    /// `code` is inline, because it appears inside a sentence. `pre` is a block.
    /// Both preserve their content, which is why the two questions are separate.
    #[test]
    fn code_is_inline_and_pre_is_a_block_and_both_preserve() {
        assert!(!is_block("code"));
        assert!(is_block("pre"));
        assert_eq!(content_mode("code"), ContentMode::Preserved);
        assert_eq!(content_mode("pre"), ContentMode::Preserved);
    }

    /// The lists are sorted so that a name is easy to find and a duplicate is
    /// visible in review rather than silently harmless.
    #[test]
    fn the_lists_are_sorted_and_free_of_duplicates() {
        for (label, list) in [("VOID", VOID), ("BLOCK", BLOCK), ("PRESERVED", PRESERVED)] {
            let mut sorted: Vec<&str> = list.to_vec();
            sorted.sort_unstable();
            assert_eq!(sorted, list.to_vec(), "{label} is not sorted");
            sorted.dedup();
            assert_eq!(sorted.len(), list.len(), "{label} has a duplicate");
        }
    }
}
