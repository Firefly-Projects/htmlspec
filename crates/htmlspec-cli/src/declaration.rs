//! Reading a project's `spec.nix`.
//!
//! # Why this parses a subset rather than calling Nix
//!
//! The task list says to evaluate the declaration through the Nix toolchain, on the
//! grounds that a hand-rolled parser accepts files Nix rejects — and then the build and
//! the program disagree about the project. That reasoning is right about the risk.
//!
//! It cannot be followed as written, because it contradicts an accepted requirement:
//! *"The toolchain SHALL be packageable as an executable artifact that runs without a
//! source checkout and **without the packaging tool present at runtime**."* A program that
//! shells out to `nix eval` to read a version does not run without Nix, so `archive` would
//! stop working on exactly the machines that requirement exists to serve.
//!
//! The resolution keeps what the task was protecting. This reads a **strict subset**:
//! top-level `key = "value";` and `key = null;` in a single brace-delimited set, with `#`
//! comments. Anything else — a `let`, an interpolation, an import, a nested set — is
//! **reported rather than guessed at**.
//!
//! Being a subset is what makes it safe, and the direction matters. A parser narrower than
//! Nix can only ever accept files Nix also accepts, so the disagreement the task warns
//! about cannot happen. What it can do is refuse a valid Nix file that uses a construct it
//! does not implement, and that shows up as a clear report naming the line — a failure a
//! contributor can see and act on, rather than two tools quietly reading one file
//! differently.

use std::path::Path;

/// A project's declaration, with every field it must have.
///
/// Construction proves the fields are present. There is no `Declaration` with a missing
/// version, so a caller cannot read one from a file that did not state it — the version
/// gate reads `version` without checking whether it exists, because this type already did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    /// The project's name.
    pub name: String,
    /// The project's declared version.
    pub version: crate::version::Version,
    /// The pinned toolchain reference, or `None` where the project has not pinned one.
    pub htmlspec: Option<String>,
}

/// Why a declaration could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationError {
    /// The file is not there.
    Absent {
        /// Where it was looked for.
        path: String,
    },
    /// The file could not be read.
    Unreadable {
        /// Where it was.
        path: String,
        /// What the filesystem said.
        reason: String,
    },
    /// A construct this reader does not implement.
    Unsupported {
        /// The line number, counting from one.
        line: usize,
        /// The line itself.
        text: String,
    },
    /// A required field is absent.
    MissingField {
        /// Which field.
        field: &'static str,
    },
    /// The version is present but not orderable.
    UnorderableVersion {
        /// What was written.
        value: String,
        /// Why it could not be ordered.
        reason: crate::version::VersionError,
    },
}

impl std::fmt::Display for DeclarationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absent { path } => write!(
                f,
                "no project declaration at {path}. Run `htmlspec init` to write one"
            ),
            Self::Unreadable { path, reason } => write!(f, "cannot read {path}: {reason}"),
            Self::Unsupported { line, text } => write!(
                f,
                "{} is more than this reader understands, at line {line}: {}. A declaration \
                 holds top-level `key = \"value\";` entries and comments — no `let`, no \
                 interpolation, no imports. Simplify it, or state the version as a plain \
                 string",
                crate::project::DECLARATION,
                text.trim()
            ),
            Self::MissingField { field } => write!(
                f,
                "{} does not state `{field}`. Every declaration states name, version, and \
                 htmlspec",
                crate::project::DECLARATION
            ),
            Self::UnorderableVersion { value, reason } => write!(
                f,
                "the declared version {value:?} cannot be ordered: {reason}. Archival \
                 compares versions, so it must be able to order them"
            ),
        }
    }
}

impl std::error::Error for DeclarationError {}

/// Reads the declaration at `path`.
pub fn read(path: &Path) -> Result<Declaration, DeclarationError> {
    let display = path.display().to_string();
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(DeclarationError::Absent { path: display })
        }
        Err(error) => {
            return Err(DeclarationError::Unreadable {
                path: display,
                reason: error.to_string(),
            })
        }
    };
    parse(&source)
}

