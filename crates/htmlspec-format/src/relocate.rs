//! Re-expressing a document's relative references for a directory that moved.
//!
//! Archival renames a change's directory one level deeper, and a relative reference written for the
//! old place then reaches another file: `../../assets/` from `htmlspec/changes/<name>/` is the
//! project's store, and from `htmlspec/changes/archive/<date>-<name>/` it is a directory that does not
//! exist. [`relocate`] rewrites each reference that would reach a different file after the move so
//! that it reaches the one it reached before, and leaves every other byte as it was.
//!
//! **Lexical, as a browser resolves a relative URL**, rather than through the filesystem. A reference
//! is resolved against the directory it was written for, and re-expressed from the one it will be
//! read in, so what the move changes is the spelling and never the target. A reference that reached
//! nothing before the move reaches the same nothing after it.
//!
//! **Which values are references** is the tag scanner's answer, as it is for
//! [`crate::asset_references`]: the attributes through which an element loads an asset,
//! [`htmlspec_assets::REFERENCING_ATTRIBUTES`], and the two through which a reader follows a link,
//! `a href` and `area href`, read from the start tags the parser builds. An author's quotation of
//! `href="…"` in text is words, and stays as written.
//!
//! [`crate::format`] does not call this, for the reason [`crate::rewrite`] gives: formatting stays a
//! pure function of its source. The caller that knows a document moved composes the two.

use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use htmlspec_assets::ReferencingAttribute;

use crate::tags;

/// The attributes through which a reader follows a link, which a move breaks as surely as it breaks
/// a stylesheet.
///
/// Not in [`htmlspec_assets::REFERENCING_ATTRIBUTES`], because a hyperlink is a citation rather than
/// a dependency and the asset rules have nothing to say about its bytes. A move has something to say
/// about where it leads.
const HYPERLINK_ATTRIBUTES: &[(&str, &str)] = &[("a", "href"), ("area", "href")];

/// A directory's move, from one place to another.
///
/// Both places are absolute, and the constructor is what says so: a reference is resolved against
/// one and re-expressed from the other, and a pair spelled from two different starting points would
/// relate two places that are not where the documents are.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Move {
    /// Where the directory was, as its components with `.` and `..` resolved.
    from: Vec<String>,
    /// Where it is going, the same way.
    to: Vec<String>,
}

/// A place given for a move is not an absolute path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotAbsolute(pub PathBuf);

impl Move {
    /// The move from `from` to `to`, refusing either one where it is not absolute.
    pub fn between(from: &Path, to: &Path) -> Result<Self, NotAbsolute> {
        Ok(Self {
            from: components(from)?,
            to: components(to)?,
        })
    }
}

/// An absolute path's components, with `.` and `..` resolved lexically.
fn components(path: &Path) -> Result<Vec<String>, NotAbsolute> {
    if !path.is_absolute() {
        return Err(NotAbsolute(path.to_owned()));
    }
    let mut resolved: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => resolved.push(name.to_string_lossy().into_owned()),
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir | Component::RootDir => {}
            Component::Prefix(prefix) => {
                resolved.push(prefix.as_os_str().to_string_lossy().into_owned());
            }
        }
    }
    Ok(resolved)
}

/// One reference the move re-expressed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rewritten {
    /// The reference as the document carried it.
    pub was: String,
    /// The reference as it now reads, reaching the same file from the new place.
    pub now: String,
}

/// A document's source after a move, and what changed in it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relocated {
    /// The source, with each reference that would have reached another file re-expressed.
    pub source: String,
    /// Every reference re-expressed, in document order.
    pub rewritten: Vec<Rewritten>,
}

impl Relocated {
    /// Whether the move changed nothing in the document.
    pub fn is_unchanged(&self) -> bool {
        self.rewritten.is_empty()
    }
}

