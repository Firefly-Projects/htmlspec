//! What a command line hands to a program as a script.
//!
//! Wherever it stands on a line of a file that runs, a shell (`bash`, `sh`, `zsh`, `dash`, `ksh`),
//! `source`, `.`, or another interpreter (`perl`, `ruby`, `node`, `deno`, `bun`, `php`, `lua`) is
//! read for the word it runs as a script, after its options. Nothing here asks where a command
//! starts: `timeout 600 bash X`, `nix develop -c bash X`, and `sudo -u runner bash X` hand `X` to
//! bash as surely as `bash X` does. A shell that reads its script from standard input, after `|`,
//! with `-s`, with a `<` redirection, or with no script named, is handed whatever file the line
//! names. `make -f X` and `just --justfile X` hand `X` to make and just, which read it as a
//! Makefile or a justfile.
//!
//! The words are read as a shell reads them: quotes group and are removed, a backslash escapes,
//! and an unquoted `#` that starts a word starts a comment. A quoted word that holds a space is
//! read as a command line of its own, after any `name:` before it, so a shell's `-c` argument, an
//! `eval`'s, and a stage string of `scripts/check` such as `"tidy:bash ci/steps"` are read.
//!
//! This module names candidates. Which of them are files of the repository, and which of those are
//! permitted, the matchers decide, so a word that names no tracked file, such as `/etc/os-release`,
//! `~/.cargo/env`, or `$block`, is never a finding.

/// A word of a command line, its quotes removed.
#[derive(Debug, PartialEq, Eq)]
enum Token {
    Word(String),
    /// `|`, after which a command reads what the one before it wrote.
    Pipe,
    /// Anything else that ends one command and lets the next begin.
    Separator,
}

/// What a line hands to a program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Handoff {
    /// The word a shell, `source`, `.`, or an interpreter runs as its script.
    Script(String),
    /// A word on a line where a shell reads its script from standard input.
    Input(String),
    /// The word `make -f` or `just --justfile` reads as its rules.
    Rules(String),
}

/// The shells, and the other interpreters, whose first argument is a script.
const SHELLS: [&str; 5] = ["bash", "sh", "zsh", "dash", "ksh"];
const INTERPRETERS: [&str; 7] = ["perl", "ruby", "node", "deno", "bun", "php", "lua"];

/// The words of one command line.
fn tokens(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut single = false;
    let mut double = false;
    // The quoting each open `$(` interrupted, so its `)` restores it.
    let mut substitutions: Vec<bool> = Vec::new();
    let mut chars = line.chars().peekable();
    let finish = |tokens: &mut Vec<Token>, word: &mut String, started: &mut bool| {
        if *started {
            tokens.push(Token::Word(std::mem::take(word)));
            *started = false;
        }
    };
    while let Some(c) = chars.next() {
        if single {
            if c == '\'' {
                single = false;
            } else {
                word.push(c);
            }
            continue;
        }
        let opens_substitution = c == '$' && chars.peek() == Some(&'(');
        if double {
            match c {
                '"' => double = false,
                '\\' => word.extend(chars.next()),
                '`' => {
                    finish(&mut tokens, &mut word, &mut started);
                    tokens.push(Token::Separator);
                }
                _ if opens_substitution => {
                    chars.next();
                    finish(&mut tokens, &mut word, &mut started);
                    tokens.push(Token::Separator);
                    substitutions.push(true);
                    double = false;
                }
                _ => word.push(c),
            }
            continue;
        }
        match c {
            _ if c.is_whitespace() => finish(&mut tokens, &mut word, &mut started),
            '\'' => {
                single = true;
                started = true;
            }
            '"' => {
                double = true;
                started = true;
            }
            '\\' => {
                word.extend(chars.next());
                started = true;
            }
            '#' if !started => break,
            '|' => {
                finish(&mut tokens, &mut word, &mut started);
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::Separator);
                } else {
                    tokens.push(Token::Pipe);
                }
            }
            ';' | '&' | '(' | '`' | '{' | '}' | ',' => {
                finish(&mut tokens, &mut word, &mut started);
                tokens.push(Token::Separator);
            }
            ')' => {
                finish(&mut tokens, &mut word, &mut started);
                tokens.push(Token::Separator);
                if let Some(quoted) = substitutions.pop() {
                    double = quoted;
                    started = quoted;
                }
            }
            _ if opens_substitution => {
                chars.next();
                finish(&mut tokens, &mut word, &mut started);
                tokens.push(Token::Separator);
                substitutions.push(false);
            }
            _ => {
                word.push(c);
                started = true;
            }
        }
    }
    finish(&mut tokens, &mut word, &mut started);
    tokens
}