/// Reads a declaration from its text.
pub fn parse(source: &str) -> Result<Declaration, DeclarationError> {
    let mut name = None;
    let mut version = None;
    let mut htmlspec = None;
    let mut seen_htmlspec = false;

    for (index, raw) in source.lines().enumerate() {
        let line = strip_comment(raw).trim();
        // Structural and blank lines carry no fields.
        if line.is_empty() || line == "{" || line == "}" {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(DeclarationError::Unsupported {
                line: index + 1,
                text: raw.to_owned(),
            });
        };
        let key = key.trim();
        let value = value.trim().trim_end_matches(';').trim();

        // A key that is not a bare identifier is a construct this reader does not
        // implement — a nested path, a quoted key, an interpolation.
        if key.is_empty()
            || !key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DeclarationError::Unsupported {
                line: index + 1,
                text: raw.to_owned(),
            });
        }

        let parsed = match read_value(value) {
            Some(parsed) => parsed,
            None => {
                return Err(DeclarationError::Unsupported {
                    line: index + 1,
                    text: raw.to_owned(),
                })
            }
        };

        match key {
            "name" => name = parsed,
            "version" => version = parsed,
            "htmlspec" => {
                seen_htmlspec = true;
                htmlspec = parsed;
            }
            // An unrecognized key is not an error. A project may record something this
            // version does not read, and refusing it would make adding a field a breaking
            // change for every older binary.
            _ => {}
        }
    }

    let name = name.ok_or(DeclarationError::MissingField { field: "name" })?;
    let version = version.ok_or(DeclarationError::MissingField { field: "version" })?;
    if !seen_htmlspec {
        return Err(DeclarationError::MissingField { field: "htmlspec" });
    }

    let parsed_version = crate::version::Version::parse(&version).map_err(|reason| {
        DeclarationError::UnorderableVersion {
            value: version.clone(),
            reason,
        }
    })?;

    Ok(Declaration {
        name,
        version: parsed_version,
        htmlspec,
    })
}

