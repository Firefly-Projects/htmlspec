//! The matchers: what in a tracked file is Python, and what is a program outside the four
//! permitted shell files.
//!
//! Each is a pure function of a file's path, its entry, and a [`Context`]: which paths are the
//! permitted shell files, which files are tracked, and which are handed to `make -f` somewhere.
//! Some find a file as a whole: a Python or shell file name, a file under `scripts/` beside the two
//! that belong there, a first line that runs Python or that cannot be read, a permitted shell file
//! that is not bash, a file that begins with `#!` or carries the executable bit, a submodule, and a
//! link out of the repository or to an interpreter. Others find lines: a name that means Python in
//! a file that runs, a tracked file a file that runs hands to a shell or an interpreter, a local
//! action outside `.github/` or one that runs a program, a name in a runbook's preflight
//! declaration, and a Rust `Command::new` that runs Python. Prose is never read: Markdown, HTML
//! other than a preflight declaration, YAML that is neither a workflow nor an action, and a Rust
//! string that no `Command::new` takes describe Python and run none.

mod nix;
mod rust_source;
mod shell;
mod words;

use std::collections::BTreeSet;
use std::fmt;
use std::num::NonZeroUsize;

use super::tracked::{Entry, FileSet, RepoPath};

pub use words::names_python;

/// What made a whole file a finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileRule {
    /// Its name is a Python source, bytecode, notebook, or packaging file's.
    PythonFileName,
    /// Its name is a shell script's.
    ShellFileName,
    /// It is under `scripts/`, which holds `scripts/check` and `scripts/install-hooks` alone.
    UnderScripts,
    /// Its first line runs Python, directly or through `env`.
    PythonShebang { interpreter: String },
    /// Its first line begins with `#!` and names no program the stage can read, so the stage
    /// refuses it rather than guess.
    UnreadableShebang,
    /// It is a permitted shell file whose first line is not exactly [`BASH`].
    NotBash,
    /// Its first line begins with `#!`, so it is a program, and it is none of the four permitted
    /// shell files.
    BeginsWithShebang,
    /// Its mode carries the executable bit, and it is none of the four permitted shell files.
    ExecutableBit,
    /// It is a submodule, whose code no Cargo workspace here builds.
    Submodule,
    /// It is a link to an absolute path or to a path outside the repository.
    LinkLeaves { target: String },
    /// It is a link whose target names an interpreter.
    LinkToInterpreter { target: String },
}

/// What made one line a finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineRule {
    /// A file that runs names Python.
    Names { word: String },
    /// A file that runs hands a shell, `source`, or an interpreter a tracked file to run that is
    /// none of the four permitted shell files.
    RunsScript { target: String },
    /// A workflow uses a local action outside `.github/`, where the repository's actions live.
    LocalActionOutside { target: String },
    /// A local action runs a program, JavaScript or a container, rather than composing steps.
    ActionRuns { using: String },
    /// A runbook's preflight declaration names Python.
    Declares { word: String },
    /// A Rust source's `Command::new` runs Python.
    Runs { word: String },
}

/// The first line every permitted shell file begins with: bash, through `env`.
pub const BASH: &str = "#!/usr/bin/env bash";

/// One thing the stage refuses, before the allow-list or the private list excuses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    File {
        path: RepoPath,
        rule: FileRule,
    },
    Line {
        path: RepoPath,
        number: NonZeroUsize,
        /// The line's exact bytes, without its line ending, which an allowed line names.
        text: Vec<u8>,
        rule: LineRule,
    },
}

