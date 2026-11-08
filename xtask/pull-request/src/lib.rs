//! `cargo xtask check-pull-request`: the guards a pull request must satisfy.
//!
//! Why this is separate from `scripts/check`
//! -----------------------------------------
//! `scripts/check` answers "is this revision healthy?", a question about a tree of files, which is
//! why `nix flake check` can answer it inside a sandbox that holds a snapshot and no `.git` at all.
//! The questions here are not about a tree. They are about a *proposal*: what it is titled, and
//! what it changes relative to the branch it targets. Neither is visible in a snapshot, so neither
//! can be a stage in that suite without breaking the property that makes the suite trustworthy,
//! that continuous integration and a contributor's machine run the identical thing.
//!
//! So this is a tool of its own, run by `.github/workflows/pr-guard.yml` against the pull-request
//! event. It is a tool rather than inline workflow YAML for the reason `scripts/check` gives: logic
//! that lives only in a workflow file is logic nobody can run before pushing, and a gate a
//! contributor cannot rehearse is a gate that teaches nothing until it fails.
//!
//! How it divides work with `cargo xtask check-release`
//! ----------------------------------------------------
//! `check-release` asks whether the *settled record* is coherent: every archived change records a
//! version, versions are distinct, and each has a changelog heading. It reads files and never Git,
//! deliberately, because it runs where Git is absent.
//!
//! This tool asks the one question that requires Git and therefore cannot live there: did *this
//! change, in flight* write down what it did? `check-release` cannot notice a pull request that
//! shipped a behaviour change and touched nothing in `CHANGELOG.md`, because the fault is invisible
//! in the tree: the changelog is well-formed either way. Comparing two refs is the only way to see
//! it, and a workflow is the only place that has both.
//!
//! The two rules
//! -------------
//! 1. **The title reads as this project's history reads.** The repository squash-merges, so a pull
//!    request's title becomes a commit subject verbatim and is what someone reads in `git log`
//!    years later. The history is imperative prose, "Finish the archival, and fix the gate that
//!    never fired", not Conventional Commits, and this check encodes the convention the project
//!    has rather than importing one it does not.
//!
//! 2. **A change records what it changed.** `CHANGELOG.md` must appear in the diff. The
//!    `skip-changelog` label is the escape hatch, because some pull requests change nothing a
//!    reader of the changelog would care about, and a gate with no exit is a gate people learn to
//!    route around.
//!
//! What rule 1 cannot check
//! ------------------------
//! Imperative mood is not mechanically decidable: "Bring", "Bed", and "Added" are not separable by
//! suffix, and a part-of-speech tagger is not a dependency this project is going to grow for a
//! title lint. So the check refuses a short list of openers that are unambiguously not imperative,
//! and is silent about the rest. It catches the common miss, "Fixed the parser", and lets an
//! unusual correct title through. A lint that guesses is worse than one with a stated blind spot.
//!
//! The title alone
//! ---------------
//! `--title-only` checks the title against rule 1 and reads no diff and no labels. The private
//! working repository's export runs it from the tree it exports, before it commits anything,
//! because the public repository's guard runs that tree's own rules on the export's pull request.
//!
//! Reading the title
//! -----------------
//! The rules were first a Python script, and they read a title as it did, so a title this passes
//! is one that script passed: the title is trimmed of white space and split into words at runs of
//! it, white space being Unicode's and the four ASCII separators U+001C to U+001F as well; its
//! length is counted in characters; and the openers are matched without regard to case, the
//! dotless and dotted capital `i` counting as `i`. Whether a character is a letter, an upper-case
//! one, or a digit is Rust's reading of Unicode, which differs from the script's on a few
//! characters no title has used: combining marks, letter-like numerals, and circled letters. A
//! quoted title is quoted as the script quoted it.
//!
//! Exit codes match `cargo xtask check-release`: 0 clean, 1 faults found, 2 the tool could not ask
//! the question.
//!
//! A member of its own
//! -------------------
//! The rules are a part of the tools crate with a workspace member of their own,
//! `htmlspec-pull-request`, because the guard runs on every edit of a title or a label, and the
//! tools crate compiles the program's library first: from an empty cache the guard's job took 137
//! seconds, 109 of them building the tools, past the two minutes after which the change's design
//! moves the rules. This crate depends on `clap` and `serde_json` alone, and its binary,
//! `check-pull-request`, is what the guard builds and runs. `cargo xtask check-pull-request` takes
//! the same arguments, through [`Arguments`], and runs the same code. The report every tool ends
//! with, [`report`], lives here too, as the crate the others depend on.

