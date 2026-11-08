//! A crate's SPDX license expression, and the licenses it is credited under.
//!
//! Precedence is SPDX's: `WITH` binds tightest, then `AND`, then `OR`, and parentheses group. The
//! legacy `/` reads as `OR`. Under `OR` the alternative whose least-preferred license comes
//! earliest in [`License::PREFERENCE`] is credited, which for alternatives of one license each is
//! simply the first of them in that order; under `AND` every part is; and a license `WITH` an
//! exception is credited as that license. An expression that leaves no accepted choice is refused.

use std::collections::BTreeSet;
use std::fmt;

/// A license this project accepts. The order of the variants is the order of preference, in which
/// one is chosen under `OR`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum License {
    Mit,
    Apache2,
    Bsd3Clause,
    Bsd2Clause,
    Isc,
    Zlib,
    Unicode3,
    Mit0,
    Unlicense,
    Mpl2,
}

impl License {
    /// Every accepted license, in the order one is chosen under `OR`.
    pub const PREFERENCE: [Self; 10] = [
        Self::Mit,
        Self::Apache2,
        Self::Bsd3Clause,
        Self::Bsd2Clause,
        Self::Isc,
        Self::Zlib,
        Self::Unicode3,
        Self::Mit0,
        Self::Unlicense,
        Self::Mpl2,
    ];

    /// Its SPDX identifier.
    pub fn spdx(self) -> &'static str {
        match self {
            Self::Mit => "MIT",
            Self::Apache2 => "Apache-2.0",
            Self::Bsd3Clause => "BSD-3-Clause",
            Self::Bsd2Clause => "BSD-2-Clause",
            Self::Isc => "ISC",
            Self::Zlib => "Zlib",
            Self::Unicode3 => "Unicode-3.0",
            Self::Mit0 => "MIT-0",
            Self::Unlicense => "Unlicense",
            Self::Mpl2 => "MPL-2.0",
        }
    }

    /// The accepted license an SPDX identifier names, exactly as SPDX spells it.
    pub fn from_spdx(identifier: &str) -> Option<Self> {
        Self::PREFERENCE
            .into_iter()
            .find(|license| license.spdx() == identifier)
    }

    /// The spellings a `LICENSE-X` or `LICENSE.X` file uses for it, as X. A license with none is
    /// matched only by a single-license crate's `LICENSE`, `LICENCE`, or `COPYING`, and the
    /// Unlicense by an `UNLICENSE` file too.
    pub fn file_keys(self) -> &'static [&'static str] {
        match self {
            Self::Mit => &["MIT"],
            Self::Apache2 => &["APACHE", "APACHE2", "APACHE-2.0"],
            Self::Bsd3Clause | Self::Bsd2Clause => &["BSD"],
            Self::Zlib => &["ZLIB"],
            Self::Unicode3 => &["UNICODE"],
            Self::Isc | Self::Mit0 | Self::Unlicense | Self::Mpl2 => &[],
        }
    }

    /// Whether the license asks that a recipient of an executable be told where its source form
    /// is, as MPL-2.0's section 3.2 does.
    pub fn asks_for_source_form(self) -> bool {
        self == Self::Mpl2
    }

    /// The accepted licenses, joined as a sentence lists them.
    pub fn accepted() -> String {
        Self::PREFERENCE
            .into_iter()
            .map(Self::spdx)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl fmt::Display for License {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spdx())
    }
}

/// A license identifier an expression names: one this project accepts, or any other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Identifier {
    Accepted(License),
    Other(String),
}

impl Identifier {
    fn read(token: &str) -> Self {
        License::from_spdx(token).map_or_else(|| Self::Other(token.to_owned()), Self::Accepted)
    }

    fn spdx(&self) -> &str {
        match self {
            Self::Accepted(license) => license.spdx(),
            Self::Other(identifier) => identifier,
        }
    }
}

/// An SPDX expression, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Identifier(Identifier),
    /// An expression qualified by an exception. The exception is not kept: a license with an
    /// exception is credited as that license.
    With(Box<Expression>),
    /// Every part, joined by `AND`: at least two.
    All(Vec<Expression>),
    /// One of the alternatives, joined by `OR`: at least two.
    Any(Vec<Expression>),
}

/// Why a crate's expression credits no license.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChoiceFault {
    /// The manifest states no license.
    NoLicense,
    /// The expression is not one this reader can read.
    Unreadable { expression: String },
    /// Every choice the expression leaves names a license this project does not accept.
    NoneAccepted { expression: String },
}

impl fmt::Display for ChoiceFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoLicense => formatter.write_str("declares no license"),
            Self::Unreadable { expression } => {
                write!(
                    formatter,
                    "cannot read the license expression {expression:?}"
                )
            }
            Self::NoneAccepted { expression } => write!(
                formatter,
                "is licensed {expression}, which leaves no license this project accepts: {}",
                License::accepted()
            ),
        }
    }
}