impl Finding {
    pub fn path(&self) -> &RepoPath {
        match self {
            Self::File { path, .. } | Self::Line { path, .. } => path,
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const OUTSIDE: &str = "and it is none of the four permitted shell files";
        match self {
            Self::File { path, rule } => match rule {
                FileRule::PythonFileName => write!(f, "{path}: a Python file name"),
                FileRule::ShellFileName => {
                    write!(f, "{path}: a shell script's file name, {OUTSIDE}")
                }
                FileRule::UnderScripts => write!(
                    f,
                    "{path}: a file under scripts/, which holds scripts/check and \
                     scripts/install-hooks alone"
                ),
                FileRule::PythonShebang { interpreter } => {
                    write!(f, "{path}:1: a first line that runs {interpreter}")
                }
                FileRule::UnreadableShebang => write!(
                    f,
                    "{path}:1: a first line whose program this stage cannot read, which it refuses \
                     rather than guess"
                ),
                FileRule::NotBash => write!(
                    f,
                    "{path}:1: a permitted shell file whose first line is not {BASH}"
                ),
                FileRule::BeginsWithShebang => write!(f, "{path}:1: begins with #!, {OUTSIDE}"),
                FileRule::ExecutableBit => {
                    write!(f, "{path}: carries the executable bit, {OUTSIDE}")
                }
                FileRule::Submodule => write!(
                    f,
                    "{path}: a submodule, whose code no Cargo workspace here builds"
                ),
                FileRule::LinkLeaves { target } => {
                    write!(f, "{path}: a link to {target}, outside the repository")
                }
                FileRule::LinkToInterpreter { target } => {
                    write!(f, "{path}: a link to {target}, which names an interpreter")
                }
            },
            Self::Line {
                path, number, rule, ..
            } => match rule {
                LineRule::Names { word } => write!(f, "{path}:{number}: names {word}"),
                LineRule::RunsScript { target } => {
                    write!(f, "{path}:{number}: runs {target} as a script, {OUTSIDE}")
                }
                LineRule::LocalActionOutside { target } => write!(
                    f,
                    "{path}:{number}: uses the local action {target}, outside .github/, where the \
                     repository's actions live"
                ),
                LineRule::ActionRuns { using } => write!(
                    f,
                    "{path}:{number}: an action that runs {using}, a program no Cargo workspace builds"
                ),
                LineRule::Declares { word } => {
                    write!(f, "{path}:{number}: a preflight declaration names {word}")
                }
                LineRule::Runs { word } => {
                    write!(f, "{path}:{number}: Command::new runs {word}")
                }
            },
        }
    }
}

/// Every finding in one tracked file, read in the context of the rest of the repository.
pub fn findings(path: &RepoPath, entry: &Entry, context: &Context) -> Vec<Finding> {
    let is_permitted = context.permitted.contains(&path.as_str());
    let mut rules = Vec::new();
    let name = path.file_name();
    if python_file_name(name) {
        rules.push(FileRule::PythonFileName);
    }
    if shell_file_name(name) && !is_permitted {
        rules.push(FileRule::ShellFileName);
    }
    if path.is_under("scripts") && !is_permitted {
        rules.push(FileRule::UnderScripts);
    }
    let content = match entry {
        Entry::File {
            executable,
            content,
        } => {
            rules.extend(program_rules(content, *executable, is_permitted));
            content
        }
        Entry::Link { target } => {
            rules.extend(link_rules(path, target));
            return file_findings(path, rules);
        }
        Entry::Submodule => {
            rules.push(FileRule::Submodule);
            return file_findings(path, rules);
        }
    };
    let mut found = file_findings(path, rules);
    let shebang = shebang(content);
    let lines: Vec<(NonZeroUsize, &[u8])> = numbered_lines(content).collect();
    let text = |number: usize| -> Vec<u8> {
        lines
            .get(number.saturating_sub(1))
            .map(|(_, line)| line.to_vec())
            .unwrap_or_default()
    };
    let mut line_rules: Vec<(NonZeroUsize, LineRule)> = Vec::new();
    let is_rules = context.rules.contains(path.as_str());
    match reading(path, &shebang, is_permitted, is_rules) {
        Reading::Commands(syntax) => {
            for (number, line) in &lines {
                let word = match syntax {
                    Syntax::Nix => words::nix_identifier(line).or_else(|| words::python_word(line)),
                    Syntax::Shell | Syntax::Yaml { .. } => words::python_word(line),
                };
                if let Some((_, word)) = word {
                    line_rules.push((*number, LineRule::Names { word }));
                }
            }
            let at = |number: usize| NonZeroUsize::MIN.saturating_add(number.saturating_sub(1));
            for (number, command) in command_lines(content, syntax) {
                let mut targets = Vec::new();
                for handoff in shell::handoffs(&command) {
                    let (shell::Handoff::Script(word) | shell::Handoff::Input(word)) = handoff
                    else {
                        continue;
                    };
                    if let Some(file) = context.unpermitted_file(&word) {
                        if !targets.contains(&file) {
                            targets.push(file);
                        }
                    }
                }
                line_rules.extend(
                    targets
                        .into_iter()
                        .map(|target| (at(number), LineRule::RunsScript { target })),
                );
                if let Syntax::Yaml { action } = syntax {
                    for target in yaml_values(&command, "uses") {
                        let Some(local) = target.strip_prefix("./") else {
                            continue;
                        };
                        if !RepoPath::parse(local.trim_end_matches('/'))
                            .is_ok_and(|local| local.is_under(".github"))
                        {
                            line_rules.push((at(number), LineRule::LocalActionOutside { target }));
                        }
                    }
                    if action {
                        for using in yaml_values(&command, "using") {
                            if !using.eq_ignore_ascii_case("composite") {
                                line_rules.push((at(number), LineRule::ActionRuns { using }));
                            }
                        }
                    }
                }
            }
        }
        Reading::Preflight => line_rules.extend(preflight_lines(content, &lines)),
        Reading::Rust => {
            for run in rust_source::commands_running_python(content) {
                let index = content[..run.at].iter().filter(|b| **b == b'\n').count();
                line_rules.push((
                    NonZeroUsize::MIN.saturating_add(index),
                    LineRule::Runs { word: run.word },
                ));
            }
        }
        Reading::Nothing => {}
    }
    line_rules.sort_by_key(|(number, _)| *number);
    found.extend(line_rules.into_iter().map(|(number, rule)| Finding::Line {
        path: path.clone(),
        number,
        text: text(number.get()),
        rule,
    }));
    found
}

fn file_findings(path: &RepoPath, rules: Vec<FileRule>) -> Vec<Finding> {
    rules
        .into_iter()
        .map(|rule| Finding::File {
            path: path.clone(),
            rule,
        })
        .collect()
}

/// What a file's first line and mode make of it.
fn program_rules(content: &[u8], executable: bool, is_permitted: bool) -> Vec<FileRule> {
    let mut rules = Vec::new();
    match shebang(content) {
        Shebang::Program(name) if names_python(&name) => {
            rules.push(FileRule::PythonShebang { interpreter: name })
        }
        Shebang::Unreadable => rules.push(FileRule::UnreadableShebang),
        Shebang::Program(_) | Shebang::None => {}
    }
    if is_permitted && first_line(content) != BASH.as_bytes() {
        rules.push(FileRule::NotBash);
    }
    if begins_with_shebang(content) {
        rules.push(FileRule::BeginsWithShebang);
    }
    if executable {
        rules.push(FileRule::ExecutableBit);
    }
    rules
}

/// What a link's target makes of it.
fn link_rules(path: &RepoPath, target: &[u8]) -> Vec<FileRule> {
    let shown = String::from_utf8_lossy(target).into_owned();
    let mut rules = Vec::new();
    let mut depth = path.components().count().saturating_sub(1);
    let mut leaves = target.starts_with(b"/");
    for component in target.split(|byte| *byte == b'/') {
        match component {
            b"" | b"." => {}
            b".." if depth == 0 => leaves = true,
            b".." => depth -= 1,
            _ => depth += 1,
        }
    }
    if leaves {
        rules.push(FileRule::LinkLeaves {
            target: shown.clone(),
        });
    }
    let last = target
        .rsplit(|byte| *byte == b'/')
        .next()
        .unwrap_or_default();
    if names_python(&String::from_utf8_lossy(last)) {
        rules.push(FileRule::LinkToInterpreter { target: shown });
    }
    rules
}

/// Whether a file's name is a Python source's, bytecode's, notebook's, or packaging file's, in any
/// case.
pub fn python_file_name(name: &str) -> bool {
    const SUFFIXES: [&str; 7] = [".py", ".pyw", ".pyi", ".pyc", ".pyo", ".pyz", ".ipynb"];
    const NAMES: [&str; 9] = [
        "pyproject.toml",
        "setup.py",
        "setup.cfg",
        "pipfile",
        "pipfile.lock",
        "poetry.lock",
        "uv.lock",
        "tox.ini",
        ".python-version",
    ];
    let name = name.to_ascii_lowercase();
    SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
        || NAMES.contains(&name.as_str())
        || (name.starts_with("requirements") && name.ends_with(".txt"))
}

/// Whether a file's name is a shell script's, in any case.
pub fn shell_file_name(name: &str) -> bool {
    const SUFFIXES: [&str; 5] = [".sh", ".bash", ".zsh", ".ksh", ".dash"];
    let name = name.to_ascii_lowercase();
    SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
}

/// Whether the file begins with an interpreter line, `#!`. A Rust source may begin with an inner
/// attribute, `#![`, which rustc reads as an attribute and not as an interpreter line, and which a
/// Cargo workspace builds; so a first line that begins `#![` is not one.
pub fn begins_with_shebang(content: &[u8]) -> bool {
    content.starts_with(b"#!") && !content.starts_with(b"#![")
}

fn first_line(content: &[u8]) -> &[u8] {
    let line = content
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    line.strip_suffix(b"\r").unwrap_or(line)
}

/// What a first line `#!` runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shebang {
    /// The file does not begin with `#!`.
    None,
    /// The name of the program it runs, read through `env` and its options.
    Program(String),
    /// A line the stage cannot read with certainty: a quote or a backslash in it, an option of
    /// `env` it does not know, or no program at all.
    Unreadable,
}