fn basename(word: &str) -> &str {
    match word.rsplit_once('/') {
        Some((_, name)) => name,
        None => word,
    }
}

/// A quoted word's text without a `name:` before it, as a stage string of `scripts/check` has.
fn without_label(word: &str) -> &str {
    match word.split_once(':') {
        Some((label, rest))
            if !label.is_empty()
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')) =>
        {
            rest
        }
        _ => word,
    }
}

/// Every handoff on a command line.
pub fn handoffs(line: &str) -> Vec<Handoff> {
    let mut found = Vec::new();
    scan(&tokens(line), &mut found);
    let mut seen = Vec::new();
    found.retain(|handoff| {
        let new = !seen.contains(handoff);
        if new {
            seen.push(handoff.clone());
        }
        new
    });
    found
}

fn word_at(tokens: &[Token], index: usize) -> Option<&str> {
    match tokens.get(index) {
        Some(Token::Word(word)) => Some(word),
        _ => None,
    }
}

fn scan(tokens: &[Token], found: &mut Vec<Handoff>) {
    for (index, token) in tokens.iter().enumerate() {
        let Token::Word(word) = token else {
            continue;
        };
        if word.contains(char::is_whitespace) {
            scan(&self::tokens(without_label(word)), found);
        }
        let name = basename(word).to_ascii_lowercase();
        let after_pipe = index > 0 && matches!(tokens.get(index - 1), Some(Token::Pipe));
        if SHELLS.contains(&name.as_str()) {
            match shell_script(tokens, index + 1, found) {
                Some(script) if !after_pipe => found.push(Handoff::Script(script)),
                _ => input(tokens, index, found),
            }
        } else if name == "source" || word == "." {
            if let Some(target) = word_at(tokens, index + 1) {
                found.push(Handoff::Script(target.to_owned()));
            }
        } else if INTERPRETERS.contains(&name.as_str()) {
            let mut at = index + 1;
            while let Some(argument) = word_at(tokens, at) {
                if matches!(argument, "-e" | "-E" | "--eval" | "-r") {
                    at += 2;
                } else if argument.starts_with('-') {
                    at += 1;
                } else {
                    found.push(Handoff::Script(argument.to_owned()));
                    break;
                }
            }
        } else if name == "make" || name == "gmake" || name == "just" {
            rules(tokens, index + 1, &name, found);
        }
    }
}

/// The script a shell's arguments from `index` name, or nothing where it reads standard input or
/// runs a `-c` command line, which is read here.
fn shell_script(tokens: &[Token], mut index: usize, found: &mut Vec<Handoff>) -> Option<String> {
    while let Some(argument) = word_at(tokens, index) {
        index += 1;
        let short = argument
            .strip_prefix('-')
            .or_else(|| argument.strip_prefix('+'))
            .filter(|cluster| !cluster.starts_with('-'));
        match argument {
            "--" => return word_at(tokens, index).map(str::to_owned),
            "-o" | "+o" | "-O" | "+O" | "--rcfile" | "--init-file" => index += 1,
            _ if argument.starts_with('<') => return None,
            _ if short.is_some_and(|cluster| cluster.contains('c')) => {
                if let Some(command) = word_at(tokens, index) {
                    scan(&self::tokens(command), found);
                }
                return None;
            }
            _ if short.is_some_and(|cluster| cluster.contains('s')) => return None,
            _ if argument.starts_with('-') || argument.starts_with('+') => {}
            _ => return Some(argument.to_owned()),
        }
    }
    None
}

/// A shell reads its script from standard input: every word on the line may be what it reads.
fn input(tokens: &[Token], shell: usize, found: &mut Vec<Handoff>) {
    for (index, token) in tokens.iter().enumerate() {
        if let Token::Word(word) = token {
            if index != shell {
                let file = word.trim_start_matches('<');
                if !file.is_empty() {
                    found.push(Handoff::Input(file.to_owned()));
                }
            }
        }
    }
}

/// The file `make -f`, `make --file`, `make --makefile`, `just -f`, or `just --justfile` reads.
fn rules(tokens: &[Token], mut index: usize, program: &str, found: &mut Vec<Handoff>) {
    let long: &[&str] = if program == "just" {
        &["--justfile"]
    } else {
        &["--file", "--makefile"]
    };
    while let Some(argument) = word_at(tokens, index) {
        index += 1;
        if argument == "-f" || long.contains(&argument) {
            if let Some(file) = word_at(tokens, index) {
                found.push(Handoff::Rules(file.to_owned()));
            }
            index += 1;
        } else if let Some(file) = argument.strip_prefix("-f").filter(|f| !f.is_empty()) {
            found.push(Handoff::Rules(file.to_owned()));
        } else if let Some(file) = long
            .iter()
            .find_map(|option| argument.strip_prefix(option)?.strip_prefix('='))
        {
            found.push(Handoff::Rules(file.to_owned()));
        }
    }
}