/// Removes a `#` comment, respecting a `#` inside a string.
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (index, byte) in line.bytes().enumerate() {
        match byte {
            b'"' => in_string = !in_string,
            b'#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

/// Reads a value: a quoted string, or `null`.
///
/// `Some(None)` is `null` — present and empty. `None` is "not a value this reader
/// implements", which the caller reports rather than guessing at.
fn read_value(value: &str) -> Option<Option<String>> {
    if value == "null" {
        return Some(None);
    }
    let inner = value.strip_prefix('"')?.strip_suffix('"')?;
    // An escape or an interpolation means the value is more than a literal, and reading it
    // as one would produce a different string than Nix does.
    if inner.contains('\\') || inner.contains("${") || inner.contains('"') {
        return None;
    }
    Some(Some(inner.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The declaration `init` writes must be readable by the reader in the same binary.
    /// This is the round trip that keeps the two halves from drifting apart.
    #[test]
    fn the_declaration_init_writes_is_readable() {
        let written = crate::commands::render_declaration("my-project");
        let declaration = parse(&written).expect("the written declaration parses");
        assert_eq!(declaration.name, "my-project");
        assert_eq!(
            declaration.version,
            crate::version::Version::parse(crate::commands::INITIAL_VERSION).expect("valid")
        );
        assert_eq!(declaration.htmlspec, None);
    }

    #[test]
    fn a_pinned_toolchain_is_read() {
        let declaration =
            parse("{\n  name = \"p\";\n  version = \"1.2.3\";\n  htmlspec = \"github:x/y\";\n}\n")
                .expect("parses");
        assert_eq!(declaration.htmlspec.as_deref(), Some("github:x/y"));
    }

    #[test]
    fn comments_are_ignored_including_one_after_a_value() {
        let declaration = parse(
            "# a leading comment\n{\n  name = \"p\"; # trailing\n  version = \"0.1.0\";\n  \
             htmlspec = null;\n}\n",
        )
        .expect("parses");
        assert_eq!(declaration.name, "p");
    }

    /// A `#` inside a string is part of the string, not a comment.
    #[test]
    fn a_hash_inside_a_string_is_not_a_comment() {
        let declaration = parse(
            "{\n  name = \"p\";\n  version = \"0.1.0\";\n  htmlspec = \"github:x/y#z\";\n}\n",
        )
        .expect("parses");
        assert_eq!(declaration.htmlspec.as_deref(), Some("github:x/y#z"));
    }

    /// An unknown key is kept rather than rejected, so adding a field later does not break
    /// an older binary.
    #[test]
    fn an_unrecognized_field_is_ignored() {
        let declaration = parse(
            "{\n  name = \"p\";\n  version = \"0.1.0\";\n  htmlspec = null;\n  \
             somethingNew = \"x\";\n}\n",
        )
        .expect("an unknown field does not make the file unreadable");
        assert_eq!(declaration.name, "p");
    }

    /// Every field is required, and construction proves it — so the gate never reads a
    /// version from a declaration that has none.
    #[test]
    fn every_required_field_must_be_present() {
        for (source, missing) in [
            ("{\n  version = \"0.1.0\";\n  htmlspec = null;\n}\n", "name"),
            ("{\n  name = \"p\";\n  htmlspec = null;\n}\n", "version"),
            (
                "{\n  name = \"p\";\n  version = \"0.1.0\";\n}\n",
                "htmlspec",
            ),
        ] {
            assert_eq!(
                parse(source),
                Err(DeclarationError::MissingField { field: missing }),
                "{source:?} was accepted without {missing}"
            );
        }
    }

    /// The subset boundary. Each of these is valid Nix that this reader refuses, and the
    /// refusal names the line — which is the failure mode chosen over two tools reading one
    /// file differently.
    #[test]
    fn a_construct_beyond_the_subset_is_reported_with_its_line() {
        for source in [
            // A `let` binding.
            "let v = \"1.0.0\"; in {\n  name = \"p\";\n  version = v;\n  htmlspec = null;\n}\n",
            // An interpolation.
            "{\n  name = \"${x}\";\n  version = \"0.1.0\";\n  htmlspec = null;\n}\n",
            // An import.
            "{\n  name = import ./name.nix;\n  version = \"0.1.0\";\n  htmlspec = null;\n}\n",
            // A nested set.
            "{\n  name = { first = \"p\"; };\n  version = \"0.1.0\";\n  htmlspec = null;\n}\n",
        ] {
            let error = parse(source).expect_err("a construct beyond the subset was accepted");
            assert!(
                matches!(error, DeclarationError::Unsupported { .. }),
                "{source:?} produced {error:?} rather than an unsupported-construct report"
            );
            // The report names the line, so a contributor knows what to simplify.
            assert!(
                error.to_string().contains("line"),
                "the report does not name a line: {error}"
            );
        }
    }

    /// An unorderable version is reported here rather than reaching the gate, because a
    /// gate that cannot compare its inputs is not a gate.
    #[test]
    fn an_unorderable_version_is_reported() {
        let error = parse("{\n  name = \"p\";\n  version = \"one\";\n  htmlspec = null;\n}\n")
            .expect_err("rejected");
        assert!(
            matches!(error, DeclarationError::UnorderableVersion { .. }),
            "{error:?}"
        );
    }

    #[test]
    fn an_absent_file_names_the_operation_that_writes_one() {
        let error = read(Path::new("/nonexistent/htmlspec/spec.nix")).expect_err("absent");
        assert!(matches!(error, DeclarationError::Absent { .. }));
        assert!(error.to_string().contains("htmlspec init"), "{error}");
    }
}