/// One token of an expression.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Open,
    Close,
    And,
    Or,
    With,
    Word(String),
}

/// The tokens of `expression`, with the legacy `/` read as `OR`: parentheses, and runs of ASCII
/// letters, digits, `.`, `+`, and `-`, separated by any space.
fn tokens(expression: &str) -> Option<Vec<Token>> {
    let text = expression.replace('/', " OR ");
    let mut found = Vec::new();
    let mut characters = text.chars().peekable();
    while let Some(&character) = characters.peek() {
        if character.is_whitespace() {
            characters.next();
            continue;
        }
        if character == '(' || character == ')' {
            characters.next();
            found.push(if character == '(' {
                Token::Open
            } else {
                Token::Close
            });
            continue;
        }
        let mut word = String::new();
        while let Some(&next) = characters.peek() {
            if next.is_ascii_alphanumeric() || matches!(next, '.' | '+' | '-') {
                word.push(next);
                characters.next();
            } else {
                break;
            }
        }
        if word.is_empty() {
            return None;
        }
        found.push(match word.as_str() {
            "AND" => Token::And,
            "OR" => Token::Or,
            "WITH" => Token::With,
            _ => Token::Word(word),
        });
    }
    Some(found)
}

/// A reader of one expression's tokens, by recursive descent.
struct Reader {
    tokens: Vec<Token>,
    position: usize,
}

impl Reader {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn take(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += 1;
        token
    }

    fn joined(mut parts: Vec<Expression>, join: fn(Vec<Expression>) -> Expression) -> Expression {
        if parts.len() == 1 {
            if let Some(only) = parts.pop() {
                return only;
            }
        }
        join(parts)
    }

    /// conjunction (`OR` conjunction)*
    fn disjunction(&mut self) -> Option<Expression> {
        let mut alternatives = vec![self.conjunction()?];
        while self.peek() == Some(&Token::Or) {
            self.take();
            alternatives.push(self.conjunction()?);
        }
        Some(Self::joined(alternatives, Expression::Any))
    }

    /// atom (`AND` atom)*
    fn conjunction(&mut self) -> Option<Expression> {
        let mut parts = vec![self.atom()?];
        while self.peek() == Some(&Token::And) {
            self.take();
            parts.push(self.atom()?);
        }
        Some(Self::joined(parts, Expression::All))
    }

    /// (`(` disjunction `)` | identifier) [`WITH` exception]
    fn atom(&mut self) -> Option<Expression> {
        let read = match self.take()? {
            Token::Open => {
                let inner = self.disjunction()?;
                if self.take()? != Token::Close {
                    return None;
                }
                inner
            }
            Token::Word(word) => Expression::Identifier(Identifier::read(&word)),
            Token::Close | Token::And | Token::Or | Token::With => return None,
        };
        if self.peek() == Some(&Token::With) {
            self.take();
            return match self.take()? {
                Token::Word(_) => Some(Expression::With(Box::new(read))),
                _ => None,
            };
        }
        Some(read)
    }
}

impl Expression {
    /// Reads an SPDX expression as a manifest's `license` states it.
    pub fn parse(expression: &str) -> Result<Self, ChoiceFault> {
        if expression.trim().is_empty() {
            return Err(ChoiceFault::NoLicense);
        }
        let unreadable = || ChoiceFault::Unreadable {
            expression: expression.to_owned(),
        };
        let tokens = tokens(expression).ok_or_else(unreadable)?;
        let mut reader = Reader {
            tokens,
            position: 0,
        };
        let read = reader.disjunction().ok_or_else(unreadable)?;
        if reader.position != reader.tokens.len() {
            return Err(unreadable());
        }
        Ok(read)
    }

    /// Every license identifier the expression names, its exceptions left out.
    pub fn identifiers(&self) -> BTreeSet<&str> {
        let mut found = BTreeSet::new();
        self.collect_identifiers(&mut found);
        found
    }

    fn collect_identifiers<'a>(&'a self, found: &mut BTreeSet<&'a str>) {
        match self {
            Self::Identifier(identifier) => {
                found.insert(identifier.spdx());
            }
            Self::With(expression) => expression.collect_identifiers(found),
            Self::All(parts) | Self::Any(parts) => {
                for part in parts {
                    part.collect_identifiers(found);
                }
            }
        }
    }

    /// Whether the expression names `license` anywhere, as an alternative or as a part.
    pub fn names(&self, license: License) -> bool {
        self.identifiers().contains(license.spdx())
    }

    /// The licenses this choice credits, with repeats, or `None` where it names one this project
    /// does not accept.
    fn credited(&self) -> Option<Vec<License>> {
        match self {
            Self::Identifier(Identifier::Accepted(license)) => Some(vec![*license]),
            Self::Identifier(Identifier::Other(_)) => None,
            Self::With(expression) => expression.credited(),
            Self::All(parts) => {
                let mut every = Vec::new();
                for part in parts {
                    every.extend(part.credited()?);
                }
                Some(every)
            }
            // The alternative whose least-preferred license comes earliest: each alternative's
            // licenses, most preferred last, compared in turn. The first of equals wins.
            Self::Any(alternatives) => {
                alternatives
                    .iter()
                    .filter_map(Self::credited)
                    .min_by_key(|licenses| {
                        let mut key = licenses.clone();
                        key.sort_unstable_by(|one, other| other.cmp(one));
                        key
                    })
            }
        }
    }
}

