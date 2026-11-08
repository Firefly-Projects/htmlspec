//! The README check's cases: each check refuses the README it exists for, the good README passes,
//! and `--print` prints an install block and refuses an upgrade block. Each runs the real `bash`,
//! as the check does, `bash -n` on every block and the pick line in a new scratch directory that is
//! its home too. No block is ever run.

use std::cell::RefCell;

use super::*;

/// A README with every marked block, each right.
const GOOD: &str = r#"# A README
<!-- install: linux-release -->
```sh
(
  set -eu
  target="$(uname -m)-unknown-linux-musl"
  file="$(grep -- "-$target.tar.gz\$" SHA256SUMS | cut -d' ' -f3)"
  sha256sum --check --ignore-missing SHA256SUMS
  tar -xzf "$file"
  install -m 755 htmlspec "$HOME/.local/bin/htmlspec"
)
```
<!-- install: linux-homebrew -->
```sh
brew install example/tap/htmlspec
```
<!-- install: cargo -->
```sh
cargo install --locked htmlspec-cli
```
<!-- install: macos-homebrew -->
```sh
brew install example/tap/htmlspec
```
<!-- upgrade: upgrade-homebrew -->
```sh
brew upgrade htmlspec
```
<!-- upgrade: upgrade-cargo -->
```sh
cargo install --locked htmlspec-cli
```
<!-- upgrade: upgrade-nix -->
```sh
nix profile upgrade htmlspec
```
<!-- upgrade: upgrade-project -->
```sh
htmlspec init --tools claude
htmlspec validate
```
"#;

fn scratch() -> Result<TemporaryDirectory, CannotAsk> {
    new_scratch()
}