pub mod report;

use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::str::FromStr;

use crate::report::{Exit, Report};

/// The command line, which the guard's binary and `cargo xtask check-pull-request` share.
#[derive(Clone, Debug, clap::Args)]
pub struct Arguments {
    /// The pull request's title, which becomes the squash commit's subject.
    #[arg(long, value_name = "TITLE")]
    pub title: String,
    /// The ref the pull request targets, read when --changed is absent.
    #[arg(long, value_name = "REF", default_value = "origin/main")]
    pub base: String,
    /// A file of the changed paths, one per line, or - for standard input.
    #[arg(long, value_name = "FILE")]
    pub changed: Option<Source>,
    /// The pull request's labels, as a JSON array of names, such as ["skip-changelog"].
    #[arg(long, value_name = "JSON", default_value = "[]")]
    pub labels_json: String,
    /// Check the title alone, against the title rules, and read no diff and no labels. The private
    /// working repository's export runs this from the tree it exports, because the public
    /// repository's guard runs that tree's rules on the export's pull request.
    #[arg(long, conflicts_with_all = ["base", "changed", "labels_json"])]
    pub title_only: bool,
}

impl Arguments {
    /// What the arguments ask the tool to check.
    pub fn job(self) -> Job {
        if self.title_only {
            Job::TitleOnly { title: self.title }
        } else {
            Job::Whole {
                title: self.title,
                changed: match self.changed {
                    Some(source) => Changed::Listed(source),
                    None => Changed::FromGit { base: self.base },
                },
                labels_json: self.labels_json,
            }
        }
    }
}

/// A squash-merged subject wider than this wraps or is truncated in `git log --oneline`, in
/// GitHub's commit list, and in most terminals. The longest subject in this project's history is
/// well inside it, so the limit constrains nothing that already reads well.
pub const MAX_TITLE: usize = 72;

/// The floor, as a character count rather than a word count. A word count looked like the natural
/// rule and was wrong: "Archive distinguish-command-names-from-skill-names" is two words and one of
/// the most specific subjects in this project's history, while "Dark mode" is two words and names
/// an area rather than a change. Length separates them; a word count calls them equivalent.
///
/// 16 sits just under the shortest subject the project has written, "Add readme markdown", 19, so
/// it refuses labels without second-guessing history.
pub const MIN_TITLE: usize = 16;

/// The fewest words a title may hold.
pub const MIN_WORDS: usize = 2;

/// The file that must appear in a pull request's diff.
pub const CHANGELOG: &str = "CHANGELOG.md";

/// The label that excuses the changelog.
pub const SKIP_LABEL: &str = "skip-changelog";

/// Openers that are unambiguously reports rather than instructions. Deliberately short: see "What
/// rule 1 cannot check" above. Compared against the first word, lower-cased, with any `:` and `,`
/// at its ends removed.
const NON_IMPERATIVE: [&str; 27] = [
    "added",
    "adding",
    "adds",
    "changed",
    "changes",
    "changing",
    "created",
    "creates",
    "creating",
    "fixed",
    "fixes",
    "fixing",
    "implemented",
    "implements",
    "implementing",
    "moved",
    "moves",
    "moving",
    "removed",
    "removes",
    "removing",
    "renamed",
    "renames",
    "renaming",
    "updated",
    "updates",
    "updating",
];

