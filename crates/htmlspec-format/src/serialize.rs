//! The canonical serializer.
//!
//! One decision governs the whole module: **text is never rewrapped.** Author line
//! breaks inside text-bearing content are preserved exactly. The serializer
//! normalizes the indentation and line breaking of *block structure*, fixes
//! attribute order and quoting, and prescribes one escaping form per character. It
//! has no opinion about where a paragraph's lines break.
//!
//! That is what makes the fixed-point property nearly structural rather than
//! something to be asserted and hoped for. General HTML formatters that rewrap
//! text are notoriously hard to make idempotent, because inline whitespace is
//! semantic and every wrap decision interacts with the next. Refusing to rewrap
//! removes the interaction.
//!
//! It also delivers the absence of formatter-induced churn: editing a sentence
//! cannot make the formatter rewrite the surrounding paragraph. It does *not*
//! deliver a minimal Git diff — a paragraph held on one long line diffs as a whole
//! line when one sentence changes — and that cost belongs to the line structure
//! rather than to the formatter.
//!
//! The layout rule, stated once: a block element opens on its own line, its
//! children are indented one level, and it closes on its own line. Everything else
//! — inline elements, text, and the contents of a block whose children are all
//! inline — is written in place, with author line breaks intact.

use kuchikiki::{NodeData, NodeRef};

use crate::attributes;
use crate::element::{self, ContentMode};

/// The prescribed indentation of one nesting level.
///
/// Two spaces, and it is not configurable. Every knob added here becomes a
/// compatibility obligation, and a document formatted one way on one machine and
/// another way on another is the exact failure canonical serialization exists to
/// prevent.
const INDENT: &str = "  ";

/// The prescribed line ending.
///
/// A single form, chosen once, so that contributors on different operating
/// systems produce byte-identical output from the same document.
pub const LINE_ENDING: &str = "\n";

/// The prescribed doctype, written for every document.
///
/// The short form, which is what keeps a browser out of quirks mode. It is
/// written unconditionally rather than copied from the source: a document's
/// doctype is not authored content, and copying a legacy one would preserve a
/// defect the parser already reports.
const DOCTYPE: &str = "<!DOCTYPE html>";

/// Serializes a parsed document into its canonical form.
///
/// The result ends with exactly one newline and carries no trailing whitespace on
/// any line, so it satisfies the file-level normalization requirement by
/// construction rather than by a cleanup pass.
pub fn document(node: &NodeRef) -> String {
    let mut out = String::new();
    out.push_str(DOCTYPE);
    out.push_str(LINE_ENDING);

    for child in node.children() {
        match child.data() {
            // The doctype is written above, from the prescription rather than
            // from the source.
            NodeData::Doctype(_) => {}
            _ => write_block_child(&child, 0, &mut out),
        }
    }

    // Every line-emitting path below appends its own line ending, so the buffer
    // already ends with exactly one. An empty document still gets the doctype's.
    debug_assert!(out.ends_with(LINE_ENDING));
    out
}

/// Writes one child of a block context: on its own line, at `depth`.
fn write_block_child(node: &NodeRef, depth: usize, out: &mut String) {
    match node.data() {
        NodeData::Element(_) => write_element(node, depth, out),
        NodeData::Comment(comment) => {
            push_indent(depth, out);
            out.push_str("<!--");
            out.push_str(&comment.borrow());
            out.push_str("-->");
            out.push_str(LINE_ENDING);
        }
        NodeData::Text(text) => {
            // Whitespace between blocks is layout the serializer owns, so it is
            // dropped: it carries no content and keeping it would mean the source's
            // indentation survived into the canonical form.
            let text = text.borrow();
            if !text.trim().is_empty() {
                write_inline_line(node, depth, out);
            }
        }
        // A processing instruction or a second doctype cannot appear in a parsed
        // HTML document, and a document fragment is not produced by this parser.
        _ => {}
    }
}

