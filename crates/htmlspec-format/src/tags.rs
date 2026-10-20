//! A document's start tags, read from its source as the HTML tokenizer reads them.
//!
//! [`crate::references`] changes one attribute value's bytes and nothing else, so it works on
//! the source rather than on the tree. But the source alone does not say whether `href="…"` is
//! an attribute or an author's words about one. Only the tokenizer's states say that: a `<`
//! opens a tag in text, and opens nothing inside a comment, inside a quoted value, or inside a
//! `<script>`. The formatter once searched the source for `href="` and filled an author's
//! quotation of one, which is what this module exists to prevent.
//!
//! It follows those states far enough to place each start tag's attributes, and no further.
//! It decodes no character references and builds no tree. The tree stays the authority: a test
//! in the validator's asset rules holds this module's answer to the tree's, over the
//! repository's documents and over generated ones.
//!
//! **Where it stops following the parser** is a document the parser recovers errors in. The
//! tree builder then moves or drops elements (an `<img>` inside `<select>` is dropped), and the
//! formatter refuses such a document anyway. Within a document read without error, it follows
//! the parser into foreign content (`<svg>` and `<math>`, where `style` and `template` are
//! ordinary elements, and back out at each integration point the pinned parser has), and
//! through the escape states of a script's text, where `<!--<script>` postpones the end tag.

use std::ops::Range;

/// A start tag the tree would hold, with its attributes.
pub(crate) struct StartTag {
    /// The tag's name, lowercased.
    pub(crate) name: String,
    /// Its attributes in source order. Of a repeated name, only the first, which is the one
    /// the tokenizer keeps.
    pub(crate) attributes: Vec<Attribute>,
}

/// One attribute of a start tag.
pub(crate) struct Attribute {
    /// The attribute's name, lowercased.
    pub(crate) name: String,
    /// Where its value sits in the source, inside any quotes. An attribute written with no
    /// value has the empty string as its value, as the parser reads it, and an empty range.
    pub(crate) value: Range<usize>,
}

/// Elements whose contents are text up to their own end tag.
///
/// `noscript` is among them because the pinned parser runs with scripting enabled.
/// `plaintext` is not: it has no end tag, and the rest of the source is its text.
const RAW_TEXT: &[&str] = &[
    "script", "style", "xmp", "iframe", "noembed", "noframes", "noscript", "title", "textarea",
];

/// SVG's integration points: elements whose children the parser reads as HTML.
const SVG_INTEGRATION_POINTS: &[&str] = &["foreignobject", "desc", "title"];

/// MathML's text integration points. Their children are read as HTML, except `mglyph` and
/// `malignmark`, which stay MathML.
const MATHML_TEXT_INTEGRATION_POINTS: &[&str] = &["mi", "mo", "mn", "ms", "mtext"];

/// The namespace of foreign content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Namespace {
    Svg,
    MathMl,
}

/// How a start tag is read, which depends on the element it is inside.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Reading {
    /// As HTML, where raw-text elements, `template`, and `plaintext` mean what they do.
    Html,
    /// As foreign content, where every element is ordinary.
    Foreign(Namespace),
}

/// An element open in foreign content, or at its edge, kept so that its end tag can be
/// matched and its children read as the parser reads them.
struct OpenElement {
    /// Its name, lowercased, which its end tag repeats.
    name: String,
    namespace: Namespace,
    /// How its children are read: as HTML at an integration point, otherwise as foreign
    /// content.
    children: Reading,
}