/// One way a title departs from the convention.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TitleFault {
    /// Nothing but white space.
    Empty,
    /// It opens with `wip`, `[wip]`, `draft`, or `[draft]`, in any case.
    WorkInProgress { title: String },
    /// It opens with a Conventional Commits prefix, such as `feat:` or `fix(parser)!:`. Refused
    /// rather than ignored, because a repository whose history is half prose and half type
    /// prefixes has no convention at all.
    Conventional { title: String },
    /// More characters than [`MAX_TITLE`].
    TooLong { length: usize },
    /// Fewer characters than [`MIN_TITLE`], or fewer words than [`MIN_WORDS`].
    TooShort { length: usize, words: usize },
    /// Its first character is not a letter.
    OpensWithoutAWord { first: char },
    /// Its first character is a letter, and not an upper-case one.
    Lowercase { title: String },
    /// Its first word is one of the openers that report rather than instruct.
    Reports { first: String },
    /// It ends with a full stop.
    EndsWithAPeriod,
    /// It ends with a pull request's number, ` (#123)`, which GitHub appends to a squash subject
    /// itself.
    Numbered { title: String },
}

impl fmt::Display for TitleFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("the title is empty. It becomes the squash commit's subject"),
            Self::WorkInProgress { title } => write!(
                formatter,
                "the title is marked as work in progress: {}. Mark the pull request itself a draft \
                 instead, so the title is the one it merges with",
                quoted(title)
            ),
            Self::Conventional { title } => write!(
                formatter,
                "the title carries a Conventional Commits prefix: {}. This project's history is \
                 imperative prose — see `git log --format=%s`",
                quoted(title)
            ),
            Self::TooLong { length } => write!(
                formatter,
                "the title is {length} characters, over {MAX_TITLE}. It becomes a commit subject, \
                 which is read in one line or not at all"
            ),
            Self::TooShort { length, words } => write!(
                formatter,
                "the title is {length} character(s) in {words} word(s), under {MIN_TITLE}. Say what \
                 the change does, not what area it touches"
            ),
            Self::OpensWithoutAWord { first } => write!(
                formatter,
                "the title starts with {}. A commit subject opens with a word",
                quoted(&first.to_string())
            ),
            Self::Lowercase { title } => write!(
                formatter,
                "the title starts lowercase: {}. A commit subject is a sentence, not a branch name",
                quoted(title)
            ),
            Self::Reports { first } => write!(
                formatter,
                "the title opens with {}, which reports rather than instructs. This project's \
                 subjects are imperative: 'Fix the gate', not 'Fixed the gate'",
                quoted(first)
            ),
            Self::EndsWithAPeriod => {
                formatter.write_str("the title ends with a period. A commit subject is not a sentence")
            }
            Self::Numbered { title } => write!(
                formatter,
                "the title already names a pull request number: {}. GitHub appends one on squash, \
                 and the subject would carry two",
                quoted(title)
            ),
        }
    }
}

/// One way a pull request's diff fails to record the change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangelogFault {
    /// The diff names no file. A fault of its own kind: it means the tool was handed nothing to
    /// judge, and reporting "no changelog entry" would blame the pull request for the missing
    /// input.
    NothingChanged,
    /// The diff does not name [`CHANGELOG`], and no label excuses it.
    Untouched,
}

impl fmt::Display for ChangelogFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingChanged => formatter.write_str(
                "the pull request changes no files, or the diff could not be read. Nothing here can \
                 be judged",
            ),
            Self::Untouched => write!(
                formatter,
                "{CHANGELOG} is untouched. Record what this change does for someone reading the \
                 project a year from now, or label the pull request {} to state that there is \
                 nothing to record",
                quoted(SKIP_LABEL)
            ),
        }
    }
}

/// Whether `character` is white space as the title is read: Unicode's, and the four ASCII
/// information separators, U+001C to U+001F, which the Python script's reading counted as well.
fn is_space(character: char) -> bool {
    character.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&character)
}

/// Whether `character` belongs to a word, for the boundary after `wip` and `draft`.
fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// Whether `character` is `letter`, an ASCII lower-case letter, without regard to case: its
/// upper-case form, and the two characters whose lower case is `i` or `k` though they are not
/// ASCII, and the two the script's matching took for `i` and `s` beside them.
fn folds_to(character: char, letter: char) -> bool {
    character.to_ascii_lowercase() == letter
        || matches!(
            (letter, character),
            ('i', '\u{130}' | '\u{131}') | ('k', '\u{212a}') | ('s', '\u{17f}')
        )
}

