//! The text of a Nix expression's strings, line by line.
//!
//! A Nix expression runs commands only from its strings, a build phase's `''…''` or a `"…"`, so
//! those are where a handoff is read. Outside them, `bash` is an attribute name or a package, as in
//! `sh = pkgs.bash;` or a list of inputs, and reading those words as commands found scripts in
//! them that nothing runs. Comments are skipped, and the escapes of both kinds of string are
//! honoured, so `''$` and `'''` do not end an indented string.

/// The strings' text on each line that holds some, numbered from 1, the strings on one line
/// joined by a space.
pub fn string_lines(content: &[u8]) -> Vec<(usize, String)> {
    #[derive(PartialEq)]
    enum State {
        Code,
        Double,
        Indented,
    }
    let mut lines: Vec<(usize, Vec<u8>)> = Vec::new();
    let mut number = 1;
    let mut state = State::Code;
    let mut at = 0;
    let push = |lines: &mut Vec<(usize, Vec<u8>)>, number: usize, byte: u8| match lines.last_mut() {
        Some((last, text)) if *last == number => text.push(byte),
        _ => lines.push((number, vec![byte])),
    };
    while let Some(&byte) = content.get(at) {
        let rest = &content[at..];
        if byte == b'\n' {
            number += 1;
            at += 1;
            continue;
        }
        match state {
            State::Code => {
                if byte == b'#' {
                    at += rest.iter().position(|b| *b == b'\n').unwrap_or(rest.len());
                    continue;
                }
                if rest.starts_with(b"/*") {
                    let end = rest
                        .windows(2)
                        .position(|w| w == b"*/")
                        .map_or(rest.len(), |end| end + 2);
                    number += rest[..end].iter().filter(|b| **b == b'\n').count();
                    at += end;
                    continue;
                }
                if byte == b'"' {
                    state = State::Double;
                    push(&mut lines, number, b' ');
                } else if rest.starts_with(b"''") {
                    state = State::Indented;
                    push(&mut lines, number, b' ');
                    at += 1;
                }
            }
            State::Double => match byte {
                b'\\' => {
                    if let Some(&next) = rest.get(1) {
                        if next != b'\n' {
                            push(&mut lines, number, next);
                        }
                    }
                    at += 1;
                }
                b'"' => state = State::Code,
                _ => push(&mut lines, number, byte),
            },
            State::Indented => {
                if rest.starts_with(b"'''") || rest.starts_with(b"''$") {
                    push(&mut lines, number, rest[2]);
                    at += 3;
                    continue;
                }
                if rest.starts_with(b"''\\") {
                    if let Some(&next) = rest.get(3) {
                        push(&mut lines, number, next);
                    }
                    at += 4;
                    continue;
                }
                if rest.starts_with(b"''") {
                    state = State::Code;
                    at += 2;
                    continue;
                }
                push(&mut lines, number, byte);
            }
        }
        at += 1;
    }
    lines
        .into_iter()
        .map(|(number, text)| (number, String::from_utf8_lossy(&text).into_owned()))
        .filter(|(_, text)| !text.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_strings_are_read() {
        let flake = concat!(
            "{\n",
            "  sh = pkgs.bash; # bash tools/x\n",
            "  inputs = [ bash jq ];\n",
            "  name = \"source\";\n",
            "  buildPhase = ''\n",
            "    bash ci/steps\n",
            "    echo ''${HOME} '''quoted'''\n",
            "  '';\n",
            "  /* bash tools/y */ check = \"bash ci/other\";\n",
            "}\n",
        );
        let lines: Vec<(usize, String)> = string_lines(flake.as_bytes())
            .into_iter()
            .map(|(number, text)| (number, text.trim().to_owned()))
            .collect();
        assert_eq!(
            lines,
            [
                (4, String::from("source")),
                (6, String::from("bash ci/steps")),
                (7, String::from("echo ${HOME} 'quoted'")),
                (9, String::from("bash ci/other")),
            ]
        );
    }
}