/// Re-expresses each relative reference in `source` that would reach another file after `moved`.
///
/// A reference is rewritten exactly when, read from the new place, it would reach a different file
/// than it reached from the old one. So a reference within the directory, which moves with it, is
/// left as written, and so is one that names no relative path: a URL with a scheme, a path from the
/// root, a bare fragment or query, and a path written with a backslash, which a browser reads by rules
/// this does not follow.
pub fn relocate(source: &str, moved: &Move) -> Relocated {
    let mut out = String::with_capacity(source.len());
    let mut rewritten = Vec::new();
    let mut copied = 0;
    for span in movable_values(source) {
        let Some(value) = source.get(span.clone()) else {
            continue;
        };
        let Some(now) = moved.re_express(value) else {
            continue;
        };
        out.push_str(source.get(copied..span.start).unwrap_or_default());
        out.push_str(&now);
        rewritten.push(Rewritten {
            was: value.to_owned(),
            now,
        });
        copied = span.end;
    }
    out.push_str(source.get(copied..).unwrap_or_default());
    Relocated {
        source: out,
        rewritten,
    }
}

impl Move {
    /// `reference` as it must read from the new place to reach what it reached from the old one, or
    /// `None` where it reads correctly as it is.
    fn re_express(&self, reference: &str) -> Option<String> {
        let (path, suffix) =
            reference.split_at(reference.find(['?', '#']).unwrap_or(reference.len()));
        if path.is_empty() || path.starts_with('/') || path.contains('\\') || has_scheme(path) {
            return None;
        }
        let reached = Target::resolve(&self.from, path).moved(&self.from, &self.to);
        if Target::resolve(&self.to, path) == reached {
            return None;
        }
        Some(format!("{}{suffix}", reached.reached_from(&self.to)))
    }
}

/// Whether a path begins with a URL scheme, such as `https:` or `mailto:`.
///
/// A scheme is a letter followed by letters, digits, `+`, `-`, and `.`, up to a colon. A colon after
/// a `/` is part of a path segment, and does not start one.
fn has_scheme(path: &str) -> bool {
    let Some((scheme, _)) = path.split_once(':') else {
        return false;
    };
    let mut characters = scheme.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters.all(|rest| rest.is_ascii_alphanumeric() || matches!(rest, '+' | '-' | '.'))
}

/// What a relative path reaches: a file, or a directory, named by its components from the root.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Target {
    /// The components from the root, the last naming the file where this is not a directory.
    components: Vec<String>,
    /// Whether the path names a directory, as one ending in `/`, `.`, or `..` does.
    directory: bool,
}

impl Target {
    /// What `path` reaches from a document in `base`, resolved as a URL's path is.
    ///
    /// `.` names the directory it is in and `..` its parent, written plainly or percent-encoded as
    /// the URL standard allows. `..` at the root stays at the root.
    fn resolve(base: &[String], path: &str) -> Self {
        let mut components = base.to_vec();
        let mut directory = false;
        let mut segments = path.split('/').peekable();
        while let Some(segment) = segments.next() {
            let last = segments.peek().is_none();
            if is_single_dot(segment) {
                directory = last;
            } else if is_double_dot(segment) {
                components.pop();
                directory = last;
            } else if last && segment.is_empty() {
                directory = true;
            } else {
                components.push(segment.to_owned());
                directory = false;
            }
        }
        Self {
            components,
            directory,
        }
    }

    /// Where this target is after the directory `from` moves to `to`: beneath `to` where it was
    /// beneath `from`, and where it was otherwise.
    fn moved(self, from: &[String], to: &[String]) -> Self {
        match self.components.strip_prefix(from) {
            Some(within) => Self {
                components: to.iter().chain(within).cloned().collect(),
                directory: self.directory,
            },
            None => self,
        }
    }

    /// The relative path that reaches this target from a document in `base`.
    fn reached_from(&self, base: &[String]) -> String {
        let (directories, file) = match (self.directory, self.components.split_last()) {
            (false, Some((file, directories))) => (directories, Some(file.as_str())),
            _ => (self.components.as_slice(), None),
        };
        let shared = base
            .iter()
            .zip(directories)
            .take_while(|(one, other)| one == other)
            .count();
        let mut path = "../".repeat(base.len() - shared);
        for directory in directories.iter().skip(shared) {
            path.push_str(directory);
            path.push('/');
        }
        if let Some(file) = file {
            path.push_str(file);
        }
        if path.is_empty() {
            return String::from("./");
        }
        // A first segment holding a colon would be read as a scheme.
        if path
            .split('/')
            .next()
            .is_some_and(|first| first.contains(':'))
        {
            path.insert_str(0, "./");
        }
        path
    }
}