/// What follows `word` at the start of `text`, matched without regard to case, or nothing.
fn after_folded<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    let mut characters = text.char_indices();
    for letter in word.chars() {
        match characters.next() {
            Some((_, character)) if folds_to(character, letter) => {}
            _ => return None,
        }
    }
    Some(characters.next().map_or("", |(at, _)| &text[at..]))
}

/// Whether the title opens with a marker meaning "not ready": `wip` or `draft` as a word, or
/// either in square brackets, in any case.
fn marked_as_work_in_progress(title: &str) -> bool {
    ["wip", "draft"].into_iter().any(|word| {
        let bare = after_folded(title, word)
            .is_some_and(|rest| rest.chars().next().is_none_or(|next| !is_word(next)));
        let bracketed = title
            .strip_prefix('[')
            .and_then(|rest| after_folded(rest, word))
            .is_some_and(|rest| rest.starts_with(']'));
        bare || bracketed
    })
}

/// Whether the title opens with a Conventional Commits prefix: lower-case ASCII letters, a scope in
/// brackets where there is one, an optional `!`, a colon, and white space.
fn conventional(title: &str) -> bool {
    let letters = title
        .find(|character: char| !character.is_ascii_lowercase())
        .unwrap_or(title.len());
    if letters == 0 {
        return false;
    }
    let rest = &title[letters..];
    let rest = match rest.strip_prefix('(') {
        Some(scope) => match scope.find(')') {
            Some(close) => &scope[close + 1..],
            None => return false,
        },
        None => rest,
    };
    let rest = rest.strip_prefix('!').unwrap_or(rest);
    rest.strip_prefix(':')
        .and_then(|rest| rest.chars().next())
        .is_some_and(is_space)
}

/// Whether the title ends with ` (#123)`, a pull request's number.
fn numbered(title: &str) -> bool {
    let Some(inside) = title.strip_suffix(')') else {
        return false;
    };
    let digits: String = inside
        .chars()
        .rev()
        .take_while(|character| character.is_numeric())
        .collect();
    !digits.is_empty() && inside[..inside.len() - digits.len()].ends_with("(#")
}

/// The title as it is judged: trimmed of white space at both ends.
pub fn trimmed(title: &str) -> &str {
    title.trim_matches(is_space)
}

/// Every way the title departs from the convention, so one run reports all of them: none for a
/// title the rules take.
///
/// A list rather than the first fault, for the reason `scripts/check` keeps going after a failing
/// stage: a contributor fixing three problems should learn about all three in one run.
pub fn title_faults(title: &str) -> Vec<TitleFault> {
    let title = trimmed(title);
    if title.is_empty() {
        return vec![TitleFault::Empty];
    }
    let mut faults = Vec::new();
    if marked_as_work_in_progress(title) {
        faults.push(TitleFault::WorkInProgress {
            title: title.to_owned(),
        });
    }
    if conventional(title) {
        faults.push(TitleFault::Conventional {
            title: title.to_owned(),
        });
    }
    let length = title.chars().count();
    if length > MAX_TITLE {
        faults.push(TitleFault::TooLong { length });
    }
    let words: Vec<&str> = title
        .split(is_space)
        .filter(|word| !word.is_empty())
        .collect();
    if words.len() < MIN_WORDS || length < MIN_TITLE {
        faults.push(TitleFault::TooShort {
            length,
            words: words.len(),
        });
    }
    // A trimmed title that is not empty holds a word, and a word a character.
    if let Some(first_word) = words.first() {
        if let Some(first) = first_word.chars().next() {
            if !first.is_alphabetic() {
                faults.push(TitleFault::OpensWithoutAWord { first });
            } else if !first.is_uppercase() {
                faults.push(TitleFault::Lowercase {
                    title: title.to_owned(),
                });
            }
        }
        let opener = first_word.to_lowercase();
        if NON_IMPERATIVE.contains(&opener.trim_matches([':', ','])) {
            faults.push(TitleFault::Reports {
                first: (*first_word).to_owned(),
            });
        }
    }
    if title.ends_with('.') {
        faults.push(TitleFault::EndsWithAPeriod);
    }
    if numbered(title) {
        faults.push(TitleFault::Numbered {
            title: title.to_owned(),
        });
    }
    faults
}