/// Every start tag in `source` that the tree would hold, in source order.
///
/// A start tag inside a `<template>` is left out. The parser puts a template's contents in a
/// separate fragment, so the tree a document is judged by does not hold them.
pub(crate) fn start_tags(source: &str) -> Vec<StartTag> {
    let bytes = source.as_bytes();
    let mut tags = Vec::new();
    let mut template_depth: usize = 0;
    let mut open_elements: Vec<OpenElement> = Vec::new();
    let mut at = 0;

    while let Some(open) = position(bytes, at, |byte| byte == b'<') {
        let rest = tail(bytes, open);
        let reading = open_elements
            .last()
            .map_or(Reading::Html, |element| element.children);
        at = if rest.starts_with(b"<!--") {
            comment_end(bytes, open + 4)
        } else if rest.starts_with(b"<![CDATA[") {
            after(bytes, open + 9, b"]]>")
        } else if rest.starts_with(b"<!") || rest.starts_with(b"<?") {
            // A doctype, another markup declaration, or a processing instruction, which the
            // parser reads as a bogus comment. Each ends at the next `>`.
            after(bytes, open + 2, b">")
        } else if rest.starts_with(b"</") {
            match rest.get(2) {
                Some(byte) if byte.is_ascii_alphabetic() => {
                    // An end tag. Its attributes are read so that a `>` inside a quoted value
                    // does not end it, and then discarded, as the tokenizer discards them.
                    let Some(tag) = read_tag(bytes, open + 2) else {
                        break;
                    };
                    close(&mut open_elements, &mut template_depth, reading, &tag.name);
                    tag.end
                }
                // `</>` is dropped.
                Some(b'>') => open + 3,
                Some(_) => after(bytes, open + 2, b">"),
                // `</` at the end of the source is text.
                None => break,
            }
        } else if rest.get(1).is_some_and(u8::is_ascii_alphabetic) {
            // A start tag. One cut off by the end of the source is never emitted, and nothing
            // after it is a tag.
            let Some(tag) = read_tag(bytes, open + 1) else {
                break;
            };
            let Tag {
                name,
                attributes,
                self_closing,
                end,
            } = tag;
            let opened = opens(reading, open_elements.last(), &name, self_closing);
            if template_depth == 0 {
                tags.push(StartTag {
                    name: name.clone(),
                    attributes,
                });
            }
            if let Some(element) = opened {
                open_elements.push(element);
            }
            match reading {
                // In foreign content every element is ordinary.
                Reading::Foreign(_) => end,
                Reading::Html => {
                    if name == "template" {
                        template_depth += 1;
                    }
                    if name == "plaintext" {
                        break;
                    }
                    if RAW_TEXT.contains(&name.as_str()) {
                        // The contents are text, up to the end tag that closes this element.
                        // With no such end tag, the rest of the source is text.
                        let Some(close) = raw_text_end(bytes, end, &name) else {
                            break;
                        };
                        close
                    } else {
                        end
                    }
                }
            }
        } else {
            // A `<` followed by anything else is text.
            open + 1
        };
    }
    tags
}

/// The element a start tag opens in foreign content or at its edge, if it opens one that
/// changes how what follows is read.
///
/// An HTML element is not tracked: only `svg` and `math` change how its children are read. A
/// self-closing tag opens nothing, because in foreign content the parser closes it at once.
fn opens(
    reading: Reading,
    parent: Option<&OpenElement>,
    name: &str,
    self_closing: bool,
) -> Option<OpenElement> {
    if self_closing {
        return None;
    }
    let namespace = match reading {
        Reading::Html => match name {
            "svg" => Namespace::Svg,
            "math" => Namespace::MathMl,
            // Inside a MathML text integration point, `mglyph` and `malignmark` stay MathML.
            "mglyph" | "malignmark"
                if parent.is_some_and(|parent| {
                    parent.namespace == Namespace::MathMl
                        && MATHML_TEXT_INTEGRATION_POINTS.contains(&parent.name.as_str())
                }) =>
            {
                Namespace::MathMl
            }
            _ => return None,
        },
        // In foreign content an element takes its parent's namespace, whatever its name: an
        // `svg` inside MathML is a MathML element. The exception is an `svg` directly inside
        // `annotation-xml`, which the parser reads as HTML would, and so places in SVG.
        Reading::Foreign(namespace) => {
            if name == "svg"
                && parent.is_some_and(|parent| {
                    parent.namespace == Namespace::MathMl && parent.name == "annotation-xml"
                })
            {
                Namespace::Svg
            } else {
                namespace
            }
        }
    };
    let children = if is_integration_point(namespace, name) {
        Reading::Html
    } else {
        Reading::Foreign(namespace)
    };
    Some(OpenElement {
        name: name.to_owned(),
        namespace,
        children,
    })
}