/// Whether a path segment is `.`, as the URL standard reads one.
fn is_single_dot(segment: &str) -> bool {
    segment == "." || segment.eq_ignore_ascii_case("%2e")
}

/// Whether a path segment is `..`, as the URL standard reads one.
fn is_double_dot(segment: &str) -> bool {
    matches!(
        segment.to_ascii_lowercase().as_str(),
        ".." | ".%2e" | "%2e." | "%2e%2e"
    )
}

/// Every value in `source` that a move can break, with where it sits.
fn movable_values(source: &str) -> Vec<Range<usize>> {
    tags::start_tags(source)
        .into_iter()
        .flat_map(|tags::StartTag { name, attributes }| {
            attributes.into_iter().filter_map(move |attribute| {
                let read = ReferencingAttribute::find(&name, &attribute.name).is_some()
                    || HYPERLINK_ATTRIBUTES
                        .iter()
                        .any(|(element, value)| *element == name && *value == attribute.name);
                if !read {
                    return None;
                }
                let written = source.get(attribute.value.clone())?;
                let value = written.trim();
                if value.is_empty() {
                    return None;
                }
                let start = attribute.value.start + (written.len() - written.trim_start().len());
                Some(start..start + value.len())
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The move archival makes: one level deeper, under a dated name.
    fn archival() -> Move {
        Move::between(
            Path::new("/project/htmlspec/changes/sample"),
            Path::new("/project/htmlspec/changes/archive/2026-10-14-sample"),
        )
        .expect("both places are absolute")
    }

    fn link(reference: &str) -> String {
        format!("<a href=\"{reference}\">a link</a>\n")
    }

    /// What one reference reads as after archival's move, written in a hyperlink.
    fn after_archival(reference: &str) -> String {
        let relocated = relocate(&link(reference), &archival());
        relocated
            .source
            .strip_prefix("<a href=\"")
            .and_then(|rest| rest.split_once('"'))
            .map(|(value, _)| value.to_owned())
            .expect("the hyperlink survives")
    }

    /// The defect: a reference out of the directory reaches the same file from the archive.
    #[test]
    fn a_reference_out_of_the_directory_reaches_the_same_file_from_the_new_one() {
        let source = "<link href=\"../../assets/sha256-03627137bcbb81c9/htmlspec.css\" \
                      rel=\"stylesheet\">\n\
                      <script defer=\"\" src=\"../../assets/sha256-84427613110957a7/htmlspec.js\">\
                      </script>\n\
                      <p><a href=\"../../specs/toolchain-packaging/spec.html#req-x\">the \
                      requirement</a></p>\n";
        let relocated = relocate(source, &archival());
        assert_eq!(
            relocated.source,
            source.replace("\"../../", "\"../../../"),
            "every reference out of the change should climb one level more"
        );
        assert_eq!(relocated.rewritten.len(), 3, "{:#?}", relocated.rewritten);
        assert_eq!(
            relocated.rewritten.first(),
            Some(&Rewritten {
                was: String::from("../../assets/sha256-03627137bcbb81c9/htmlspec.css"),
                now: String::from("../../../assets/sha256-03627137bcbb81c9/htmlspec.css"),
            })
        );
    }

    /// A reference that never leaves the directory moves with it, and is left as written.
    #[test]
    fn a_reference_within_the_directory_is_unchanged() {
        for reference in [
            "evidence/the-two-defects.md",
            "tasks.html#task-1-1",
            "./design.html",
            "evidence/../proposal.html",
        ] {
            assert_eq!(after_archival(reference), reference);
        }
    }

    /// A value that names no relative path is not the move's business.
    #[test]
    fn a_reference_naming_no_relative_path_is_unchanged() {
        for reference in [
            "https://example.com/a",
            "mailto:someone@example.com",
            "data:text/plain,x",
            "/absolute/path.html",
            "//example.com/a",
            "#hs-content",
            "?only=a-query",
            "c:relative-with-a-scheme",
        ] {
            assert_eq!(after_archival(reference), reference);
        }
        let empty = "<a href=\"\">empty</a><a href=\"  \">blank</a>\n";
        assert_eq!(relocate(empty, &archival()).source, empty);
    }

    /// A query and a fragment travel unchanged behind the rewritten path.
    #[test]
    fn a_query_and_a_fragment_are_kept() {
        assert_eq!(
            after_archival("../../assets/a.css?v=2#x"),
            "../../../assets/a.css?v=2#x"
        );
        assert_eq!(
            after_archival("../../specs/a/spec.html#req-a"),
            "../../../specs/a/spec.html#req-a"
        );
    }

    /// A directory reference stays one.
    #[test]
    fn a_directory_reference_keeps_its_trailing_slash() {
        assert_eq!(after_archival("../../specs/"), "../../../specs/");
    }

    /// A reference that leaves the directory and comes back into it is re-expressed by where it
    /// lands, which moved with the directory.
    #[test]
    fn a_reference_leaving_and_reentering_is_expressed_by_where_it_lands() {
        assert_eq!(after_archival("../sample/tasks.html"), "tasks.html");
        assert_eq!(
            after_archival("../sample/evidence/x.md#a"),
            "evidence/x.md#a"
        );
        assert_eq!(
            after_archival("../other/proposal.html"),
            "../../other/proposal.html"
        );
    }

    /// Text that quotes a reference, and attributes nobody loads or follows, are words.
    #[test]
    fn text_and_unread_attributes_are_unchanged() {
        let source = "<p><code>href=\"../../assets/quoted.css\"</code></p>\n\
                      <blockquote cite=\"../../cited.html\"><p>q</p></blockquote>\n\
                      <!-- <a href=\"../../commented.html\"> -->\n\
                      <script>var s = '<a href=\"../../scripted.html\">';</script>\n\
                      <p data-refs=\"../../not-a-url\">p</p>\n";
        let relocated = relocate(source, &archival());
        assert_eq!(relocated.source, source);
        assert!(relocated.is_unchanged());
    }

    /// Every attribute the asset rules read is relocated, and so is an image map's area.
    #[test]
    fn every_loading_attribute_and_both_hyperlinks_are_relocated() {
        let source = "<img alt=\"\" src=\"../../assets/a.png\">\n\
                      <map name=\"m\"><area alt=\"\" href=\"../../b.html\"></map>\n\
                      <object data=\"../../assets/c.svg\"></object>\n";
        assert_eq!(
            relocate(source, &archival()).source,
            source.replace("\"../../", "\"../../../")
        );
    }

    /// Only the value's bytes change: quoting, spacing, and the rest of the tag are copied through.
    #[test]
    fn every_byte_outside_a_rewritten_value_is_copied() {
        let source = "<A  HREF = '../../x.html'  class=c>single</A>\n\
                      <a href=../../y.html>unquoted</a>\n\
                      <a href=\"  ../../z.html  \">padded</a>\n";
        let expected = "<A  HREF = '../../../x.html'  class=c>single</A>\n\
                        <a href=../../../y.html>unquoted</a>\n\
                        <a href=\"  ../../../z.html  \">padded</a>\n";
        assert_eq!(relocate(source, &archival()).source, expected);
    }

    /// A move to where the directory already is changes nothing.
    #[test]
    fn a_move_to_the_same_place_changes_nothing() {
        let stay = Move::between(Path::new("/p/a"), Path::new("/p/a")).expect("absolute");
        let source = link("../../assets/a.css");
        assert_eq!(relocate(&source, &stay).source, source);
    }

    /// A place spelled relative to wherever the program runs is refused, not guessed at.
    #[test]
    fn a_relative_place_is_refused() {
        assert_eq!(
            Move::between(Path::new("htmlspec/changes/a"), Path::new("/p/b")),
            Err(NotAbsolute(PathBuf::from("htmlspec/changes/a")))
        );
        assert!(Move::between(Path::new("/p/a"), Path::new("b")).is_err());
    }

    /// The places are resolved lexically, so `..` in either one names its parent.
    #[test]
    fn a_place_spelled_with_parent_steps_is_resolved() {
        let spelled = Move::between(
            Path::new("/project/htmlspec/changes/../changes/sample"),
            Path::new("/project/htmlspec/changes/archive/./2026-10-14-sample"),
        )
        .expect("absolute");
        assert_eq!(spelled, archival());
    }
}
