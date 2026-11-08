//! The names that mean Python: an interpreter's, the action that installs one, and the launchers
//! that run one.
//!
//! A name is found as a whole word, a word being a run of letters, digits, `_`, and `-`. So
//! `pkgs.python3`, `/usr/bin/python3`, and `` `python3` `` each hold the word `python3`, while
//! `no-python`, `python-debt`, and `python3-pip` are words of their own. The match ignores
//! case, because a case-insensitive disk such as macOS's runs `Python3` as readily as `python3`.

/// A byte that continues a word: a letter, a digit, `_`, or `-`.
pub fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// What may follow a name's stem.
#[derive(Clone, Copy)]
enum Suffix {
    /// `python`, `python3`, `python313`, `python3.13`, `python3.13t`, `python3m`.
    Python,
    /// `pypy`, `pypy3`, `pypy3.10`.
    Pypy,
    /// `pip`, `pip3`, `pip3.12`, `pipx`.
    Pip,
    /// `uv`, `uvx`.
    Uv,
    /// The stem alone: `setup-python`, `poetry`, `conda`.
    None,
}

/// Each stem, with what may follow it. `setup-python` and `setup-uv`, the actions that install an
/// interpreter and a launcher, are stems of their own, because `-` continues a word and so
/// `python` inside the first is no whole word.
const STEMS: [(&[u8], Suffix); 9] = [
    (b"setup-python", Suffix::None),
    (b"setup-uv", Suffix::None),
    (b"python", Suffix::Python),
    (b"pypy", Suffix::Pypy),
    (b"pipx", Suffix::None),
    (b"pip", Suffix::Pip),
    (b"uv", Suffix::Uv),
    (b"poetry", Suffix::None),
    (b"conda", Suffix::None),
];

fn digits(text: &[u8], at: usize) -> usize {
    text.get(at..)
        .unwrap_or_default()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count()
}

fn byte_at(text: &[u8], at: usize) -> Option<u8> {
    text.get(at).map(u8::to_ascii_lowercase)
}

/// Where a name whose suffix starts at `at` may end, shortest first.
fn ends(text: &[u8], at: usize, suffix: Suffix) -> Vec<usize> {
    let mut ends = vec![at];
    let version = |ends: &mut Vec<usize>, from: usize| {
        let whole = digits(text, from);
        if whole == 0 {
            return None;
        }
        let mut end = from + whole;
        ends.push(end);
        if byte_at(text, end) == Some(b'.') {
            let minor = digits(text, end + 1);
            if minor > 0 {
                end += 1 + minor;
                ends.push(end);
            }
        }
        Some(end)
    };
    match suffix {
        Suffix::Python => {
            if let Some(end) = version(&mut ends, at) {
                if matches!(byte_at(text, end), Some(b't' | b'm')) {
                    ends.push(end + 1);
                }
            }
        }
        Suffix::Pypy | Suffix::Pip => {
            version(&mut ends, at);
        }
        Suffix::Uv => {
            if byte_at(text, at) == Some(b'x') {
                ends.push(at + 1);
            }
        }
        Suffix::None => {}
    }
    ends
}

/// The first name that means Python in `text`, as a whole word in any case, with where it starts.
pub fn python_word(text: &[u8]) -> Option<(usize, String)> {
    let lower = text.to_ascii_lowercase();
    for start in 0..lower.len() {
        if start > 0 && is_word_byte(lower[start - 1]) {
            continue;
        }
        let rest = &lower[start..];
        for (stem, suffix) in STEMS {
            if !rest.starts_with(stem) {
                continue;
            }
            let found = ends(&lower, start + stem.len(), suffix)
                .into_iter()
                .rev()
                .find(|end| !lower.get(*end).is_some_and(|byte| is_word_byte(*byte)));
            if let Some(end) = found {
                return Some((
                    start,
                    String::from_utf8_lossy(&text[start..end]).into_owned(),
                ));
            }
        }
    }
    None
}

/// Whether a program's whole name means Python: an interpreter, or a launcher that runs one.
pub fn names_python(name: &str) -> bool {
    matches!(python_word(name.as_bytes()), Some((0, word)) if word.len() == name.len())
}