/// What the file's first line runs.
pub fn shebang(content: &[u8]) -> Shebang {
    if !begins_with_shebang(content) {
        return Shebang::None;
    }
    let line = String::from_utf8_lossy(&first_line(content)[2..]).into_owned();
    if line.contains(['"', '\'', '\\']) {
        return Shebang::Unreadable;
    }
    let mut words: Vec<&str> = line.split_ascii_whitespace().collect();
    words.reverse();
    let Some(program) = words.pop() else {
        return Shebang::Unreadable;
    };
    if !basename(program).eq_ignore_ascii_case("env") {
        return Shebang::Program(basename(program).to_owned());
    }
    // `env`'s own options and assignments come before the program it runs. `-S` splits the rest of
    // the line into words, which splitting on whitespace has already done, and may be joined to
    // the first of them, as `--split-string=` may.
    while let Some(word) = words.pop() {
        match word {
            "-S"
            | "--split-string"
            | "-i"
            | "-"
            | "--ignore-environment"
            | "-0"
            | "--null"
            | "-v"
            | "--debug" => {}
            "-u" | "--unset" | "-C" | "--chdir" | "-P" => {
                if words.pop().is_none() {
                    return Shebang::Unreadable;
                }
            }
            _ if word.starts_with("--unset=") || word.starts_with("--chdir=") => {}
            _ if word.starts_with("--split-string=") => {
                words.push(&word["--split-string=".len()..]);
            }
            _ if word.starts_with("-S") => words.push(&word[2..]),
            _ if word.starts_with('-') => return Shebang::Unreadable,
            _ if word.contains('=') => {}
            _ => return Shebang::Program(basename(word).to_owned()),
        }
    }
    Shebang::Unreadable
}

fn basename(program: &str) -> &str {
    match program.rsplit_once('/') {
        Some((_, name)) => name,
        None => program,
    }
}

/// How a file's lines are read.
enum Reading {
    /// A file that runs: every line for a name that means Python, and its commands for the
    /// scripts they hand a shell.
    Commands(Syntax),
    /// A runbook's specification: its preflight declarations.
    Preflight,
    /// A Rust source: its `Command::new` calls.
    Rust,
    /// Prose or data.
    Nothing,
}

/// Which lines of a file that runs are commands.
#[derive(Clone, Copy)]
enum Syntax {
    /// Every line: a shell script, a hook, a Makefile, a justfile, `.envrc`, a Dockerfile, Cargo's
    /// configuration, and a file `make -f` or `just --justfile` is handed.
    Shell,
    /// Every line, flow style and quoted keys alike, of a workflow, an action, or a pre-commit
    /// configuration. An action's `using:` is read too.
    Yaml { action: bool },
    /// The strings, and every identifier that holds `python` on any line.
    Nix,
}

/// What the matchers know of the repository beyond the file in hand.
pub struct Context<'a> {
    /// The shell files the repository may run: the allow-list's three, and the private list's one
    /// where it is read.
    pub permitted: &'a [&'a str],
    /// The tracked files, so a handoff is a finding only where it names one of them.
    pub files: &'a FileSet,
    /// The files `make -f` or `just --justfile` is handed somewhere, read as files that run.
    pub rules: &'a BTreeSet<String>,
}