/// Every fault the check finds in `text`, as the stage prints them.
fn faults(text: &str) -> Vec<String> {
    check(text, &Target::ALL, &RealBash, &scratch)
        .expect("the check could ask")
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// `text` draws a fault naming `expected`.
fn refused(text: &str, expected: &str) {
    let found = faults(text);
    assert!(
        found.iter().any(|fault| fault.contains(expected)),
        "expected a fault naming {expected:?}, got {found:?}"
    );
}

#[test]
fn a_good_readme_passes() {
    assert_eq!(faults(GOOD), Vec::<String>::new());
}

#[test]
fn a_missing_homebrew_block() {
    refused(
        &GOOD.replace("<!-- install: linux-homebrew -->", ""),
        "linux-homebrew",
    );
}

#[test]
fn a_missing_cargo_block() {
    refused(
        &GOOD.replace("<!-- install: cargo -->", ""),
        "`install: cargo`",
    );
}

#[test]
fn a_missing_macos_block() {
    refused(
        &GOOD.replace("<!-- install: macos-homebrew -->", ""),
        "macos-homebrew",
    );
}

#[test]
fn a_syntax_error() {
    refused(&GOOD.replace("tar -xzf \"$file\"", "if then fi"), "bash -n");
}

#[test]
fn no_checksum_line() {
    refused(
        &GOOD.replace("  sha256sum --check --ignore-missing SHA256SUMS\n", ""),
        "nobody checked",
    );
}

/// Beyond the Python's cases: a `sha256sum --check` line that checks another file than
/// `SHA256SUMS` checks nothing the release wrote.
#[test]
fn a_checksum_line_that_names_another_file_checks_nothing() {
    refused(
        &GOOD.replace(
            "  sha256sum --check --ignore-missing SHA256SUMS\n",
            "  sha256sum --check --ignore-missing OLD_SHA256SUMS\n",
        ),
        "nobody checked",
    );
}

#[test]
fn unpacking_before_the_checksum() {
    refused(
        &GOOD.replace(
            "  sha256sum --check --ignore-missing SHA256SUMS\n  tar -xzf \"$file\"\n",
            "  tar -xzf \"$file\"\n  sha256sum --check --ignore-missing SHA256SUMS\n",
        ),
        "before its",
    );
}

#[test]
fn a_target_the_release_does_not_build() {
    refused(
        &GOOD.replace("unknown-linux-musl", "unknown-linux-gnu"),
        "unknown-linux-gnu",
    );
}

#[test]
fn a_wrong_field_of_sha256sums() {
    refused(&GOOD.replace("-f3", "-f1"), "picks");
}

#[test]
fn a_marker_with_no_fence() {
    refused(&GOOD.replacen("```sh\ncargo", "cargo", 1), "fence");
}

#[test]
fn a_name_marking_two_blocks() {
    refused(
        &format!("{GOOD}<!-- install: linux-homebrew -->\n```sh\ntrue\n```\n"),
        "two blocks",
    );
}

#[test]
fn a_missing_upgrade_block() {
    refused(
        &GOOD.replace("<!-- upgrade: upgrade-nix -->", ""),
        "`upgrade: upgrade-nix`",
    );
}

#[test]
fn a_missing_project_upgrade() {
    refused(
        &GOOD.replace("<!-- upgrade: upgrade-project -->", ""),
        "`upgrade: upgrade-project`",
    );
}

#[test]
fn an_upgrade_block_bash_refuses() {
    refused(
        &GOOD.replace("brew upgrade htmlspec", "brew upgrade <entry>"),
        "`upgrade: upgrade-homebrew` is refused by bash -n",
    );
}

#[test]
fn an_upgrade_block_marked_as_an_install_block() {
    refused(
        &GOOD.replace(
            "<!-- upgrade: upgrade-cargo -->",
            "<!-- install: upgrade-cargo -->",
        ),
        "mark it `upgrade: upgrade-cargo`",
    );
}

#[test]
fn a_name_shared_by_an_install_and_an_upgrade_block() {
    refused(
        &format!("{GOOD}<!-- upgrade: cargo -->\n```sh\ntrue\n```\n"),
        "two blocks",
    );
}

#[test]
fn an_install_block_marked_as_an_upgrade_block() {
    refused(
        &GOOD.replace("<!-- install: cargo -->", "<!-- upgrade: cargo -->"),
        "mark it `install: cargo`",
    );
}

#[test]
fn printing_an_install_block() {
    assert_eq!(
        block_to_print(GOOD, "linux-homebrew"),
        Ok(String::from("brew install example/tap/htmlspec\n"))
    );
}

#[test]
fn printing_an_upgrade_block_is_refused() {
    match block_to_print(GOOD, "upgrade-homebrew") {
        Err(Unprinted::Fault(fault)) => assert!(fault.to_string().contains("never run")),
        other => panic!("an upgrade block was printed: {other:?}"),
    }
}

/// A `bash` that runs what the real one runs, and records every script it is handed to run.
struct Recording {
    parsed: RefCell<Vec<String>>,
    ran: RefCell<Vec<(String, PathBuf)>>,
}

impl Bash for Recording {
    fn parse(&self, body: &str) -> Result<Option<String>, CannotAsk> {
        self.parsed.borrow_mut().push(body.to_owned());
        RealBash.parse(body)
    }

    fn run(&self, script: &str, directory: &Path) -> Result<String, CannotAsk> {
        self.ran
            .borrow_mut()
            .push((script.to_owned(), directory.to_path_buf()));
        RealBash.run(script, directory)
    }
}

/// The check parses every block and runs none: what it runs is the Linux block's pick line alone,
/// for each machine, in a scratch directory under the system's temporary directory, never in the
/// home directory of whoever runs it.
#[test]
fn no_block_is_run_and_the_pick_line_runs_in_a_scratch_directory() {
    let bash = Recording {
        parsed: RefCell::new(Vec::new()),
        ran: RefCell::new(Vec::new()),
    };
    let found = check(GOOD, &Target::ALL, &bash, &scratch).expect("the check could ask");
    assert!(found.is_empty(), "{found:?}");
    assert_eq!(bash.parsed.borrow().len(), 8);
    let ran = bash.ran.borrow();
    let pick = "file=\"$(grep -- \"-$target.tar.gz\\$\" SHA256SUMS | cut -d' ' -f3)\"";
    let machines: Vec<String> = ran
        .iter()
        .map(|(script, _)| {
            let target = script
                .lines()
                .nth(1)
                .and_then(|line| line.strip_prefix("target=\""))
                .unwrap_or_default();
            assert_eq!(
                script,
                &format!("set -eu\ntarget=\"{target}\n{pick}\nprintf %s \"$file\"\n")
            );
            target.trim_end_matches('"').to_owned()
        })
        .collect();
    assert_eq!(
        machines,
        [
            "x86_64-unknown-linux-musl",
            "aarch64-unknown-linux-musl",
            "riscv64-unknown-linux-musl"
        ]
    );
    let home = std::env::var_os("HOME").map(PathBuf::from);
    for (_, directory) in ran.iter() {
        assert!(
            directory.starts_with(std::env::temp_dir()),
            "{}",
            directory.display()
        );
        if let Some(home) = home
            .as_ref()
            .filter(|home| home.as_path() != Path::new("/"))
        {
            assert!(!directory.starts_with(home) || std::env::temp_dir().starts_with(home));
        }
        assert!(!directory.exists(), "the scratch directory is left behind");
    }
}

/// The pick line's home is its scratch directory, so `~` and `$HOME` in it never reach the home of
/// whoever runs the check: a pick line that names a different home picks nothing a release lists.
#[test]
fn the_pick_line_s_home_is_its_scratch_directory() {
    let watching = GOOD.replace(
        "cut -d' ' -f3)\"",
        "cut -d' ' -f3)$([ \"$HOME\" -ef \"$PWD\" ] || printf ' in another home')\"",
    );
    assert_eq!(faults(&watching), Vec::<String>::new());
    let elsewhere = GOOD.replace(
        "cut -d' ' -f3)\"",
        "cut -d' ' -f3)$([ \"$HOME\" -ef / ] || printf ' in another home')\"",
    );
    refused(&elsewhere, "in another home");
}

/// The README's blocks, printed: exit 0 with the body; an upgrade block, exit 1; no such block,
/// exit 2; and the check's own endings, 0 naming every block and 1 naming each fault.
#[test]
fn each_ending_has_its_exit_code() {
    let job = |name: &str| Job::Print(name.to_owned());
    let printed = report(
        Ok(String::from(GOOD)),
        &job("cargo"),
        &Target::ALL,
        &RealBash,
        &scratch,
    );
    assert_eq!(printed.exit, Exit::Clean);
    assert_eq!(printed.out, ["cargo install --locked htmlspec-cli"]);
    let upgrade = report(
        Ok(String::from(GOOD)),
        &job("upgrade-nix"),
        &Target::ALL,
        &RealBash,
        &scratch,
    );
    assert_eq!(upgrade.exit, Exit::Faults);
    let absent = report(
        Ok(String::from(GOOD)),
        &job("absent"),
        &Target::ALL,
        &RealBash,
        &scratch,
    );
    assert_eq!(absent.exit, Exit::CouldNotAsk);
    assert_eq!(
        absent.err,
        ["check-readme-install: no block is marked `install: absent`"]
    );
    let checked = report(
        Ok(String::from(GOOD)),
        &Job::Check,
        &Target::ALL,
        &RealBash,
        &scratch,
    );
    assert_eq!(checked.exit, Exit::Clean);
    assert_eq!(
        checked.out,
        [
            "  ok  the README's install and upgrade blocks: cargo, linux-homebrew, linux-release, \
          macos-homebrew, upgrade-cargo, upgrade-homebrew, upgrade-nix, upgrade-project"
        ]
    );
    let broken = report(
        Ok(GOOD.replace("-f3", "-f1")),
        &Job::Check,
        &Target::ALL,
        &RealBash,
        &scratch,
    );
    assert_eq!(broken.exit, Exit::Faults);
    assert!(broken.err.iter().all(|line| line.starts_with("!!! ")));
    let unreadable = report(
        Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
        &Job::Check,
        &Target::ALL,
        &RealBash,
        &scratch,
    );
    assert_eq!(unreadable.exit, Exit::CouldNotAsk);
}

/// A block with no lines prints one empty line, as the Python printed it.
#[test]
fn an_empty_block_prints_one_empty_line() {
    let text = "<!-- install: nothing -->\n```sh\n```\n";
    assert_eq!(block_to_print(text, "nothing"), Ok(String::from("\n")));
    assert_eq!(
        report(
            Ok(String::from(text)),
            &Job::Print(String::from("nothing")),
            &Target::ALL,
            &RealBash,
            &scratch
        )
        .out,
        [""]
    );
}