/// The first Nix identifier on a line that holds `python` or `pypy` in any case, as in
/// `pkgs.python313`, `python3Minimal`, `python312.withPackages`, `buildPythonPackage`, or
/// `pkgs.pypy3`. A Nix identifier is a run of letters, digits, `_`, `'`, and `-`, and `.` separates
/// the parts of an attribute path.
pub fn nix_identifier(line: &[u8]) -> Option<(usize, String)> {
    let is_identifier = |byte: u8| is_word_byte(byte) || byte == b'\'';
    let mut start = 0;
    while start < line.len() {
        if !is_identifier(line[start]) {
            start += 1;
            continue;
        }
        let length = line[start..]
            .iter()
            .take_while(|byte| is_identifier(**byte))
            .count();
        let identifier = &line[start..start + length];
        let lower = identifier.to_ascii_lowercase();
        let holds = |needle: &[u8]| lower.windows(needle.len()).any(|window| window == needle);
        if holds(b"python") || holds(b"pypy") {
            return Some((start, String::from_utf8_lossy(identifier).into_owned()));
        }
        start += length;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str) -> Option<String> {
        python_word(text.as_bytes()).map(|(_, word)| word)
    }

    #[test]
    fn an_interpreter_is_a_whole_word_in_any_case() {
        for (text, found) in [
            ("run: python3 publish/test", "python3"),
            ("pkgs.python3", "python3"),
            ("is `/usr/bin/python3`, a stub", "python3"),
            ("python3.13 -m venv", "python3.13"),
            ("python3.13t -c 1", "python3.13t"),
            ("python3m -c 1", "python3m"),
            ("python313 -c 1", "python313"),
            ("end of python3.", "python3"),
            ("python -c pass", "python"),
            ("Python3 -c 1", "Python3"),
            ("Python's pty, in prose", "Python"),
            ("pypy3 run.py", "pypy3"),
            ("uses: actions/setup-python@v5", "setup-python"),
            ("uses: astral-sh/setup-uv@v5", "setup-uv"),
            ("\"tools\":[\"cargo\",\"python3\"]", "python3"),
        ] {
            assert_eq!(word(text).as_deref(), Some(found), "{text}");
        }
        for prose in [
            "PYTHONPATH",
            "the no-python stage",
            "xtask/python-debt",
            "python3-pip",
            "cpython",
            "mypython3",
            "python_requires",
            "ipython",
            "release_facts.py",
            "python3Minimal",
        ] {
            assert_eq!(word(prose), None, "{prose}");
        }
    }

    #[test]
    fn a_launcher_that_runs_python_is_a_name_too() {
        for (text, found) in [
            ("pip install x", "pip"),
            ("pip3 install x", "pip3"),
            ("pipx run black", "pipx"),
            ("uv run --python 3.12 x", "uv"),
            ("uvx ruff", "uvx"),
            ("poetry run x", "poetry"),
            ("conda activate", "conda"),
        ] {
            assert_eq!(word(text).as_deref(), Some(found), "{text}");
        }
        for prose in [
            "uv-index",
            "pipeline",
            "pipe",
            "uvicorn",
            "recipe",
            "ultraviolet",
        ] {
            assert_eq!(word(prose), None, "{prose}");
        }
    }

    #[test]
    fn a_program_name_means_python_only_whole() {
        for name in [
            "python",
            "python2",
            "python3",
            "python3.13",
            "python3.13t",
            "Python3",
            "pypy3",
            "uv",
        ] {
            assert!(names_python(name), "{name}");
        }
        for name in [
            "pythonw",
            "python3.",
            "python3.x",
            "cpython",
            "bash",
            "",
            "python3-config",
        ] {
            assert!(!names_python(name), "{name}");
        }
    }

    #[test]
    fn a_nix_identifier_that_holds_python_is_found() {
        let found = |line: &str| nix_identifier(line.as_bytes()).map(|(_, word)| word);
        for (line, identifier) in [
            ("packages = [ pkgs.python313 ];", "python313"),
            ("pkgs.python3Minimal", "python3Minimal"),
            (
                "(pkgs.python312.withPackages (p: [ p.requests ]))",
                "python312",
            ),
            ("pkgs.pypy3", "pypy3"),
            ("buildPythonPackage {", "buildPythonPackage"),
        ] {
            assert_eq!(found(line).as_deref(), Some(identifier), "{line}");
        }
        assert_eq!(
            found("nativeBuildInputs = [ pkgs.bash pkgs.gitMinimal ];"),
            None
        );
    }
}
