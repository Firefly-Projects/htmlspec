//! What a Rust source runs through `Command::new`: a program named by a string literal, and, when
//! that program is a wrapper that runs another, such as `env`, `sh`, or `nix`, the string literals
//! the rest of the statement hands it, where `python3` among the arguments of `env` is as much a
//! run of Python as `python3` itself. This documentation spells no call out in full, because the
//! stage reads this file too.
//!
//! The statement runs from the call to the `;` that ends it, or to the `}` that closes the block
//! it is the tail of, whichever comes first outside the brackets the statement opens, its string
//! and character literals, and its comments. So a closure's `;` does not end it early, and a tail
//! expression does not run on into the code after it.
//!
//! A program named by a variable or a constant is not read, as an interpreter named through a
//! variable is not: the flake's missing interpreter holds that case in the sandbox.

use super::words::python_word;

/// Programs that run another program their arguments name.
const WRAPPERS: [&str; 10] = [
    "env", "sh", "bash", "zsh", "dash", "ksh", "nix", "uv", "xargs", "timeout",
];

/// One `Command::new` that runs Python: where it starts, and the name it runs.
pub struct Run {
    pub at: usize,
    pub word: String,
}

/// Every `Command::new` in a Rust source that runs Python, directly or through a wrapper.
pub fn commands_running_python(content: &[u8]) -> Vec<Run> {
    const CALL: &[u8] = b"Command::new";
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = find(&content[from..], CALL) {
        let at = from + offset;
        from = at + CALL.len();
        let statement = Statement::read(content, from);
        let Some(program) = statement.program else {
            continue;
        };
        if let Some((_, word)) = python_word(program) {
            found.push(Run { at, word });
            continue;
        }
        let name = String::from_utf8_lossy(program).to_ascii_lowercase();
        let name = name.rsplit('/').next().unwrap_or_default();
        if !WRAPPERS.contains(&name) {
            continue;
        }
        let word = statement
            .literals
            .iter()
            .find_map(|literal| python_word(literal).map(|(_, word)| word));
        if let Some(word) = word {
            found.push(Run { at, word });
        }
    }
    found
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// The string literals of one statement that starts with `Command::new`.
struct Statement<'a> {
    /// The literal that is the call's own argument, where its argument is one.
    program: Option<&'a [u8]>,
    /// Every string literal before the statement ends, the program's among them.
    literals: Vec<&'a [u8]>,
}

impl<'a> Statement<'a> {
    fn read(content: &'a [u8], from: usize) -> Self {
        let program = string_argument(content, from);
        let mut literals = Vec::new();
        let mut depth = 0usize;
        let mut at = from;
        while let Some(&byte) = content.get(at) {
            let rest = &content[at..];
            if rest.starts_with(b"//") {
                at += rest.iter().position(|b| *b == b'\n').unwrap_or(rest.len());
                continue;
            }
            if rest.starts_with(b"/*") {
                at += find(&rest[2..], b"*/").map_or(rest.len(), |end| end + 4);
                continue;
            }
            let previous_is_identifier = content[..at]
                .last()
                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
            if !previous_is_identifier {
                if let Some((literal, end)) = string_literal(content, at) {
                    literals.push(literal);
                    at = end;
                    continue;
                }
            }
            match byte {
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' if depth == 0 => break,
                b')' | b']' | b'}' => depth -= 1,
                b';' if depth == 0 => break,
                b'\'' => at += char_literal(rest) - 1,
                _ => {}
            }
            at += 1;
        }
        Self { program, literals }
    }
}

/// The literal that is `Command::new`'s own argument, where the call's text continues at `from`.
fn string_argument(content: &[u8], from: usize) -> Option<&[u8]> {
    let skip_space = |at: usize| {
        at + content[at..]
            .iter()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count()
    };
    let open = skip_space(from);
    if content.get(open) != Some(&b'(') {
        return None;
    }
    string_literal(content, skip_space(open + 1)).map(|(literal, _)| literal)
}

/// The length of a character literal at the start of `rest`, such as `';'`, `'\''`, or the tail
/// of `b'x'`, or 1 for the quote that begins a lifetime.
fn char_literal(rest: &[u8]) -> usize {
    match rest {
        [b'\'', b'\\', ..] => rest
            .get(3..)
            .and_then(|tail| tail.iter().position(|b| *b == b'\''))
            .map_or(1, |end| end + 4),
        [b'\'', _, b'\'', ..] => 3,
        _ => 1,
    }
}