/// Whether the change wrote down what it did: no fault where a label excuses it or the diff names
/// [`CHANGELOG`].
pub fn changelog_faults(changed: &[String], labels: &[String]) -> Vec<ChangelogFault> {
    if labels.iter().any(|label| label == SKIP_LABEL) {
        Vec::new()
    } else if changed.is_empty() {
        vec![ChangelogFault::NothingChanged]
    } else if !changed.iter().any(|path| path == CHANGELOG) {
        vec![ChangelogFault::Untouched]
    } else {
        Vec::new()
    }
}

/// Where `--changed` reads the list of changed files from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// `-`: standard input.
    StandardInput,
    /// A file, one path per line.
    File(PathBuf),
}

impl FromStr for Source {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(if text == "-" {
            Self::StandardInput
        } else {
            Self::File(PathBuf::from(text))
        })
    }
}

/// Where the changed files come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Changed {
    /// A list the caller gives.
    Listed(Source),
    /// The three-dot diff of `HEAD` against this ref: the files this branch changes since it left
    /// the ref, the same set GitHub lists under "Files changed".
    FromGit { base: String },
}

/// What a run checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Job {
    /// The title and the diff, as the pull-request guard checks them.
    Whole {
        title: String,
        changed: Changed,
        labels_json: String,
    },
    /// The title alone.
    TitleOnly { title: String },
}

/// Why the tool could not ask its question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CannotAsk {
    /// `--labels-json` is not a JSON array of strings.
    Labels { given: String },
    /// The list of changed files could not be read.
    Unreadable {
        source: Source,
        kind: std::io::ErrorKind,
    },
    /// Git could not give the diff against the base: an unfetched base is the usual cause, an
    /// environment problem rather than a finding about the pull request.
    NoDiff { base: String },
}

impl fmt::Display for CannotAsk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Labels { given } => write!(
                formatter,
                "--labels-json is not a JSON array of strings: {}",
                quoted(given)
            ),
            Self::Unreadable {
                source: Source::StandardInput,
                kind,
            } => write!(
                formatter,
                "cannot read --changed from standard input: {kind}"
            ),
            Self::Unreadable {
                source: Source::File(path),
                kind,
            } => write!(
                formatter,
                "cannot read --changed {}: {kind}",
                path.display()
            ),
            Self::NoDiff { base } => write!(
                formatter,
                "cannot diff against {base}. Fetch it, or pass --changed with the file list"
            ),
        }
    }
}

/// The labels, from `--labels-json`.
fn labels(given: &str) -> Result<Vec<String>, CannotAsk> {
    serde_json::from_str(given).map_err(|_| CannotAsk::Labels {
        given: given.to_owned(),
    })
}

/// The paths a list names: one per line, trimmed, blank lines left out.
pub fn listed(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.trim_matches(is_space))
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The files `HEAD` changes in `directory`'s repository since it left `base`, or nothing where
/// Git cannot say.
pub fn changed_from_git(base: &str, directory: &Path) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "-z"])
        .arg(format!("{base}...HEAD"))
        .current_dir(directory)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| String::from_utf8_lossy(path).into_owned())
            .collect(),
    )
}