/// Whether the parser reads this foreign element's children as HTML.
///
/// The HTML specification also makes a MathML `annotation-xml` whose `encoding` is
/// `text/html` an integration point. The pinned parser does not: html5ever asks its tree sink,
/// and kuchikiki keeps markup5ever's default answer, which is no. The tree is the authority, so
/// an annotation's children are read as MathML here too.
fn is_integration_point(namespace: Namespace, name: &str) -> bool {
    match namespace {
        Namespace::Svg => SVG_INTEGRATION_POINTS.contains(&name),
        Namespace::MathMl => MATHML_TEXT_INTEGRATION_POINTS.contains(&name),
    }
}

/// Applies an end tag to the open foreign elements and the template depth.
///
/// In foreign content an end tag closes the nearest open element it names. Read as HTML, it
/// closes the integration point it names, and otherwise an HTML element this module does not
/// track, `template` excepted.
fn close(
    open_elements: &mut Vec<OpenElement>,
    template_depth: &mut usize,
    reading: Reading,
    name: &str,
) {
    match reading {
        Reading::Foreign(_) => {
            if let Some(index) = open_elements
                .iter()
                .rposition(|element| element.name == name)
            {
                open_elements.truncate(index);
            }
        }
        Reading::Html => {
            if open_elements
                .last()
                .is_some_and(|element| element.name == name)
            {
                open_elements.pop();
            } else if name == "template" {
                *template_depth = template_depth.saturating_sub(1);
            }
        }
    }
}

/// A tag read from the source, start or end.
struct Tag {
    name: String,
    attributes: Vec<Attribute>,
    /// Whether the tag ends `/>`. Foreign content closes such an element at once; HTML
    /// ignores the flag on all but void elements.
    self_closing: bool,
    /// Just after the tag's closing `>`.
    end: usize,
}

/// Reads the tag whose name begins at `name_start`, following the tokenizer's tag states.
///
/// Returns `None` when the source ends inside the tag, because the tokenizer emits nothing
/// for a tag it never finished.
fn read_tag(bytes: &[u8], name_start: usize) -> Option<Tag> {
    let name_end = position(bytes, name_start, |byte| {
        is_space(byte) || byte == b'/' || byte == b'>'
    })?;
    let name = lowercase(bytes.get(name_start..name_end));
    let mut attributes: Vec<Attribute> = Vec::new();
    let mut at = name_end;

    loop {
        // Before an attribute's name.
        at = skip_space(bytes, at);
        match bytes.get(at)? {
            // The self-closing solidus, or a stray one. Either way, what follows is read as
            // though it began the next attribute.
            b'/' => {
                at += 1;
                if bytes.get(at) == Some(&b'>') {
                    return Some(Tag {
                        name,
                        attributes,
                        self_closing: true,
                        end: at + 1,
                    });
                }
                continue;
            }
            b'>' => {
                return Some(Tag {
                    name,
                    attributes,
                    self_closing: false,
                    end: at + 1,
                })
            }
            _ => {}
        }

        // The attribute's name. A leading `=` belongs to it, as the tokenizer reads it.
        let attribute_start = at;
        if bytes.get(at) == Some(&b'=') {
            at += 1;
        }
        at = position(bytes, at, |byte| {
            is_space(byte) || matches!(byte, b'/' | b'>' | b'=')
        })?;
        let attribute_name = lowercase(bytes.get(attribute_start..at));

        // Its value, if it has one.
        at = skip_space(bytes, at);
        let value = if bytes.get(at) == Some(&b'=') {
            at = skip_space(bytes, at + 1);
            match bytes.get(at)? {
                quote @ (b'"' | b'\'') => {
                    let quote = *quote;
                    let start = at + 1;
                    let end = position(bytes, start, |byte| byte == quote)?;
                    at = end + 1;
                    start..end
                }
                // `name=>`: the value is missing, and the `>` ends the tag.
                b'>' => at..at,
                _ => {
                    let start = at;
                    at = position(bytes, at, |byte| is_space(byte) || byte == b'>')?;
                    start..at
                }
            }
        } else {
            at..at
        };

        if !attributes
            .iter()
            .any(|attribute| attribute.name == attribute_name)
        {
            attributes.push(Attribute {
                name: attribute_name,
                value,
            });
        }
    }
}

