//! What a terminal shows: the characters it would not show as themselves, and text written so that
//! each of them is named by its code point instead of acted on.
//!
//! Text a document holds reaches an operator's terminal in reports: a parse recovery's message,
//! which quotes the character the parser stopped at, and, in the command-line program, a runbook's
//! name and title in a listing and a declaration's field, tool, path, and remedy in a preflight
//! report. Any of it can hold a character a terminal acts on, an escape sequence's opening, or one
//! that changes how the line is drawn or is drawn as nothing, so the line reads one way and holds
//! another. One rule, [`hidden`], says which characters those are, so every report that names them
//! and every rule that refuses them read one set. It lives here, in the crate every other crate
//! depends on, so a recovered parse error is written safely where it is made.

/// Each character of `text`, with whether a terminal would not show it as itself where it stands.
///
/// `hides_itself` decides for every character but the two emoji presentation selectors, U+FE0E
/// and U+FE0F. Those choose text or emoji presentation for the character before them, so one is
/// shown as part of a visible character it directly follows, and hidden anywhere else: at the
/// start, after a space or another hidden character, or after another selector.
pub fn hidden(text: &str) -> impl Iterator<Item = (char, bool)> + '_ {
    let mut after_visible = false;
    text.chars().map(move |character| {
        let selects = matches!(character, '\u{FE0E}' | '\u{FE0F}');
        let hides = if selects {
            !after_visible
        } else {
            hides_itself(character)
        };
        // A joiner is kept but draws nothing of its own, so a selector after one has nothing to
        // select for, as after a space.
        after_visible = !hides && !selects && !matches!(character, ' ' | '\u{200C}' | '\u{200D}');
        (character, hides)
    })
}

/// The first character of `text` a terminal would not show as itself where it stands, if any.
pub fn first_hidden(text: &str) -> Option<char> {
    hidden(text)
        .find(|(_, hides)| *hides)
        .map(|(character, _)| character)
}

/// Whether a terminal would not show `character` as itself, wherever it stands.
///
/// Four kinds:
///
/// - **a control character**, which a terminal acts on, an escape sequence's opening included;
/// - **whitespace other than the plain space**: a tab, a line or paragraph separator, or a space of
///   another width, a non-breaking one included, any of which breaks the one line or hides where a
///   word ends;
/// - **an invisible character**, [`is_invisible`];
/// - **the braille pattern blank**, U+2800, which a terminal draws as a blank space although
///   Unicode gives it neither property [`is_invisible`] reads: it is a printable symbol with no dots,
///   so a line holding it reads as holding a space it does not hold.
///
/// The two emoji presentation selectors are among the invisible characters here. [`hidden`] shows
/// one that directly follows a visible character, which is the only place it means anything.
pub(crate) fn hides_itself(character: char) -> bool {
    character.is_control()
        || (character.is_whitespace() && character != ' ')
        || is_invisible(character)
        // Drawn as a blank, though it is neither whitespace nor default-ignorable.
        || character == '\u{2800}'
}

/// Whether `character` is drawn as nothing, or changes how the text around it is drawn, other than
/// the two joiners.
///
/// The union of two Unicode 16.0 sets:
///
/// - **the format category, Cf**: the zero-width space and word joiner, the byte-order mark, the
///   soft hyphen, the bidirectional marks, embeddings, overrides, and isolates, interlinear
///   annotation, the prepended concatenation marks, and the tag characters, which spell hidden
///   ASCII;
/// - **Default_Ignorable_Code_Point**, which a renderer draws as nothing when it has no glyph: the
///   combining grapheme joiner, the Hangul fillers, the Khmer inherent vowels, the Mongolian free
///   variation selectors, every variation selector, and the reserved ignorable ranges.
///
/// So a line holding one can read one way and hold another.
///
/// The zero-width non-joiner and joiner, U+200C and U+200D, are kept, because scripts such as
/// Persian and Devanagari and every emoji sequence need them, and they change how neighbouring
/// characters join rather than what the line says.
///
/// The table is stated here because the standard library does not expose Unicode properties, and a
/// dependency for two properties would be a crate the notices must then credit.
const fn is_invisible(character: char) -> bool {
    matches!(
        character,
        '\u{00AD}'
            | '\u{034F}'
            | '\u{0600}'..='\u{0605}'
            | '\u{061C}'
            | '\u{06DD}'
            | '\u{070F}'
            | '\u{0890}'..='\u{0891}'
            | '\u{08E2}'
            | '\u{115F}'..='\u{1160}'
            | '\u{17B4}'..='\u{17B5}'
            | '\u{180B}'..='\u{180F}'
            | '\u{200B}'
            | '\u{200E}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{3164}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FEFF}'
            | '\u{FFA0}'
            | '\u{FFF0}'..='\u{FFFB}'
            | '\u{110BD}'
            | '\u{110CD}'
            | '\u{13430}'..='\u{1343F}'
            | '\u{1BCA0}'..='\u{1BCA3}'
            | '\u{1D173}'..='\u{1D17A}'
            | '\u{E0000}'..='\u{E0FFF}'
    )
}

/// `text` as a report may show it: each character a terminal would not show as itself is written
/// as its code point, `<U+001B>`, and every other character as it is.
///
/// For text a document holds that a report names: a parse recovery's message, which quotes the
/// character the parser stopped at, and, in the command-line program, a declaration's field name,
/// tool, or path, so a document cannot write an escape sequence into the report through it.
pub fn shown(text: &str) -> String {
    hidden(text)
        .map(|(character, hides)| {
            if hides {
                format!("<U+{:04X}>", u32::from(character))
            } else {
                character.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{first_hidden, hides_itself, shown};

    /// The bidirectional formatting characters, each of which reorders what a terminal draws around
    /// it: U+202E makes `txt.exe` read as `exe.txt`.
    const BIDIRECTIONAL: [char; 12] = [
        '\u{061C}', '\u{200E}', '\u{200F}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}',
        '\u{202E}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
    ];

    /// Every bidirectional formatting character hides itself, and is named instead of passed
    /// through.
    #[test]
    fn every_bidirectional_formatting_character_is_named() {
        for character in BIDIRECTIONAL {
            assert!(hides_itself(character), "U+{:04X}", u32::from(character));
            assert!(
                !shown(&format!("a{character}b")).contains(character),
                "U+{:04X}",
                u32::from(character)
            );
        }
    }

    /// A control character is named by its code point, and text a terminal shows as itself passes
    /// unchanged.
    #[test]
    fn a_control_character_is_named_and_visible_text_passes() {
        assert_eq!(shown("x\u{1b}[2J"), "x<U+001B>[2J");
        assert_eq!(shown("tab\there"), "tab<U+0009>here");
        let text = "deploy the release, \u{0645}\u{200C}\u{06CC} and \u{1F469}\u{200D}\u{1F4BB}";
        assert_eq!(shown(text), text);
        assert_eq!(first_hidden(text), None);
    }
}