/// Writes one element, choosing its layout from its name.
fn write_element(node: &NodeRef, depth: usize, out: &mut String) {
    let Some(element) = node.as_element() else {
        return;
    };
    let name = element.name.local.to_string();
    let open = open_tag(node, &name);

    match element::content_mode(&name) {
        ContentMode::Void => {
            push_indent(depth, out);
            out.push_str(&open);
            out.push_str(LINE_ENDING);
        }
        // Preserved and raw content are written between the tags with nothing
        // added, because a byte of indentation inserted here would be content.
        ContentMode::Preserved | ContentMode::Raw => {
            push_indent(depth, out);
            out.push_str(&open);
            write_verbatim_content(node, &name, element::content_mode(&name), out);
            out.push_str(&close_tag(&name));
            out.push_str(LINE_ENDING);
        }
        ContentMode::Laid => {
            let children: Vec<NodeRef> = significant_children(node);
            if children.iter().any(is_block_level) {
                push_indent(depth, out);
                out.push_str(&open);
                out.push_str(LINE_ENDING);
                for child in &children {
                    write_block_child(child, depth + 1, out);
                }
                push_indent(depth, out);
                out.push_str(&close_tag(&name));
                out.push_str(LINE_ENDING);
            } else {
                // No block child, so the element and its content occupy one
                // logical line — which may still span several physical lines if
                // the author put newlines in the text. Those are preserved.
                push_indent(depth, out);
                out.push_str(&open);
                for child in &children {
                    write_inline(child, out);
                }
                out.push_str(&close_tag(&name));
                out.push_str(LINE_ENDING);
            }
        }
    }
}

/// Writes text or an inline element that begins its own line.
fn write_inline_line(node: &NodeRef, depth: usize, out: &mut String) {
    push_indent(depth, out);
    let mut written = String::new();
    write_inline(node, &mut written);
    // Author line breaks survive; the leading and trailing whitespace of the run
    // is layout, and the indent above replaces it.
    out.push_str(written.trim());
    out.push_str(LINE_ENDING);
}

/// Writes a node inline, adding no whitespace of its own.
fn write_inline(node: &NodeRef, out: &mut String) {
    match node.data() {
        NodeData::Text(text) => out.push_str(&crate::escape::text(&text.borrow())),
        NodeData::Comment(comment) => {
            out.push_str("<!--");
            out.push_str(&comment.borrow());
            out.push_str("-->");
        }
        NodeData::Element(_) => {
            let Some(element) = node.as_element() else {
                return;
            };
            let name = element.name.local.to_string();
            out.push_str(&open_tag(node, &name));
            match element::content_mode(&name) {
                ContentMode::Void => {}
                ContentMode::Preserved | ContentMode::Raw => {
                    write_verbatim_content(node, &name, element::content_mode(&name), out);
                    out.push_str(&close_tag(&name));
                }
                ContentMode::Laid => {
                    for child in node.children() {
                        write_inline(&child, out);
                    }
                    out.push_str(&close_tag(&name));
                }
            }
        }
        _ => {}
    }
}

/// Writes the content of a preserved or raw element, byte for byte.
///
/// A comment inside preserved text is retained as a comment, per projection
/// specification section 6.4, so text split by a comment survives formatting as
/// the same three nodes.
fn write_verbatim_content(node: &NodeRef, name: &str, mode: ContentMode, out: &mut String) {
    let mut content = String::new();
    for child in node.children() {
        match child.data() {
            NodeData::Text(text) => {
                let text = text.borrow();
                match mode {
                    // Raw text is not HTML and is never escaped. It cannot be:
                    // `&` inside a script is an operator, not an entity.
                    ContentMode::Raw => content.push_str(&text),
                    _ => content.push_str(&crate::escape::text(&text)),
                }
            }
            NodeData::Comment(comment) => {
                content.push_str("<!--");
                content.push_str(&comment.borrow());
                content.push_str("-->");
            }
            // An element *inside* preserved content. `<pre><code>` is the case
            // that matters, and it is the conventional way to mark up a code
            // excerpt, so it is written inline with no layout added: a newline
            // or an indent inserted here would become content. Writing nothing
            // instead would silently discard the excerpt.
            //
            // Raw content cannot reach this arm — the tokenizer produces no
            // element inside `script` or `style` — so this is the preserved case
            // only, and `write_inline` escapes text as preserved content requires.
            NodeData::Element(_) => write_inline(&child, &mut content),
            _ => {}
        }
    }

    // The parser discards one newline immediately after `<pre>` and `<textarea>`.
    // Writing content that begins with a newline would therefore lose it on the
    // next parse, and the formatter would not be a fixed point. Doubling it here
    // means the next parse strips one and keeps the author's.
    if element::strips_leading_newline(name) && content.starts_with('\n') {
        out.push('\n');
    }
    out.push_str(&content);
}