/// Just after the comment whose text begins at `start`, which is just after its `<!--`.
///
/// A comment ends at `-->`, or at `--!>`. `<!-->` and `<!--->` are comments that end at once.
fn comment_end(bytes: &[u8], start: usize) -> usize {
    let rest = tail(bytes, start);
    if rest.starts_with(b">") {
        return start + 1;
    }
    if rest.starts_with(b"->") {
        return start + 2;
    }
    let closed = find(bytes, start, b"-->").map(|at| at + 3);
    let bang_closed = find(bytes, start, b"--!>").map(|at| at + 4);
    match (closed, bang_closed) {
        (Some(closed), Some(bang_closed)) => closed.min(bang_closed),
        (Some(end), None) | (None, Some(end)) => end,
        (None, None) => bytes.len(),
    }
}

/// Where the end tag that closes raw-text element `name` begins, searching from `from`.
///
/// Only `</` followed by the element's own name, in any case, and then a space, `/`, or `>`
/// ends its contents. Anything else, a different end tag included, is text. A script's text
/// has escape states besides, which [`script_end`] follows.
fn raw_text_end(bytes: &[u8], from: usize, name: &str) -> Option<usize> {
    if name == "script" {
        return script_end(bytes, from);
    }
    let mut at = from;
    loop {
        let close = find(bytes, at, b"</")?;
        if is_tag_named(bytes, close + 2, name) {
            return Some(close);
        }
        at = close + 2;
    }
}

/// Where a script's end tag begins, following the tokenizer's script data states.
///
/// `<!--` escapes the text. Inside the escape, `<script` begins a double escape, in which
/// `</script` only returns to the escape, and `-->` anywhere in either returns to plain script
/// data. So in `<!--<script></script>…</script>`, the first end tag closes the inner script
/// and the second ends the element.
fn script_end(bytes: &[u8], from: usize) -> Option<usize> {
    /// The tokenizer's script data states, with their dash states folded into `dashes_from`.
    #[derive(Clone, Copy)]
    enum State {
        Data,
        Escaped,
        DoubleEscaped,
    }
    let mut state = State::Data;
    let mut at = from;
    // The first position whose `-` can count toward a closing `-->`. The opening `<!--`'s own
    // dashes count, so `<!-->` closes as soon as it opens.
    let mut dashes_from = from;

    loop {
        match state {
            State::Data => {
                let open = find(bytes, at, b"<")?;
                if tail(bytes, open).starts_with(b"</") && is_tag_named(bytes, open + 2, "script") {
                    return Some(open);
                }
                if tail(bytes, open).starts_with(b"<!--") {
                    state = State::Escaped;
                    dashes_from = open + 2;
                    at = open + 4;
                } else {
                    at = open + 1;
                }
            }
            State::Escaped | State::DoubleEscaped => {
                let next = position(bytes, at, |byte| byte == b'<' || byte == b'>')?;
                let closes_comment = bytes.get(next) == Some(&b'>')
                    && next >= dashes_from + 2
                    && bytes.get(next - 2..next) == Some(b"--".as_slice());
                if closes_comment {
                    state = State::Data;
                    at = next + 1;
                    continue;
                }
                if bytes.get(next) == Some(&b'>') {
                    at = next + 1;
                    continue;
                }
                // A `<`.
                let ends_inner =
                    tail(bytes, next).starts_with(b"</") && is_tag_named(bytes, next + 2, "script");
                let opens_inner = !tail(bytes, next).starts_with(b"</")
                    && is_tag_named(bytes, next + 1, "script");
                match state {
                    State::Escaped if ends_inner => return Some(next),
                    State::Escaped if opens_inner => {
                        state = State::DoubleEscaped;
                        // Past `<script` and the character that delimits it.
                        at = next + 1 + "script".len() + 1;
                        dashes_from = at;
                    }
                    State::DoubleEscaped if ends_inner => {
                        state = State::Escaped;
                        at = next + 2 + "script".len() + 1;
                        dashes_from = at;
                    }
                    _ => {
                        at = next + 1;
                        // The dashes before a `<` do not reach past it.
                        dashes_from = at;
                    }
                }
            }
        }
    }
}

