//! One prescribed escaping form per character.
//!
//! The specs require exactly one form for each character that could be written
//! either literally or as an entity, and the choice here is the minimal one: only
//! characters that would otherwise change how the markup parses are escaped, and
//! each is written as its named reference. Everything else is written literally,
//! including every non-ASCII character, which the UTF-8 requirement already makes
//! representable.
//!
//! The reason to escape as little as possible is that escaping is invisible in a
//! rendered page and very visible in a diff. A formatter that wrote `&eacute;` for
//! `é` would make a diff of a French document unreadable to no benefit.
//!
//! Character references are resolved by the parser before the projection sees
//! them (projection specification, section 6.3), so the model cannot distinguish
//! `&amp;` from a literal `&`. That is precisely why the choice belongs here: the
//! formatter decides which form is written back, and it must decide the same way
//! every time or it is not a fixed point.

/// Escapes text content.
///
/// `<` and `&` must be escaped because each begins a construct. `>` is escaped
/// too: it is legal literally, but a lone `>` next to an escaped `<` reads as an
/// asymmetry a reviewer has to think about, and escaping it costs three bytes.
///
/// A quote is *not* escaped in text, because no quote can terminate anything
/// there. Escaping it would be churn with no parsing consequence.
pub fn text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            other => out.push(other),
        }
    }
    out
}

/// Escapes an attribute value, which is always written in double quotes.
///
/// A double quote must be escaped because it would terminate the value. A single
/// quote must not be, because the value is not delimited by one, and escaping it
/// turns an ordinary apostrophe into `&#39;` in every diff.
pub fn attribute_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_markup_characters_are_escaped_in_text() {
        assert_eq!(text("a & b < c > d"), "a &amp; b &lt; c &gt; d");
    }

    /// A quote in prose is an apostrophe or a quotation mark, and escaping it
    /// would make every quoted sentence unreadable in a diff.
    #[test]
    fn quotes_are_left_alone_in_text() {
        assert_eq!(text(r#"it's a "quote""#), r#"it's a "quote""#);
    }

    #[test]
    fn a_double_quote_is_escaped_in_an_attribute_and_a_single_quote_is_not() {
        assert_eq!(
            attribute_value(r#"say "hi" to Anna's cat"#),
            "say &quot;hi&quot; to Anna's cat"
        );
    }

    /// Non-ASCII text is written literally. The file is UTF-8 by requirement, so
    /// an entity would buy nothing and cost legibility.
    #[test]
    fn non_ascii_text_is_written_literally() {
        assert_eq!(text("café — naïve 日本語 🙂"), "café — naïve 日本語 🙂");
        assert_eq!(attribute_value("café 🙂"), "café 🙂");
    }

    /// Escaping is applied once. A literal `&` in the source has already been
    /// resolved from any entity by the parser, so escaping its output again would
    /// double it, and this asserts the function itself does not.
    #[test]
    fn an_already_escaped_sequence_is_escaped_as_the_text_it_is() {
        // The input here is the *text* `&amp;`, which is what a document
        // containing `&amp;amp;` projects. Its one canonical form escapes the
        // ampersand and leaves the rest alone.
        assert_eq!(text("&amp;"), "&amp;amp;");
    }

    /// Every escape this module emits must survive a round trip through the
    /// parser as the character it stood for, or the formatter is not a fixed
    /// point.
    #[test]
    fn every_escape_round_trips_through_the_parser() {
        for raw in [
            "a & b",
            "a < b",
            "a > b",
            r#"a " b"#,
            "a ' b",
            "</code><script>",
            "café 🙂",
        ] {
            let html = format!(
                "<!doctype html>\n<html lang=\"en\"><head><title>T</title></head>\
                 <body><p title=\"{}\">{}</p></body></html>\n",
                attribute_value(raw),
                text(raw)
            );
            let outcome = htmlspec_parse::parse_document(&html);
            assert!(
                outcome.is_recovery_free(),
                "escaping {raw:?} produced markup the parser recovered from: {:#?}",
                outcome.recovered
            );

            let paragraph = outcome
                .document
                .select_first("p")
                .expect("the paragraph parsed");
            assert_eq!(
                paragraph.as_node().text_contents(),
                raw,
                "text escaping of {raw:?} did not round trip"
            );
            let attributes = paragraph.attributes.borrow();
            assert_eq!(
                attributes.get("title"),
                Some(raw),
                "attribute escaping of {raw:?} did not round trip"
            );
        }
    }
}