/// The open tag, with attributes in prescribed order and values double-quoted.
///
/// Every attribute is quoted, including one whose value would be legal unquoted.
/// A rule that quoted only when necessary would make the canonical form depend on
/// the value, and a value edit would then move the quotes.
fn open_tag(node: &NodeRef, name: &str) -> String {
    let mut tag = String::from("<");
    tag.push_str(name);
    if let Some(element) = node.as_element() {
        for attribute in attributes::ordered(&element.attributes.borrow()) {
            tag.push(' ');
            tag.push_str(&attribute.name);
            tag.push_str("=\"");
            tag.push_str(&crate::escape::attribute_value(&attribute.value));
            tag.push('"');
        }
    }
    // No self-closing slash on a void element. The HTML parser ignores it, so it
    // is decoration, and one form has to be chosen.
    tag.push('>');
    tag
}

fn close_tag(name: &str) -> String {
    format!("</{name}>")
}

/// An element's children, with whitespace that is layout rather than content
/// dropped.
///
/// This is where source indentation stops being content, and the line is drawn in
/// two places rather than one.
///
/// Whitespace *between blocks* is layout: the serializer owns where a block sits,
/// so the author's indentation of it is discarded. Whitespace that is an element's
/// *entire* content is layout too — a `<body>` holding one newline holds nothing —
/// and this is the case that decides idempotence. Keeping it would write
/// `<body>\n</body>`, whose reparse holds that newline plus the one the serializer
/// wrote after the closing tag, and the element would grow a line on every pass.
///
/// Whitespace between *inline* content is content and is kept, because the space
/// in `a <em>b</em>` is one a reader sees.
fn significant_children(node: &NodeRef) -> Vec<NodeRef> {
    let children: Vec<NodeRef> = node.children().collect();
    let has_block = children.iter().any(is_block_level);
    let is_all_whitespace = children.iter().all(|child| match child.data() {
        NodeData::Text(text) => text.borrow().trim().is_empty(),
        _ => false,
    });

    if !has_block && !is_all_whitespace {
        return children;
    }
    children
        .into_iter()
        .filter(|child| match child.data() {
            NodeData::Text(text) => !text.borrow().trim().is_empty(),
            _ => true,
        })
        .collect()
}

/// Whether a node forces its parent to lay children out as blocks.
fn is_block_level(node: &NodeRef) -> bool {
    node.as_element()
        .is_some_and(|element| element::is_block(&element.name.local))
}