/// The string literal that starts at `at`, `"…"`, `r"…"`, `r#"…"#`, `b"…"`, or `br#"…"#`: its
/// contents, and where it ends.
fn string_literal(content: &[u8], at: usize) -> Option<(&[u8], usize)> {
    let rest = &content[at..];
    let (prefix, rest) = match rest {
        [b'b', b'r', ..] => (2, &rest[2..]),
        [b'b', b'"', ..] => (1, &rest[1..]),
        [b'r', ..] => (1, &rest[1..]),
        _ => (0, rest),
    };
    let raw = prefix > 0 && content[at + prefix - 1] == b'r';
    if !raw {
        let body = rest.strip_prefix(b"\"")?;
        let mut escaped = false;
        let end = body.iter().position(|byte| {
            let close = !escaped && *byte == b'"';
            escaped = !escaped && *byte == b'\\';
            close
        })?;
        return Some((&body[..end], at + prefix + 1 + end + 1));
    }
    let hashes = rest.iter().take_while(|byte| **byte == b'#').count();
    let body = rest[hashes..].strip_prefix(b"\"")?;
    let mut closing = vec![b'"'];
    closing.extend(std::iter::repeat_n(b'#', hashes));
    let end = find(body, &closing)?;
    Some((&body[..end], at + prefix + hashes + 1 + end + closing.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runs(source: &str) -> Vec<String> {
        commands_running_python(source.as_bytes())
            .into_iter()
            .map(|run| run.word)
            .collect()
    }

    #[test]
    fn a_program_literal_that_names_python_runs_it() {
        assert_eq!(runs("Command::new(\"python3\").arg(\"x\");"), ["python3"]);
        assert_eq!(
            runs("Command::new(\n  \"/usr/bin/python3\",\n);"),
            ["python3"]
        );
        assert_eq!(runs("Command::new(r#\"python\"#);"), ["python"]);
        assert_eq!(runs("Command::new(\"uv\").args([\"run\", \"x\"]);"), ["uv"]);
    }

    #[test]
    fn a_wrapper_runs_what_its_statement_names() {
        assert_eq!(runs("Command::new(\"env\").arg(\"python3\");"), ["python3"]);
        assert_eq!(
            runs("Command::new(\"sh\").args([\"-c\", \"python3 -c 1\"]).status();"),
            ["python3"]
        );
        assert_eq!(
            runs("Command::new(\"/bin/bash\").arg(r\"pip install x\");"),
            ["pip"]
        );
        assert_eq!(
            runs("Command::new(\"nix\").args([\"run\", \"nixpkgs#python3\"]);"),
            ["python3"]
        );
        assert_eq!(
            runs("Command::new(\"timeout\").args([\"9\", \"python3\"]);"),
            ["python3"]
        );
        assert_eq!(
            runs("Command::new(\"env\").arg(b\"python3\");"),
            ["python3"]
        );
        assert_eq!(
            runs("Command::new(\"env\").arg(br#\"python3\"#);"),
            ["python3"]
        );
    }

    /// A `;` inside a closure, a block, or a comment does not end the statement, and a tail
    /// expression ends at the `}` that closes its block.
    #[test]
    fn the_statement_is_read_to_its_own_end() {
        let closure = "Command::new(\"env\").map(|c| { c.x(); c }).arg(\"python3\");";
        assert_eq!(runs(closure), ["python3"]);
        let comment = "Command::new(\"env\") // stop; here\n    .arg(\"python3\");";
        assert_eq!(runs(comment), ["python3"]);
        let block = "Command::new(\"env\") /* ; */ .arg(\"python3\");";
        assert_eq!(runs(block), ["python3"]);
        let tail = "fn a() -> Command { Command::new(\"env\") }\nfn b() { let s = \"python3\"; }";
        assert!(runs(tail).is_empty());
        let character = "Command::new(\"env\").arg(';').arg(\"python3\");";
        assert_eq!(runs(character), ["python3"]);
    }

    #[test]
    fn a_string_beyond_the_statement_or_beside_another_program_is_data() {
        assert!(runs("let sample = \"python3\";").is_empty());
        assert!(runs("Command::new(\"cargo\").arg(\"python3\");").is_empty());
        assert!(runs("Command::new(\"env\"); let s = \"python3\";").is_empty());
        assert!(runs("Command::new(program).arg(\"python3\");").is_empty());
        assert!(runs("Command::new(\"git\").args([\"init\"]);").is_empty());
    }
}