/// Whether a tag name `name` begins at `at`, in any case, followed by what ends a tag name in
/// raw text: a space, `/`, or `>`.
fn is_tag_named(bytes: &[u8], at: usize, name: &str) -> bool {
    let end = at + name.len();
    bytes
        .get(at..end)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name.as_bytes()))
        && bytes
            .get(end)
            .is_some_and(|byte| is_space(*byte) || matches!(byte, b'/' | b'>'))
}

/// The bytes from `at` on, or none past the end.
fn tail(bytes: &[u8], at: usize) -> &[u8] {
    bytes.get(at..).unwrap_or_default()
}

/// The first position at or after `from` whose byte is `wanted`.
fn position(bytes: &[u8], from: usize, wanted: impl Fn(u8) -> bool) -> Option<usize> {
    tail(bytes, from)
        .iter()
        .position(|byte| wanted(*byte))
        .map(|offset| from + offset)
}

/// The first position at or after `from` where `needle` begins.
fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(from);
    }
    tail(bytes, from)
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

/// Just after the first `needle` at or after `from`, or the end of the source.
fn after(bytes: &[u8], from: usize, needle: &[u8]) -> usize {
    find(bytes, from, needle).map_or(bytes.len(), |at| at + needle.len())
}

/// The first position at or after `at` that is not a space, as the tokenizer counts spaces.
fn skip_space(bytes: &[u8], at: usize) -> usize {
    position(bytes, at, |byte| !is_space(byte)).unwrap_or(bytes.len())
}

/// Tab, line feed, form feed, space, and carriage return, which the parser's input
/// preprocessing turns into a line feed.
fn is_space(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\x0C' | b' ' | b'\r')
}