fn push_indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str(INDENT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use htmlspec_parse::parse_document;

    fn format(html: &str) -> String {
        document(&parse_document(html).document)
    }

    const MINIMAL: &str = "<!doctype html>\n<html lang=\"en\"><head>\
         <meta name=\"htmlspec-version\" content=\"0.1.0\">\
         <meta name=\"htmlspec-kind\" content=\"specification\">\
         <title>T</title></head><body><p>Body.</p></body></html>\n";

    #[test]
    fn a_block_opens_and_closes_on_its_own_line_with_children_indented() {
        let formatted = format("<!doctype html><html lang=\"en\"><head><title>T</title></head><body><div><p>Text.</p></div></body></html>");
        assert!(
            formatted.contains("  <body>\n    <div>\n      <p>Text.</p>\n    </div>\n  </body>"),
            "unexpected layout:\n{formatted}"
        );
    }

    #[test]
    fn the_doctype_is_written_in_one_prescribed_form() {
        for source in [
            "<!doctype html>",
            "<!DOCTYPE HTML>",
            "<!DoCtYpE   html>",
            // A document with no doctype at all: the parser recovers, validation
            // rejects it, and the formatter still writes the prescribed form
            // rather than inventing a second variant.
            "",
        ] {
            let formatted = format(&format!(
                "{source}<html lang=\"en\"><head><title>T</title></head><body></body></html>"
            ));
            assert!(
                formatted.starts_with("<!DOCTYPE html>\n"),
                "{source:?} produced {:?}",
                formatted.lines().next()
            );
        }
    }

    /// The load-bearing property: the formatter has no opinion about where a
    /// paragraph's lines break.
    #[test]
    fn author_line_breaks_inside_text_are_preserved_exactly() {
        let paragraph = "<p>One sentence.\nA second on its own line.\n  A third, indented.</p>";
        let formatted = format(&format!(
            "<!doctype html><html lang=\"en\"><head><title>T</title></head><body>{paragraph}</body></html>"
        ));
        assert!(
            formatted
                .contains("<p>One sentence.\nA second on its own line.\n  A third, indented.</p>"),
            "the formatter rewrapped text:\n{formatted}"
        );
    }

    /// Preserved content survives byte for byte, indentation included.
    #[test]
    fn preserved_content_survives_byte_for_byte() {
        let source = "fn main() {\n    let x = 1;\n        // deeper\n}\n";
        let formatted = format(&format!(
            "<!doctype html><html lang=\"en\"><head><title>T</title></head>\
             <body><figure><figcaption>C</figcaption><pre>{source}</pre></figure></body></html>"
        ));
        assert!(
            formatted.contains(&format!("<pre>{source}</pre>")),
            "preserved content changed:\n{formatted}"
        );
    }

    /// `<pre>` whose *content* begins with a newline is the case that breaks
    /// idempotence if the extra newline is not written.
    ///
    /// The parser discards one newline immediately after `<pre>`, so the source
    /// `<pre>\n\nfirst` holds the content `\nfirst`. Writing that back verbatim
    /// would lose the newline on the next parse and the content would shrink on
    /// every pass. The serializer writes the extra newline so the round trip
    /// holds.
    #[test]
    fn preserved_content_beginning_with_a_newline_round_trips() {
        let html = "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
             <body><pre>\n\nfirst line\n</pre></body></html>\n";
        let content_of = |source: &str| {
            parse_document(source)
                .document
                .select_first("pre")
                .expect("the pre parsed")
                .as_node()
                .text_contents()
        };
        assert_eq!(
            content_of(html),
            "\nfirst line\n",
            "the parser did not strip the first newline as expected"
        );

        let once = format(html);
        let twice = format(&once);
        assert_eq!(once, twice, "a leading newline in <pre> was not stable");
        assert_eq!(
            content_of(&once),
            "\nfirst line\n",
            "the leading newline was lost in the round trip"
        );
    }

    /// The `<pre><code>` arrangement is the conventional way to mark up a code
    /// excerpt, and dropping the inner element would silently discard it.
    #[test]
    fn a_code_element_inside_pre_survives_with_its_content() {
        let excerpt =
            "&lt;section class=\"requirement\"&gt;\n  &lt;h2&gt;Name&lt;/h2&gt;\n&lt;/section&gt;";
        let html = format!(
            "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
             <body><pre><code>{excerpt}</code></pre></body></html>\n"
        );
        let formatted = format(&html);
        assert!(
            formatted.contains(&format!("<pre><code>{excerpt}</code></pre>")),
            "the code excerpt did not survive:\n{formatted}"
        );
        assert_eq!(formatted, format(&formatted));
    }

    /// Source indentation between blocks is layout, and the serializer owns it.
    #[test]
    fn source_indentation_between_blocks_is_replaced() {
        let sprawling = "<!doctype html>\n<html lang=\"en\">\n        <head>\n\n\n  <title>T</title>\n   </head>\n  <body>\n\n      <p>Text.</p>\n  </body>\n</html>\n";
        let tight = "<!doctype html><html lang=\"en\"><head><title>T</title></head><body><p>Text.</p></body></html>";
        assert_eq!(
            format(sprawling),
            format(tight),
            "two indentations of one document did not converge"
        );
    }

    /// The specs' own scenario: two agents that differ only in attribute order
    /// and indentation converge on one file.
    #[test]
    fn two_authoring_styles_converge_on_one_file() {
        let first = "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head><body>\
             <section class=\"requirement\" id=\"req-1\" data-change=\"added\">\
             <h2>Name</h2><p>The system SHALL work.</p></section></body></html>\n";
        let second = "<!DOCTYPE HTML>\n<html lang='en'>\n  <head>\n    <title>T</title>\n  </head>\n  <body>\n      <section data-change=\"added\" id=\"req-1\" class=\"requirement\">\n        <h2>Name</h2>\n        <p>The system SHALL work.</p>\n      </section>\n  </body>\n</html>\n";
        assert_eq!(format(first), format(second));
    }

    #[test]
    fn every_attribute_value_is_double_quoted() {
        let formatted = format(
            "<!doctype html><html lang=en><head><title>T</title></head>\
             <body><p id=x class='y'>T</p></body></html>",
        );
        assert!(formatted.contains(r#"<html lang="en">"#), "{formatted}");
        assert!(
            formatted.contains(r#"<p class="y" id="x">T</p>"#),
            "{formatted}"
        );
    }

    #[test]
    fn a_void_element_has_no_closing_tag_and_no_self_closing_slash() {
        let formatted = format(MINIMAL);
        // `name` and `content` are both attributes the projection reads through
        // the metadata rules rather than contract attributes of a unit, so they
        // take the alphabetical tail of the prescribed order.
        assert!(
            formatted.contains(r#"<meta content="0.1.0" name="htmlspec-version">"#),
            "{formatted}"
        );
        assert!(!formatted.contains("</meta>"), "{formatted}");
        assert!(!formatted.contains("/>"), "{formatted}");
    }

    /// An inline element stays on the line, because breaking it would insert
    /// whitespace a reader can see.
    #[test]
    fn inline_elements_stay_on_the_line() {
        let formatted = format(
            "<!doctype html><html lang=\"en\"><head><title>T</title></head>\
             <body><p>A <strong>bold</strong> and <em>italic</em> word.</p></body></html>",
        );
        assert!(
            formatted.contains("<p>A <strong>bold</strong> and <em>italic</em> word.</p>"),
            "{formatted}"
        );
    }

    /// A comment inside preserved text is retained as a comment, and the text
    /// around it survives, so the three nodes reparse as three nodes. This is the
    /// arrangement the executable-content rule must accept.
    #[test]
    fn a_comment_inside_preserved_text_is_retained() {
        let html = "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
             <body><pre>before<!-- split -->after</pre></body></html>\n";
        let formatted = format(html);
        assert!(
            formatted.contains("<pre>before<!-- split -->after</pre>"),
            "{formatted}"
        );
        assert_eq!(formatted, format(&formatted));
    }

    /// A comment between blocks keeps its own line, so a reviewer's note about a
    /// unit does not end up appended to it.
    #[test]
    fn a_comment_between_blocks_keeps_its_own_line() {
        let formatted = format(
            "<!doctype html><html lang=\"en\"><head><title>T</title></head>\
             <body><!-- a note --><p>Text.</p></body></html>",
        );
        assert!(
            formatted.contains("  <body>\n    <!-- a note -->\n    <p>Text.</p>\n  </body>"),
            "{formatted}"
        );
    }

    /// Raw text is never escaped: `&&` in a script is an operator.
    #[test]
    fn raw_text_is_not_escaped() {
        let html = "<!doctype html>\n<html lang=\"en\"><head><title>T</title>\
             <script>if (a && b < c) { go(); }</script></head><body></body></html>\n";
        let formatted = format(html);
        assert!(
            formatted.contains("<script>if (a && b < c) { go(); }</script>"),
            "{formatted}"
        );
        assert_eq!(formatted, format(&formatted));
    }

    #[test]
    fn the_output_ends_with_exactly_one_newline_and_has_no_trailing_whitespace() {
        let formatted = format(MINIMAL);
        assert!(formatted.ends_with('\n'));
        assert!(!formatted.ends_with("\n\n"));
        for (number, line) in formatted.lines().enumerate() {
            // A line inside preserved content may legitimately end in a space,
            // and this sample has none, so every line is held to the rule.
            assert_eq!(
                line.trim_end(),
                line,
                "line {} has trailing whitespace",
                number + 1
            );
        }
    }

    #[test]
    fn a_document_with_a_unit_lays_out_as_expected() {
        let formatted = format(
            "<!doctype html>\n<html lang=\"en\"><head>\
             <meta name=\"htmlspec-version\" content=\"0.1.0\">\
             <meta name=\"htmlspec-kind\" content=\"specification\">\
             <title>T</title></head><body>\
             <section id=\"req-1\" class=\"requirement\"><h2>Name</h2>\
             <p>The system SHALL work.</p></section></body></html>\n",
        );
        assert_eq!(
            formatted,
            "<!DOCTYPE html>\n\
             <html lang=\"en\">\n\
             \x20 <head>\n\
             \x20   <meta content=\"0.1.0\" name=\"htmlspec-version\">\n\
             \x20   <meta content=\"specification\" name=\"htmlspec-kind\">\n\
             \x20   <title>T</title>\n\
             \x20 </head>\n\
             \x20 <body>\n\
             \x20   <section class=\"requirement\" id=\"req-1\">\n\
             \x20     <h2>Name</h2>\n\
             \x20     <p>The system SHALL work.</p>\n\
             \x20   </section>\n\
             \x20 </body>\n\
             </html>\n"
        );
    }
}