impl Context<'_> {
    /// The tracked, unpermitted file a handed-off word names, as a path from the root.
    fn unpermitted_file(&self, word: &str) -> Option<String> {
        let path = shell::repository_path(word)?;
        let tracked = matches!(
            self.files.get(&path),
            Some(Entry::File { .. } | Entry::Link { .. })
        );
        (tracked && !self.permitted.contains(&path.as_str())).then_some(path)
    }
}

/// The command lines of a file that runs, numbered from 1.
fn command_lines(content: &[u8], syntax: Syntax) -> Vec<(usize, String)> {
    match syntax {
        Syntax::Nix => nix::string_lines(content),
        Syntax::Shell | Syntax::Yaml { .. } => numbered_lines(content)
            .map(|(number, line)| (number.get(), String::from_utf8_lossy(line).into_owned()))
            .collect(),
    }
}

/// Every tracked file a file that runs hands to `make -f` or `just --justfile`.
pub fn rules_handed(path: &RepoPath, entry: &Entry, context: &Context) -> Vec<String> {
    let Entry::File { content, .. } = entry else {
        return Vec::new();
    };
    let is_permitted = context.permitted.contains(&path.as_str());
    let is_rules = context.rules.contains(path.as_str());
    let Reading::Commands(syntax) = reading(path, &shebang(content), is_permitted, is_rules) else {
        return Vec::new();
    };
    command_lines(content, syntax)
        .iter()
        .flat_map(|(_, line)| shell::handoffs(line))
        .filter_map(|handoff| match handoff {
            shell::Handoff::Rules(word) => shell::repository_path(&word),
            shell::Handoff::Script(_) | shell::Handoff::Input(_) => None,
        })
        .filter(|path| context.files.get(path).is_some())
        .collect()
}

/// The values a YAML line gives `key`, in block style or flow style, its key quoted or not.
fn yaml_values(line: &str, key: &str) -> Vec<String> {
    let mut values = Vec::new();
    for spelled in [
        format!("{key}:"),
        format!("\"{key}\":"),
        format!("'{key}':"),
    ] {
        let mut from = 0;
        while let Some(offset) = line[from..].find(&spelled) {
            let at = from + offset;
            from = at + spelled.len();
            let before = line[..at].chars().next_back();
            if before.is_some_and(|c| !(c.is_whitespace() || matches!(c, '{' | ',' | '-'))) {
                continue;
            }
            let rest = line[from..].trim_start();
            let end = rest.find([',', '}', '#']).unwrap_or(rest.len());
            let value = rest[..end].trim().trim_matches(['"', '\'']);
            if !value.is_empty() {
                values.push(value.to_owned());
            }
        }
    }
    values
}

fn reading(path: &RepoPath, shebang: &Shebang, is_permitted: bool, rules: bool) -> Reading {
    let name = path.file_name().to_ascii_lowercase();
    let yaml = name.ends_with(".yml") || name.ends_with(".yaml");
    let action = matches!(name.as_str(), "action.yml" | "action.yaml");
    let container =
        matches!(name.as_str(), "dockerfile" | "containerfile") || name.ends_with(".dockerfile");
    let shell_line = matches!(
        shebang,
        Shebang::Program(program)
            if ["sh", "bash", "zsh", "dash", "ksh"].iter().any(|shell| program.eq_ignore_ascii_case(shell))
    );
    let cargo_configuration = matches!(name.as_str(), "config.toml" | "config")
        && path.components().rev().nth(1) == Some(".cargo");
    let make = matches!(name.as_str(), "makefile" | "gnumakefile") || name.ends_with(".mk");
    let just = matches!(name.as_str(), "justfile" | ".justfile");
    if is_permitted
        || shell_file_name(&name)
        || shell_line
        || path.is_under(".githooks")
        || cargo_configuration
        || make
        || just
        || name == ".envrc"
        || container
        || rules
    {
        Reading::Commands(Syntax::Shell)
    } else if name.ends_with(".nix") {
        Reading::Commands(Syntax::Nix)
    } else if action {
        Reading::Commands(Syntax::Yaml { action: true })
    } else if yaml
        && (path.is_under(".github/workflows") || name.starts_with(".pre-commit-config."))
    {
        Reading::Commands(Syntax::Yaml { action: false })
    } else if is_runbook_specification(path) {
        Reading::Preflight
    } else if name.ends_with(".rs") {
        Reading::Rust
    } else {
        Reading::Nothing
    }
}