/// The word as a path from the repository's root, where it is one: relative, neither from the
/// home directory nor computed by the shell, and never above the root. `./` and `..` are resolved.
pub fn repository_path(word: &str) -> Option<String> {
    if word.is_empty()
        || word.starts_with('/')
        || word.starts_with('~')
        || word.contains(['$', '`', '*', '?', '['])
    {
        return None;
    }
    let mut parts: Vec<&str> = Vec::new();
    for part in word.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scripts(line: &str) -> Vec<String> {
        handoffs(line)
            .into_iter()
            .filter_map(|handoff| match handoff {
                Handoff::Script(word) => Some(word),
                _ => None,
            })
            .collect()
    }

    fn inputs(line: &str) -> Vec<String> {
        handoffs(line)
            .into_iter()
            .filter_map(|handoff| match handoff {
                Handoff::Input(word) => Some(word),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_shell_or_a_source_names_its_script_wherever_it_stands() {
        for (line, script) in [
            ("bash tools/tidy", "tools/tidy"),
            (". scripts/lib.bash", "scripts/lib.bash"),
            ("source ./scripts/lib.bash", "./scripts/lib.bash"),
            ("set -e; sh -eu 'tools/x'", "tools/x"),
            ("HOME=\"$home\" /bin/zsh -x tools/a", "tools/a"),
            ("bash -o pipefail tools/d", "tools/d"),
            ("bash -- tools/e", "tools/e"),
            ("x=$(bash tools/f)", "tools/f"),
            ("echo \"$(sh tools/g)\"", "tools/g"),
            ("bash -c 'bash tools/h'", "tools/h"),
            ("eval \"bash tools/i\"", "tools/i"),
            ("BASH tools/j", "tools/j"),
            ("timeout 600 bash ci/steps", "ci/steps"),
            ("nix develop -c bash ci/steps", "ci/steps"),
            ("sudo -u runner bash ci/steps", "ci/steps"),
            ("- { name: x, run: bash ci/steps }", "ci/steps"),
            ("\"run\": \"bash ci/steps\"", "ci/steps"),
            ("shell: bash ci/steps {0}", "ci/steps"),
            ("  \"tidy:bash ci/steps\"", "ci/steps"),
            ("perl -w tools/x.pl", "tools/x.pl"),
            ("node tools/x.js", "tools/x.js"),
        ] {
            assert_eq!(scripts(line), [script], "{line}");
        }
    }

    #[test]
    fn a_shell_reading_standard_input_is_handed_the_line_s_files() {
        assert!(inputs("cat ci/steps | bash").contains(&String::from("ci/steps")));
        assert!(inputs("bash -s < ci/steps").contains(&String::from("ci/steps")));
        assert!(inputs("bash <ci/steps").contains(&String::from("ci/steps")));
        assert!(inputs("bash -x -s < ci/steps").contains(&String::from("ci/steps")));
        assert!(scripts("cat ci/steps | bash").is_empty());
    }

    #[test]
    fn make_and_just_are_handed_their_rules() {
        let rules = |line: &str| -> Vec<Handoff> {
            handoffs(line)
                .into_iter()
                .filter(|handoff| matches!(handoff, Handoff::Rules(_)))
                .collect()
        };
        for line in [
            "make -f ci/rules",
            "make -fci/rules lint",
            "make --file=ci/rules",
            "gmake --makefile ci/rules",
            "just --justfile ci/rules lint",
            "just -f ci/rules",
        ] {
            assert_eq!(
                rules(line),
                [Handoff::Rules(String::from("ci/rules"))],
                "{line}"
            );
        }
        assert!(rules("make lint").is_empty());
    }

    #[test]
    fn prose_options_and_comments_name_no_script() {
        for line in [
            "bash --version | head -n 1",
            "name = \"source\";",
            "# runs . scripts/lib.bash and bash tools/x",
            "cat file.sh",
            "nix develop -c scripts/check",
            "cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\"",
        ] {
            assert_eq!(scripts(line), Vec::<String>::new(), "{line}");
        }
    }

    #[test]
    fn a_repository_path_is_relative_and_stays_inside() {
        assert_eq!(
            repository_path("./scripts/check").as_deref(),
            Some("scripts/check")
        );
        assert_eq!(
            repository_path("ci/../scripts/check").as_deref(),
            Some("scripts/check")
        );
        for outside in [
            "/etc/os-release",
            "~/.cargo/env",
            "$block",
            "\"$block\"",
            "../outside",
            "ci/*.sh",
            "",
            ".",
        ] {
            assert_eq!(repository_path(outside), None, "{outside}");
        }
    }
}