impl Expression {
    /// The licenses a crate is credited under, each once, in the order of preference: never none,
    /// because an expression that leaves no accepted choice is refused, naming it as `written`.
    pub fn credit(&self, written: &str) -> Result<Vec<License>, ChoiceFault> {
        let chosen: BTreeSet<License> = self
            .credited()
            .ok_or_else(|| ChoiceFault::NoneAccepted {
                expression: written.to_owned(),
            })?
            .into_iter()
            .collect();
        Ok(chosen.into_iter().collect())
    }
}

/// The licenses a crate whose manifest states `expression` is credited under, each once, in the
/// order of preference.
#[cfg(test)]
pub fn choose(expression: &str) -> Result<Vec<License>, ChoiceFault> {
    Expression::parse(expression)?.credit(expression)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spdx(licenses: &[License]) -> Vec<&'static str> {
        licenses.iter().map(|license| license.spdx()).collect()
    }

    fn chosen(expression: &str) -> Vec<&'static str> {
        spdx(&choose(expression).unwrap_or_else(|fault| panic!("{expression}: {fault}")))
    }

    #[test]
    fn under_or_the_first_accepted_license_is_credited() {
        for expression in [
            "Apache-2.0 OR MIT",
            "MIT/Apache-2.0",
            "MIT / Apache-2.0",
            "Unlicense OR MIT",
            "BSD-2-Clause OR Apache-2.0 OR MIT",
        ] {
            assert_eq!(chosen(expression), ["MIT"], "{expression}");
        }
        assert_eq!(chosen("Apache-2.0 OR MPL-2.0"), ["Apache-2.0"]);
    }

    #[test]
    fn under_and_every_part_is_credited() {
        assert_eq!(
            chosen("(MIT OR Apache-2.0) AND Unicode-3.0"),
            ["MIT", "Unicode-3.0"]
        );
        assert_eq!(chosen("Zlib AND MIT"), ["MIT", "Zlib"]);
    }

    #[test]
    fn with_credits_the_license_it_qualifies() {
        assert_eq!(chosen("Apache-2.0 WITH LLVM-exception"), ["Apache-2.0"]);
    }

    #[test]
    fn a_license_outside_the_accepted_list_is_refused() {
        assert_eq!(
            choose("GPL-3.0-only"),
            Err(ChoiceFault::NoneAccepted {
                expression: String::from("GPL-3.0-only")
            })
        );
        assert!(matches!(
            choose("GPL-3.0-only AND MIT"),
            Err(ChoiceFault::NoneAccepted { .. })
        ));
        assert_eq!(choose(""), Err(ChoiceFault::NoLicense));
        assert_eq!(chosen("GPL-3.0-only OR MIT"), ["MIT"]);
    }

    /// The alternative whose least-preferred license comes earliest: MIT AND Zlib against
    /// Apache-2.0 alone is Apache-2.0, because Zlib comes after it.
    #[test]
    fn under_or_an_alternative_is_judged_by_its_least_preferred_license() {
        assert_eq!(chosen("(MIT AND Zlib) OR Apache-2.0"), ["Apache-2.0"]);
        assert_eq!(chosen("(MIT AND ISC) OR Zlib"), ["MIT", "ISC"]);
    }

    #[test]
    fn an_expression_this_reader_cannot_read_is_refused() {
        for expression in [
            "MIT OR",
            "(MIT",
            "MIT)",
            "MIT and Apache-2.0",
            "MIT WITH",
            "MIT, Apache",
            "AND MIT",
        ] {
            assert!(
                matches!(choose(expression), Err(ChoiceFault::Unreadable { .. })),
                "{expression}"
            );
        }
    }

    #[test]
    fn an_expression_names_its_identifiers_and_not_its_exceptions() {
        let read = Expression::parse("(MIT OR Apache-2.0 WITH LLVM-exception) AND GPL-2.0+")
            .expect("it reads");
        assert_eq!(
            read.identifiers().into_iter().collect::<Vec<_>>(),
            ["Apache-2.0", "GPL-2.0+", "MIT"]
        );
        assert!(read.names(License::Apache2));
        assert!(!read.names(License::Zlib));
        for license in License::PREFERENCE {
            assert_eq!(License::from_spdx(license.spdx()), Some(license));
        }
    }
}
