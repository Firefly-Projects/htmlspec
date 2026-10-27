//! What a terminal shows, for the command-line program's reports.
//!
//! The rule for which characters a terminal would not show as themselves lives in
//! [`htmlspec_parse::visible`], so a parse recovery's message, which that crate writes, a runbook's
//! name and title in a listing, a declaration's field, tool, path, and remedy in a preflight report,
//! and the rules that refuse such characters in a remedy and a new runbook's name all read one set.
//! This module re-exports that rule and adds the listing's own rendering. The change's design,
//! decision 11 and decision 18.

pub(crate) use htmlspec_parse::visible::{first_hidden, hidden, shown};

/// `text` as a listing may show it: each character a terminal would not show as itself is written
/// as its Rust escape, `\u{1b}`, and every other character as it is.
///
/// For a runbook's name and title in `list --runbooks` and a bare `run`. Either is whatever the
/// project's directory is called or its document says, and either can hold an escape sequence that
/// recolors or rewrites the terminal it is printed to. The payload carries the text as it is,
/// because JSON escapes it.
pub(crate) fn printable(text: &str) -> String {
    hidden(text)
        .map(|(character, hides)| {
            if hides {
                character.escape_default().to_string()
            } else {
                character.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{first_hidden, printable, shown};

    /// The bidirectional formatting characters, each of which reorders what a terminal draws around
    /// it: U+202E makes `txt.exe` read as `exe.txt`.
    const BIDIRECTIONAL: [char; 12] = [
        '\u{061C}', '\u{200E}', '\u{200F}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}',
        '\u{202E}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
    ];

    /// One of each kind of default-ignorable character beyond the format category: the combining
    /// grapheme joiner, the Hangul fillers, the Khmer inherent vowels, a Mongolian free variation
    /// selector, the variation selectors of both blocks, and a reserved ignorable code point.
    const IGNORABLE: [char; 11] = [
        '\u{034F}',
        '\u{115F}',
        '\u{3164}',
        '\u{FFA0}',
        '\u{17B4}',
        '\u{180B}',
        '\u{FE00}',
        '\u{FE02}',
        '\u{E0100}',
        '\u{E01EF}',
        '\u{FFF0}',
    ];

    /// The one set both the listing and the remedy rule read: every bidirectional formatting
    /// character hides itself, and both renderings name it instead of passing it through.
    #[test]
    fn every_bidirectional_formatting_character_is_named_in_both_renderings() {
        for character in BIDIRECTIONAL {
            assert_eq!(
                first_hidden(&format!("a{character}")),
                Some(character),
                "U+{:04X}",
                u32::from(character)
            );
            let text = format!("a{character}b");
            assert!(
                !printable(&text).contains(character),
                "U+{:04X}",
                u32::from(character)
            );
            assert!(
                !shown(&text).contains(character),
                "U+{:04X}",
                u32::from(character)
            );
        }
    }

    /// Every default-ignorable character hides itself, after a visible character as anywhere else.
    #[test]
    fn every_default_ignorable_character_hides_itself() {
        for character in IGNORABLE {
            assert_eq!(
                first_hidden(&format!("a{character}")),
                Some(character),
                "U+{:04X}",
                u32::from(character)
            );
            assert_eq!(
                first_hidden(&format!("a{character}b")),
                Some(character),
                "U+{:04X}",
                u32::from(character)
            );
        }
    }

    /// A presentation selector directly after a visible character is part of it, and anywhere else
    /// it hides: at the start, after a space, after a joiner, and as a second selector.
    #[test]
    fn a_presentation_selector_shows_only_after_a_visible_character() {
        for shown_as_itself in ["\u{2764}\u{FE0F}", "a\u{FE0E}", "\u{2764}\u{FE0F} ok"] {
            assert_eq!(first_hidden(shown_as_itself), None, "{shown_as_itself:?}");
            assert_eq!(printable(shown_as_itself), shown_as_itself);
        }
        for hides in [
            "\u{FE0F}",
            "a \u{FE0F}",
            "a\u{200D}\u{FE0F}",
            "\u{2764}\u{FE0F}\u{FE0F}",
            "\u{2764}\u{FE0E}\u{FE0F}",
        ] {
            assert!(first_hidden(hides).is_some(), "{hides:?}");
        }
    }

    /// Text a terminal shows as itself passes through both renderings unchanged: the plain space,
    /// the two joiners that scripts and emoji need, and letters beyond ASCII.
    #[test]
    fn text_a_terminal_shows_as_itself_passes_unchanged() {
        let text = "deploy the release, \u{0645}\u{200C}\u{06CC} and \u{1F469}\u{200D}\u{1F4BB}";
        assert_eq!(printable(text), text);
        assert_eq!(shown(text), text);
        assert_eq!(first_hidden(text), None);
    }

    /// The two renderings name a hidden character each in its own form.
    #[test]
    fn each_rendering_names_a_hidden_character_in_its_own_form() {
        assert_eq!(printable("x\u{1b}[2J"), "x\\u{1b}[2J");
        assert_eq!(shown("x\u{1b}[2J"), "x<U+001B>[2J");
        assert_eq!(printable("tab\there"), "tab\\there");
        assert_eq!(shown("tab\there"), "tab<U+0009>here");
        assert_eq!(shown("a\u{00A0}b"), "a<U+00A0>b");
        assert_eq!(shown("a\u{2800}b"), "a<U+2800>b");
        assert_eq!(first_hidden("deploy\u{2800}"), Some('\u{2800}'));
    }
}