/// How a run ends, given the job and a way to read the changed files.
pub fn report(
    job: &Job,
    read_changed: impl FnOnce(&Changed) -> Result<Vec<String>, CannotAsk>,
) -> Report {
    match job {
        Job::TitleOnly { title } => {
            let faults = title_faults(title);
            if faults.is_empty() {
                return passed_title(title);
            }
            let mut err = vec!["pull request:".to_owned()];
            err.extend(faults.iter().map(|fault| format!("!!! {fault}")));
            err.push(String::new());
            err.push(format!("{} fault(s) in the title", faults.len()));
            Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err,
            }
        }
        Job::Whole {
            title,
            changed,
            labels_json,
        } => {
            let asked = labels(labels_json)
                .and_then(|labels| read_changed(changed).map(|changed| (labels, changed)));
            let (labels, changed) = match asked {
                Ok(asked) => asked,
                Err(cannot) => return Report::could_not_ask(&cannot),
            };
            let faults: Vec<String> = title_faults(title)
                .iter()
                .map(ToString::to_string)
                .chain(
                    changelog_faults(&changed, &labels)
                        .iter()
                        .map(ToString::to_string),
                )
                .collect();
            if faults.is_empty() {
                let excused = if labels.iter().any(|label| label == SKIP_LABEL) {
                    format!(", {SKIP_LABEL} label present")
                } else {
                    String::new()
                };
                let mut passed = passed_title(title);
                passed.out.push(format!(
                    "  ok  {} changed file(s) record the change{excused}",
                    changed.len()
                ));
                return passed;
            }
            let mut err = vec!["pull request:".to_owned()];
            err.extend(faults.iter().map(|fault| format!("!!! {fault}")));
            err.push(String::new());
            err.push(format!(
                "{} fault(s) across {} changed file(s)",
                faults.len(),
                changed.len()
            ));
            Report {
                exit: Exit::Faults,
                out: Vec::new(),
                err,
            }
        }
    }
}

/// The line a title the rules take prints.
fn passed_title(title: &str) -> Report {
    Report {
        exit: Exit::Clean,
        out: vec![format!(
            "  ok  the title reads as a commit subject: {}",
            trimmed(title)
        )],
        err: Vec::new(),
    }
}

/// Reads the changed files from where the job says: a file, standard input, or Git, in the
/// directory the tool runs in.
fn read_changed(changed: &Changed) -> Result<Vec<String>, CannotAsk> {
    match changed {
        Changed::Listed(source) => {
            let mut text = String::new();
            let read = match source {
                Source::StandardInput => std::io::stdin().read_to_string(&mut text).map(|_| ()),
                Source::File(path) => std::fs::read_to_string(path).map(|read| text = read),
            };
            read.map_err(|error| CannotAsk::Unreadable {
                source: source.clone(),
                kind: error.kind(),
            })?;
            Ok(listed(&text))
        }
        Changed::FromGit { base } => changed_from_git(base, Path::new("."))
            .ok_or_else(|| CannotAsk::NoDiff { base: base.clone() }),
    }
}

/// Checks what the job names, and prints the outcome.
pub fn run(job: Job) -> ExitCode {
    report(&job, read_changed).print()
}

/// `text` quoted as the Python script quoted a string it named, with `repr`: in single quotes,
/// unless it holds a single quote and no double one, with a backslash, the quote, a tab, a line
/// feed, and a carriage return escaped, and a character that does not print written as its code.
pub fn quoted(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(text.len() + 2);
    out.push(quote);
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ if character == quote => {
                out.push('\\');
                out.push(quote);
            }
            _ if !prints(character) => {
                let code = u32::from(character);
                if code < 0x100 {
                    out.push_str(&format!("\\x{code:02x}"));
                } else if code < 0x10000 {
                    out.push_str(&format!("\\u{code:04x}"));
                } else {
                    out.push_str(&format!("\\U{code:08x}"));
                }
            }
            _ => out.push(character),
        }
    }
    out.push(quote);
    out
}

/// Whether a character prints as itself when quoted: not a control character, not white space but
/// the space, not a private-use character, and not one of Unicode's invisible format characters.
fn prints(character: char) -> bool {
    if character == ' ' {
        return true;
    }
    let code = u32::from(character);
    let format = matches!(
        code,
        0xad | 0x600..=0x605
            | 0x61c
            | 0x6dd
            | 0x70f
            | 0x890..=0x891
            | 0x8e2
            | 0x180e
            | 0x200b..=0x200f
            | 0x202a..=0x202e
            | 0x2060..=0x2064
            | 0x2066..=0x206f
            | 0xfeff
            | 0xfff9..=0xfffb
            | 0x110bd
            | 0x110cd
            | 0x13430..=0x1343f
            | 0x1bca0..=0x1bca3
            | 0x1d173..=0x1d17a
            | 0xe0001
            | 0xe0020..=0xe007f
    );
    let private = matches!(code, 0xe000..=0xf8ff | 0xf0000..=0xffffd | 0x100000..=0x10fffd);
    !(character.is_control() || character.is_whitespace() || format || private)
}

#[cfg(test)]
mod tests;