/// A name as the tokenizer reports it, with ASCII letters lowercased.
fn lowercase(bytes: Option<&[u8]>) -> String {
    String::from_utf8_lossy(bytes.unwrap_or_default()).to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name of every start tag found.
    fn names(source: &str) -> Vec<String> {
        start_tags(source).into_iter().map(|tag| tag.name).collect()
    }

    /// Every attribute found, as its tag's name, its own name, and its value's text.
    fn attributes(source: &str) -> Vec<(String, String, String)> {
        start_tags(source)
            .into_iter()
            .flat_map(|tag| {
                tag.attributes
                    .into_iter()
                    .map(move |attribute| {
                        (
                            tag.name.clone(),
                            attribute.name,
                            source
                                .get(attribute.value)
                                .expect("a value range inside the source")
                                .to_owned(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn triple(tag: &str, name: &str, value: &str) -> (String, String, String) {
        (tag.to_owned(), name.to_owned(), value.to_owned())
    }

    #[test]
    fn text_holds_no_tags() {
        assert!(start_tags("href=\"assets/a.css\" and a < b, and <3").is_empty());
    }

    #[test]
    fn a_comment_holds_no_tags_however_it_ends() {
        assert_eq!(names("<!-- <img src=a> --><p>"), ["p"]);
        assert_eq!(names("<!-- <img src=a> --!><p>"), ["p"]);
        // Abruptly closed: each is a whole comment, so what follows is markup.
        assert_eq!(names("<!--><p>"), ["p"]);
        assert_eq!(names("<!---><p>"), ["p"]);
        assert_eq!(names("<!----><p>"), ["p"]);
        // Never closed: the rest of the source is the comment.
        assert!(names("<!-- <img src=a>").is_empty());
    }

    #[test]
    fn a_cdata_section_holds_no_tags_until_it_closes() {
        assert_eq!(
            names("<svg><![CDATA[ a > <img src=a> ]]><use></svg>"),
            ["svg", "use"]
        );
    }

    #[test]
    fn a_doctype_and_a_bogus_comment_hold_no_tags() {
        assert_eq!(names("<!doctype html><?xml <img src=a>?><p>"), ["p"]);
        assert_eq!(names("</ <img src=a>><p>"), ["p"]);
        assert_eq!(names("</><p>"), ["p"]);
    }

    #[test]
    fn values_are_read_in_every_quoting() {
        assert_eq!(
            attributes("<x a=\"1 > 2\" b='3 \" 4' c=5 d e=>"),
            [
                triple("x", "a", "1 > 2"),
                triple("x", "b", "3 \" 4"),
                triple("x", "c", "5"),
                triple("x", "d", ""),
                triple("x", "e", ""),
            ]
        );
    }

    #[test]
    fn spaces_around_the_equals_sign_are_allowed() {
        assert_eq!(
            attributes("<x a = \"1\"\tb\n=\n2>"),
            [triple("x", "a", "1"), triple("x", "b", "2")]
        );
    }

    #[test]
    fn names_are_lowercased() {
        assert_eq!(
            attributes("<LINK HREF=\"A\">"),
            [triple("link", "href", "A")]
        );
    }

    #[test]
    fn the_first_of_a_repeated_attribute_is_kept() {
        assert_eq!(attributes("<x a=1 A=2 a=3>"), [triple("x", "a", "1")]);
    }

    #[test]
    fn a_solidus_between_attributes_is_not_part_of_either() {
        assert_eq!(
            attributes("<x a=\"1\"/b='2'/>"),
            [triple("x", "a", "1"), triple("x", "b", "2")]
        );
    }

    #[test]
    fn a_leading_equals_sign_belongs_to_the_attribute_name() {
        assert_eq!(attributes("<x =a=1>"), [triple("x", "=a", "1")]);
    }

    #[test]
    fn a_quoted_bracket_does_not_end_an_end_tag() {
        assert_eq!(names("</p title=\"a > <img src=a>\"><b>"), ["b"]);
    }

    #[test]
    fn a_raw_text_element_holds_no_tags_until_its_own_end_tag() {
        for element in RAW_TEXT {
            let source =
                format!("<{element}><img src=a></p></{element}x><img src=b></{element}><b>");
            assert_eq!(names(&source), [*element, "b"], "{source}");
        }
    }

    #[test]
    fn a_raw_text_end_tag_is_matched_in_any_case() {
        assert_eq!(names("<script>x</SCRIPT ><b>"), ["script", "b"]);
        assert_eq!(names("<style>x</Style/><b>"), ["style", "b"]);
    }

    #[test]
    fn a_raw_text_element_left_open_holds_the_rest() {
        assert_eq!(names("<script><img src=a>"), ["script"]);
    }

    #[test]
    fn plaintext_holds_the_rest_of_the_source() {
        assert_eq!(names("<plaintext></plaintext><img src=a>"), ["plaintext"]);
    }

    #[test]
    fn template_contents_are_not_in_the_tree() {
        assert_eq!(
            names("<template><img src=a><template><img src=b></template><img src=c></template><b>"),
            ["template", "b"]
        );
    }

    #[test]
    fn a_tag_cut_off_by_the_end_of_the_source_is_not_emitted() {
        assert_eq!(names("<p><img src=a"), ["p"]);
        assert_eq!(names("<p><img src=\"a>"), ["p"]);
        assert_eq!(names("<p><img"), ["p"]);
        // After a whole attribute, before the next one or the `>`.
        assert_eq!(names("<p><img src=\"a\" "), ["p"]);
        assert_eq!(names("<p><img src='a'/"), ["p"]);
    }

    /// Inside `<svg>` and `<math>`, the parser reads `style`, `title`, and `script` as
    /// ordinary elements, so what they hold is markup.
    #[test]
    fn foreign_content_holds_no_raw_text() {
        assert_eq!(
            names("<svg><style><use href=a/></style></svg><b>"),
            ["svg", "style", "use", "b"]
        );
        assert_eq!(
            names("<svg><g><use href=a/></g><script><use href=b/></script></svg>"),
            ["svg", "g", "use", "script", "use"]
        );
        assert_eq!(
            names("<math><mrow><style><mi>x</mi></style></mrow></math>"),
            ["math", "mrow", "style", "mi"]
        );
    }

    /// Inside `<svg>`, a `template` is an ordinary element, and its contents are in the tree.
    #[test]
    fn a_foreign_template_is_an_ordinary_element() {
        assert_eq!(
            names("<svg><template><use href=a/></template></svg><b>"),
            ["svg", "template", "use", "b"]
        );
    }

    /// Foreign content ends with its own end tag. An `<svg/>` is closed as it opens.
    #[test]
    fn html_resumes_after_foreign_content() {
        assert_eq!(
            names("<svg></svg><style><img src=a></style><b>"),
            ["svg", "style", "b"]
        );
        assert_eq!(
            names("<svg/><style><img src=a></style><b>"),
            ["svg", "style", "b"]
        );
        assert_eq!(
            names("<SVG><Style><use href=a/></Style></SVG><style><img src=b></style>"),
            ["svg", "style", "use", "style"]
        );
    }

    /// An integration point holds HTML: SVG's `foreignObject`, `desc`, and `title`, MathML's
    /// text elements, and an `annotation-xml` that declares HTML.
    #[test]
    fn an_integration_point_holds_html() {
        assert_eq!(
            names("<svg><foreignObject><style><img src=a></style></foreignObject></svg>"),
            ["svg", "foreignobject", "style"]
        );
        assert_eq!(
            names("<svg><title><link href=a></title><desc><style><img src=b></style></desc></svg>"),
            ["svg", "title", "link", "desc", "style"]
        );
        assert_eq!(
            names("<svg><title><template><img src=a></template></title></svg>"),
            ["svg", "title", "template"]
        );
        assert_eq!(
            names("<math><mi><style><img src=a></style></mi></math>"),
            ["math", "mi", "style"]
        );
        // The pinned parser makes no `annotation-xml` an integration point, whatever its
        // encoding, so its `style` is an ordinary MathML element.
        assert_eq!(
            names(
                "<math><annotation-xml encoding=\"text/html\"><style><mi>x</mi></style>\
                 </annotation-xml></math>"
            ),
            ["math", "annotation-xml", "style", "mi"]
        );
        // Inside a MathML text integration point, `mglyph` stays MathML, so its `style` is
        // ordinary.
        assert_eq!(
            names("<math><mi><mglyph><style><mi>x</mi></style></mglyph></mi></math>"),
            ["math", "mi", "mglyph", "style", "mi"]
        );
        // And `<svg>` inside an integration point is foreign again.
        assert_eq!(
            names(
                "<svg><foreignObject><svg><style><use href=a/></style></svg></foreignObject></svg>"
            ),
            ["svg", "foreignobject", "svg", "style", "use"]
        );
    }

    /// In foreign content an element takes its parent's namespace whatever its name, except an
    /// `svg` directly inside `annotation-xml`, which is SVG.
    #[test]
    fn a_foreign_element_takes_its_parent_s_namespace() {
        // A MathML `svg`, so its `title` is no integration point and its `style` is ordinary.
        assert_eq!(
            names("<math><mrow><svg><title><style><mi>x</mi></style></title></svg></mrow></math>"),
            ["math", "mrow", "svg", "title", "style", "mi"]
        );
        // An SVG `math`, so its `mi` is no integration point either.
        assert_eq!(
            names("<svg><math><mi><style><use href=a/></style></mi></math></svg>"),
            ["svg", "math", "mi", "style", "use"]
        );
        // An SVG `svg` inside `annotation-xml`, whose `title` holds HTML.
        assert_eq!(
            names(
                "<math><annotation-xml><svg><title><style><use href=a/></style></title></svg>\
                 </annotation-xml></math>"
            ),
            ["math", "annotation-xml", "svg", "title", "style"]
        );
    }

    /// Inside a script, `<!--<script>` postpones the end tag: the next `</script>` closes the
    /// inner one, and only the one after it ends the script.
    #[test]
    fn a_script_s_escaped_inner_script_postpones_its_end() {
        assert_eq!(
            names("<script><!--<script></script><img src=a></script><b>"),
            ["script", "b"]
        );
        assert_eq!(
            names("<SCRIPT><!--<SCRIPT></SCRIPT><img src=a></SCRIPT><b>"),
            ["script", "b"]
        );
    }

    /// The escape ends where the tokenizer says it does, and only `script` opens one.
    #[test]
    fn a_script_s_escape_ends_where_the_tokenizer_ends_it() {
        // An escape with no inner script: the first end tag ends the script.
        assert_eq!(
            names("<script><!-- </script><img src=a>"),
            ["script", "img"]
        );
        // `-->` inside the inner script returns to plain script data.
        assert_eq!(
            names("<script><!--<script>--></script><img src=a>"),
            ["script", "img"]
        );
        // The inner script's end, then `-->`, then the script's own end.
        assert_eq!(
            names("<script><!--<script></script>--></script><img src=a>"),
            ["script", "img"]
        );
        // `<scripts` is not `<script`.
        assert_eq!(
            names("<script><!--<scripts></script><img src=a>"),
            ["script", "img"]
        );
        // `<!-->` and `<!--->` close as they open, with the opener's own dashes.
        assert_eq!(
            names("<script><!--><script></script><img src=a>"),
            ["script", "img"]
        );
        assert_eq!(
            names("<script><!---><script></script><img src=a>"),
            ["script", "img"]
        );
        // A `<!--` closed before the inner script does not escape it.
        assert_eq!(
            names("<script><!-- --><script></script><img src=a>"),
            ["script", "img"]
        );
        // Only a script escapes. A style's `<!--` is text.
        assert_eq!(
            names("<style><!--<style></style><img src=a>"),
            ["style", "img"]
        );
    }

    #[test]
    fn a_value_range_is_on_a_character_boundary_in_any_text() {
        let source = "<p title=\"é → ü\">ü<img src='ā/ē.png'>";
        assert_eq!(
            attributes(source),
            [
                triple("p", "title", "é → ü"),
                triple("img", "src", "ā/ē.png")
            ]
        );
    }

    use proptest::strategy::Strategy;

    proptest::proptest! {
        /// The scanner is total: on any input it returns, and every value range it reports
        /// can be sliced from the source.
        ///
        /// Built from the characters the tag states turn on, mixed with arbitrary text, so
        /// that half-written tags, quotes, and comments are the common case rather than the
        /// rare one.
        #[test]
        fn any_source_is_scanned_without_panic(
            pieces in proptest::collection::vec(
                proptest::prop_oneof![
                    proptest::sample::select(vec![
                        "<", ">", "/", "=", "\"", "'", " ", "\n", "!", "-", "?", "[", "]",
                        "<!--", "-->", "--!>", "<![CDATA[", "]]>", "</", "<a", "<img", "src",
                        "href", "<script>", "</script>", "<template>", "</template>",
                        "<plaintext>", "é", "→",
                    ])
                    .prop_map(str::to_owned),
                    ".{0,4}",
                ],
                0..40,
            )
        ) {
            let source: String = pieces.concat();
            for tag in start_tags(&source) {
                for attribute in tag.attributes {
                    proptest::prop_assert!(
                        source.get(attribute.value.clone()).is_some(),
                        "{:?} is not a slice of {source:?}",
                        attribute.value
                    );
                }
            }
        }
    }
}