/// A runbook's specification: `runbooks/NAME/specification.html`, at any depth.
fn is_runbook_specification(path: &RepoPath) -> bool {
    let mut last = path.components().rev();
    last.next() == Some("specification.html")
        && last.next().is_some()
        && last.next() == Some("runbooks")
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Each line, numbered from 1, without its line ending.
fn numbered_lines(content: &[u8]) -> impl Iterator<Item = (NonZeroUsize, &[u8])> {
    content
        .split(|byte| *byte == b'\n')
        .enumerate()
        .map(|(index, line)| {
            (
                NonZeroUsize::MIN.saturating_add(index),
                line.strip_suffix(b"\r").unwrap_or(line),
            )
        })
}

/// The lines of a runbook's specification on which a preflight declaration names Python.
///
/// A declaration is `preflight/vN` and the JSON object after it. The object may begin on a later
/// line, after any whitespace and markup, and run over several, as the program that reads a
/// declaration trims the whitespace between them; a name inside the object is what a tool's or
/// program's name would be. The rest of the document is prose.
fn preflight_lines(
    content: &[u8],
    lines: &[(NonZeroUsize, &[u8])],
) -> Vec<(NonZeroUsize, LineRule)> {
    const MARKER: &[u8] = b"preflight/v";
    let mut line_starts = Vec::with_capacity(lines.len());
    let mut position = 0;
    for (_, line) in lines {
        line_starts.push(position);
        position += line.len() + 1;
    }
    let mut found: Vec<(NonZeroUsize, LineRule)> = Vec::new();
    let mut from = 0;
    while let Some(offset) = find(&content[from..], MARKER) {
        let start = from + offset;
        let end = declaration_end(content, start);
        from = end.max(start + 1);
        let mut at = start;
        while let Some((offset, word)) = words::python_word(&content[at..end]) {
            let hit = at + offset;
            at = hit + word.len().max(1);
            let index = line_starts
                .partition_point(|start| *start <= hit)
                .saturating_sub(1);
            if let Some((number, _)) = lines.get(index) {
                if found.last().is_none_or(|(seen, _)| seen != number) {
                    found.push((*number, LineRule::Declares { word }));
                }
            }
        }
    }
    found
}

/// Where the declaration that starts at `start` ends: after the JSON object that follows the
/// marker and its version, past any whitespace and markup between, with its braces balanced
/// outside its strings; or at the end of the marker's line, where no object follows.
fn declaration_end(content: &[u8], start: usize) -> usize {
    let line_end = content[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(content.len(), |offset| start + offset);
    let mut at = start + b"preflight/v".len();
    at += content[at..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    loop {
        match content.get(at) {
            Some(byte) if byte.is_ascii_whitespace() => at += 1,
            Some(b'<') => match content[at..].iter().position(|byte| *byte == b'>') {
                Some(offset) => at += offset + 1,
                None => return line_end,
            },
            Some(b'{') => break,
            _ => return line_end,
        }
    }
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, byte) in content[at..].iter().enumerate() {
        match (in_string, escaped, byte) {
            (true, true, _) => escaped = false,
            (true, false, b'\\') => escaped = true,
            (true, false, b'"') => in_string = false,
            (true, false, _) => {}
            (false, _, b'"') => in_string = true,
            (false, _, b'{') => depth += 1,
            (false, _, b'}') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return at + offset + 1;
                }
            }
            (false, _, _) => {}
        }
    }
    content.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four permitted shell files, as the repository's lists name them.
    const PERMITTED: [&str; 4] = [
        "scripts/check",
        "scripts/install-hooks",
        ".githooks/pre-push",
        "publish/export",
    ];

    fn path(text: &str) -> RepoPath {
        RepoPath::parse(text).expect("a path")
    }

    fn file(content: &str) -> Entry {
        Entry::File {
            executable: false,
            content: content.as_bytes().to_vec(),
        }
    }

    /// The tracked files a handoff in these tests may name, the permitted four among them.
    const TRACKED: [&str; 9] = [
        "scripts/check",
        "scripts/install-hooks",
        ".githooks/pre-push",
        "publish/export",
        "scripts/lib.bash",
        "tools/tidy",
        "ci/steps",
        "ci/rules",
        "tools/x.pl",
    ];

    fn tracked() -> FileSet {
        let files = TRACKED.iter().map(|at| (path(at), file(""))).collect();
        FileSet::new(
            super::super::tracked::Origin::Index(std::path::PathBuf::new()),
            files,
        )
    }

    fn rules_with(at: &str, entry: &Entry, handed: &[&str]) -> Vec<String> {
        let files = tracked();
        let rules = handed.iter().map(|path| (*path).to_owned()).collect();
        let context = Context {
            permitted: &PERMITTED,
            files: &files,
            rules: &rules,
        };
        findings(&path(at), entry, &context)
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    fn rules(at: &str, entry: &Entry) -> Vec<String> {
        rules_with(at, entry, &[])
    }

    const BASH_SCRIPT: &str = "#!/usr/bin/env bash\nset -euo pipefail\n";

    #[test]
    fn python_and_shell_file_names_are_refused_by_name_in_any_case() {
        for name in [
            "tidy.py",
            "tidy.PY",
            "tidy.Py",
            "gui.pyw",
            "types.pyi",
            "module.cpython-313.pyc",
            "old.pyo",
            "app.pyz",
            "notebook.ipynb",
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
            "requirements.txt",
            "requirements-dev.txt",
            "Pipfile",
            "Pipfile.lock",
            "poetry.lock",
            "uv.lock",
            "tox.ini",
            ".python-version",
        ] {
            assert!(python_file_name(name), "{name}");
        }
        for name in [
            "check",
            "README.md",
            "python.md",
            "requirements.md",
            "copy.pyx",
        ] {
            assert!(!python_file_name(name), "{name}");
        }
        for name in [
            "release.sh",
            "lib.bash",
            "x.zsh",
            "x.KSH",
            "x.dash",
            "TIDY.SH",
        ] {
            assert!(shell_file_name(name), "{name}");
        }
        assert!(!shell_file_name("crush"));
    }

    #[test]
    fn a_first_line_names_its_program_directly_or_through_env() {
        let named = |line: &str| shebang(line.as_bytes());
        let program = |name: &str| Shebang::Program(name.into());
        assert_eq!(named("#!/usr/bin/env python3\nprint()"), program("python3"));
        assert_eq!(named("#!/usr/bin/python"), program("python"));
        assert_eq!(named("#! /usr/bin/env  python3.13"), program("python3.13"));
        assert_eq!(named("#!/usr/bin/env -S python3 -u"), program("python3"));
        assert_eq!(named("#!/usr/bin/env -Spython3"), program("python3"));
        assert_eq!(
            named("#!/usr/bin/env --split-string=python3"),
            program("python3")
        );
        assert_eq!(
            named("#!/usr/bin/env -i PYTHONPATH=x python3"),
            program("python3")
        );
        assert_eq!(named("#!/usr/bin/env -u HOME python3"), program("python3"));
        assert_eq!(
            named("#!/usr/bin/env -P /opt/bin python3"),
            program("python3")
        );
        assert_eq!(named("#!/usr/bin/env bash"), program("bash"));
        assert_eq!(named("#!/bin/sh -e"), program("sh"));
        assert_eq!(named("# a comment\n#!/usr/bin/env python3"), Shebang::None);
        assert_eq!(named("#![forbid(unsafe_code)]\n"), Shebang::None);
        assert_eq!(named(""), Shebang::None);
        // Quotes, backslashes, an option `env` does not know, or no program: refused, not guessed.
        for unreadable in [
            "#!/usr/bin/env -S \"python3\"",
            "#!/usr/bin/env -S 'python3'",
            "#!/usr/bin/env -S py\\thon3",
            "#!/usr/bin/env --frobnicate python3",
            "#!/usr/bin/env",
            "#!",
        ] {
            assert_eq!(named(unreadable), Shebang::Unreadable, "{unreadable}");
        }
    }

    #[test]
    fn a_workflow_names_python_on_any_line_comments_included() {
        let workflow = file(
            "jobs:\n  test:\n    # Python below\n    steps:\n      - run: python3 publish/test\n",
        );
        assert_eq!(
            rules(".github/workflows/publish.yml", &workflow),
            [
                ".github/workflows/publish.yml:3: names Python",
                ".github/workflows/publish.yml:5: names python3",
            ]
        );
        let action = file("runs:\n  steps:\n    - uses: actions/setup-python@v5\n");
        assert_eq!(
            rules(".github/actions/xtask/action.yaml", &action),
            [".github/actions/xtask/action.yaml:3: names setup-python"]
        );
        // The same text outside the files that run is prose.
        assert!(rules("docs/workflow.yml", &workflow).is_empty());
        assert!(rules("README.md", &file("```sh\npython3 -m http.server\n```\n")).is_empty());
    }

    #[test]
    fn other_runners_of_python_are_named() {
        for (line, word) in [
            ("      - run: pypy3 x", "pypy3"),
            ("      - run: python3.13t x", "python3.13t"),
            ("      - run: python3m x", "python3m"),
            ("      - run: pip install x", "pip"),
            ("      - run: pipx run black", "pipx"),
            ("      - run: uv run --python 3.12 x", "uv"),
            ("      - run: uvx ruff", "uvx"),
            ("      - run: Python3 x", "Python3"),
        ] {
            assert_eq!(
                rules(".github/workflows/x.yml", &file(line)),
                [format!(".github/workflows/x.yml:1: names {word}")],
                "{line}"
            );
        }
    }

    #[test]
    fn files_that_run_are_read_line_by_line_whatever_their_first_line() {
        let cases = [
            ("tools/build.sh", "exec python3 x\n"),
            ("tools/lib.bash", "python3 x\n"),
            (".githooks/post-commit", "python3 hook\n"),
            (".cargo/config.toml", "[alias]\nlint = \"!python3 lint\"\n"),
            ("publish/.cargo/config.toml", "python3\n"),
            ("flake.nix", "packages = [ pkgs.python3 ];\n"),
            ("Makefile", "lint:\n\tpython3 lint\n"),
            ("tools/rules.mk", "python3\n"),
            ("justfile", "python3\n"),
            (".envrc", "layout python3\n"),
            (
                "sub/.pre-commit-config.yaml",
                "repos:\n  - hooks:\n      - entry: python3 x\n",
            ),
        ];
        for (at, content) in cases {
            let said = rules(at, &file(content));
            assert!(
                said.iter().any(|line| line.contains(": names python3")),
                "{at}: {said:#?}"
            );
        }
        assert!(rules("crates/x/config.toml", &file("python3\n")).is_empty());
    }

    #[test]
    fn a_nix_identifier_that_holds_python_is_named() {
        for (line, word) in [
            ("packages = [ pkgs.python313 ];", "python313"),
            ("packages = [ pkgs.python3Minimal ];", "python3Minimal"),
            ("(pkgs.python312.withPackages (p: [ ]))", "python312"),
            ("packages = [ pkgs.pypy3 ];", "pypy3"),
        ] {
            assert_eq!(
                rules("flake.nix", &file(line)),
                [format!("flake.nix:1: names {word}")]
            );
        }
    }

    /// The reviewer's first ways around the ban: a library a permitted shell file sources, a
    /// script a workflow hands bash, and a plain shell file under `scripts/`.
    #[test]
    fn a_script_a_file_that_runs_hands_a_shell_is_one_of_the_four() {
        let check = file(&format!(
            "{BASH_SCRIPT}. scripts/lib.bash\nsource ./scripts/check\n"
        ));
        assert_eq!(
            rules("scripts/check", &check),
            [
                "scripts/check:1: begins with #!, and it is none of the four permitted shell files",
                "scripts/check:3: runs scripts/lib.bash as a script, and it is none of the four permitted shell files",
            ]
        );
        let workflow = file("    steps:\n      - run: bash tools/tidy\n      - run: bash -x \"$block\"\n      - run: scripts/check\n");
        assert_eq!(
            rules(".github/workflows/check.yml", &workflow),
            [".github/workflows/check.yml:2: runs tools/tidy as a script, and it is none of the four permitted shell files"]
        );
        let plain = rules("scripts/release.sh", &file("echo release\n"));
        assert_eq!(
            plain,
            [
                "scripts/release.sh: a shell script's file name, and it is none of the four permitted shell files",
                "scripts/release.sh: a file under scripts/, which holds scripts/check and scripts/install-hooks alone",
            ]
        );
        // A permitted file's first line is a finding the allow-list excuses, and nothing else is.
        assert_eq!(
            rules("scripts/check", &file(BASH_SCRIPT)),
            ["scripts/check:1: begins with #!, and it is none of the four permitted shell files"]
        );
    }

    /// The second review's ways around the ban: a stage string of `scripts/check`, a handoff the
    /// command-position reading missed, and a shell reading its script from standard input.
    #[test]
    fn a_handoff_is_read_wherever_it_stands() {
        let stage = file(&format!(
            "{BASH_SCRIPT}steps=(\n  \"tidy:bash ci/steps\"\n  \"format:cargo fmt --all -- --check\"\n)\n"
        ));
        assert!(rules("scripts/check", &stage)
            .contains(&String::from("scripts/check:4: runs ci/steps as a script, and it is none of the four permitted shell files")));
        for line in [
            "timeout 600 bash ci/steps",
            "nix develop -c bash ci/steps",
            "sudo -u runner bash ci/steps",
            "cat ci/steps | bash",
            "bash -s < ci/steps",
            "      - { name: x, run: bash ci/steps }",
            "        \"run\": \"bash ci/steps\"",
            "        shell: bash ci/steps {0}",
        ] {
            let workflow = file(&format!("jobs:\n{line}\n"));
            assert_eq!(
                rules(".github/workflows/x.yml", &workflow),
                [".github/workflows/x.yml:2: runs ci/steps as a script, and it is none of the four permitted shell files"],
                "{line}"
            );
        }
        let next_line = file("      - run:\n          bash ci/steps\n");
        assert_eq!(rules(".github/workflows/x.yml", &next_line).len(), 1);
        assert_eq!(
            rules("tools/Makefile", &file("lint:\n\tperl tools/x.pl\n")),
            ["tools/Makefile:2: runs tools/x.pl as a script, and it is none of the four permitted shell files"]
        );
    }

    /// What `make -f` or `just --justfile` is handed is read as a Makefile, wherever it lives.
    #[test]
    fn a_file_handed_to_make_is_read_as_one() {
        let rules_file = file("lint:\n\tpython3 lint\n");
        assert!(rules("ci/rules", &rules_file).is_empty());
        assert_eq!(
            rules_with("ci/rules", &rules_file, &["ci/rules"]),
            ["ci/rules:2: names python3"]
        );
        let files = tracked();
        let empty = BTreeSet::new();
        let context = Context {
            permitted: &PERMITTED,
            files: &files,
            rules: &empty,
        };
        let workflow =
            file("      - run: make -f ci/rules lint\n      - run: just --justfile ./ci/rules\n");
        assert_eq!(
            rules_handed(&path(".github/workflows/x.yml"), &workflow, &context),
            ["ci/rules", "ci/rules"]
        );
    }

    /// A local action is read like a workflow, wherever it lives, and the workflow that uses one
    /// outside `.github/` is refused.
    #[test]
    fn a_local_action_is_read_and_kept_under_github() {
        let action = file(
            "runs:\n  using: composite\n  steps:\n    - run: python3 -c 1\n      shell: bash\n",
        );
        assert_eq!(
            rules("ci/tidy/action.yml", &action),
            ["ci/tidy/action.yml:4: names python3"]
        );
        let python_shell =
            file("runs:\n  using: composite\n  steps:\n    - run: print(1)\n      shell: python\n");
        assert_eq!(
            rules("ci/tidy/action.yaml", &python_shell),
            ["ci/tidy/action.yaml:5: names python"]
        );
        let node = file("runs:\n  using: node20\n  main: index.js\n");
        assert_eq!(
            rules(".github/actions/x/action.yml", &node),
            [".github/actions/x/action.yml:2: an action that runs node20, a program no Cargo workspace builds"]
        );
        let workflow = file("    steps:\n      - uses: ./ci/tidy\n      - uses: ./.github/actions/xtask\n      - { uses: './ci/other' }\n      - uses: actions/checkout@v4\n");
        assert_eq!(
            rules(".github/workflows/x.yml", &workflow),
            [
                ".github/workflows/x.yml:2: uses the local action ./ci/tidy, outside .github/, where the repository's actions live",
                ".github/workflows/x.yml:4: uses the local action ./ci/other, outside .github/, where the repository's actions live",
            ]
        );
        assert_eq!(
            rules(
                ".github/workflows/x.yml",
                &file("      - uses: astral-sh/setup-uv@v5\n")
            ),
            [".github/workflows/x.yml:1: names setup-uv"]
        );
    }

    /// The false positives the handoff rule had: a path outside the repository, a Nix attribute or
    /// list, and a YAML file under `.github/` that is prose.
    #[test]
    fn what_names_no_tracked_file_or_runs_nothing_is_no_finding() {
        let workflow = file("      - run: . /etc/os-release\n      - run: source ~/.cargo/env\n      - run: bash -x \"$block\"\n      - run: bash --version\n");
        assert!(rules(".github/workflows/x.yml", &workflow).is_empty());
        let flake = file("{\n  sh = pkgs.bash;\n  inputs = [\n    bash jq\n  ];\n  check = ''\n    ./scripts/check\n  '';\n}\n");
        assert!(rules("flake.nix", &flake).is_empty());
        let issue =
            file("body:\n  - type: input\n    attributes:\n      label: Which Python version?\n");
        assert!(rules(".github/ISSUE_TEMPLATE/bug.yml", &issue).is_empty());
        assert!(rules(
            ".github/dependabot.yml",
            &file("  - package-ecosystem: pip\n")
        )
        .is_empty());
    }

    /// The reviewer's second: a permitted shell file is read whatever its first line, and must be
    /// bash.
    #[test]
    fn a_permitted_shell_file_is_bash_and_is_read_whatever_its_first_line() {
        assert_eq!(
            rules("scripts/check", &file("#!/usr/bin/env zsh\npython3 -c 1\n")),
            [
                "scripts/check:1: a permitted shell file whose first line is not #!/usr/bin/env bash",
                "scripts/check:1: begins with #!, and it is none of the four permitted shell files",
                "scripts/check:2: names python3",
            ]
        );
        assert_eq!(
            rules(
                "scripts/install-hooks",
                &file("#!/usr/bin/env -S \"python3\"\nprint('installed')\n")
            ),
            [
                "scripts/install-hooks:1: a first line whose program this stage cannot read, which it refuses rather than guess",
                "scripts/install-hooks:1: a permitted shell file whose first line is not #!/usr/bin/env bash",
                "scripts/install-hooks:1: begins with #!, and it is none of the four permitted shell files",
                "scripts/install-hooks:1: names python3",
            ]
        );
        let dash = rules(".githooks/pre-push", &file("#!/bin/dash\n"));
        assert!(dash[0].contains("is not #!/usr/bin/env bash"), "{dash:#?}");
    }

    #[test]
    fn a_runbook_preflight_declaration_is_read_and_its_prose_is_not() {
        let specification = file(concat!(
            "<p>A run MUST start with <code>python3</code> resolving to the shell.</p>\n",
            "<p data-field=\"note\">preflight/v1 {\"kind\":\"toolchain\",\"manifest\":\"flake.nix\",",
            "\"shell\":\"default\",\"tools\":[\"cargo\",\"rustc\",\"python3\"]}</p>\n",
            "<p data-field=\"note\">preflight/v1 {\"kind\":\"tool\",\n",
            "  \"program\":\"python3\"}</p>\n",
            "<p data-field=\"note\">preflight/v1 {\"kind\":\"toolchain\",\"tools\":[\"cargo\"]}</p> python3\n",
            "<p data-field=\"note\">preflight/v1\n",
            "{\"kind\":\"tool\",\"program\":\"python3\"}</p>\n",
            "<p data-field=\"note\">preflight/v2 <span>{\"kind\":\"tool\",\"program\":\"pypy3\"}</span></p>\n",
        ));
        assert_eq!(
            rules("htmlspec/runbooks/release/specification.html", &specification),
            [
                "htmlspec/runbooks/release/specification.html:2: a preflight declaration names python3",
                "htmlspec/runbooks/release/specification.html:4: a preflight declaration names python3",
                "htmlspec/runbooks/release/specification.html:7: a preflight declaration names python3",
                "htmlspec/runbooks/release/specification.html:8: a preflight declaration names pypy3",
            ]
        );
        // A run's record, a change's specification, and a design are history and prose.
        assert!(rules("htmlspec/runbooks/publish/runs/v1.3.0.html", &specification).is_empty());
        assert!(rules(
            "htmlspec/changes/archive/x/specification.html",
            &specification
        )
        .is_empty());
    }

    #[test]
    fn a_rust_command_that_runs_python_is_read_and_a_string_is_not() {
        let source = file(concat!(
            "fn main() {\n",
            "    let sample = \"python3\";\n",
            "    Command::new(\"cargo\").arg(\"python3\");\n",
            "    Command::new(\"python3\").arg(\"x\");\n",
            "    Command::new(\"env\").arg(\"python3\");\n",
            "}\n",
        ));
        assert_eq!(
            rules("crates/x/src/main.rs", &source),
            [
                "crates/x/src/main.rs:4: Command::new runs python3",
                "crates/x/src/main.rs:5: Command::new runs python3",
            ]
        );
    }

    #[test]
    fn a_program_is_found_by_its_first_line_or_its_mode() {
        let shebang = file("#!/usr/bin/env bash\necho\n");
        assert_eq!(
            rules("tools/tidy", &shebang)[0],
            "tools/tidy:1: begins with #!, and it is none of the four permitted shell files"
        );
        let executable = Entry::File {
            executable: true,
            content: b"echo no first line\n".to_vec(),
        };
        assert_eq!(
            rules("tools/tidy", &executable),
            ["tools/tidy: carries the executable bit, and it is none of the four permitted shell files"]
        );
        let python = file("#!/usr/bin/env python3\nprint()\n");
        assert_eq!(
            rules("tools/tidy", &python),
            [
                "tools/tidy:1: a first line that runs python3",
                "tools/tidy:1: begins with #!, and it is none of the four permitted shell files",
            ]
        );
        assert!(rules("notes.md", &file("# not a program\n")).is_empty());
        assert!(rules("crates/x/src/lib.rs", &file("#![forbid(unsafe_code)]\n")).is_empty());
    }

    #[test]
    fn a_link_out_or_to_an_interpreter_and_a_submodule_are_refused() {
        let link = |target: &str| Entry::Link {
            target: target.as_bytes().to_vec(),
        };
        assert_eq!(
            rules("tools/py", &link("/usr/bin/python3")),
            [
                "tools/py: a link to /usr/bin/python3, outside the repository",
                "tools/py: a link to /usr/bin/python3, which names an interpreter",
            ]
        );
        assert_eq!(
            rules("tools/py", &link("../../outside")),
            ["tools/py: a link to ../../outside, outside the repository"]
        );
        assert_eq!(
            rules("tools/py", &link("vendor/python3")),
            ["tools/py: a link to vendor/python3, which names an interpreter"]
        );
        assert!(rules("docs/latest", &link("../crates/README.md")).is_empty());
        assert_eq!(
            rules("tool.py", &link("tool")),
            ["tool.py: a Python file name"]
        );
        assert_eq!(
            rules("vendor/sub", &Entry::Submodule),
            ["vendor/sub: a submodule, whose code no Cargo workspace here builds"]
        );
    }
}
