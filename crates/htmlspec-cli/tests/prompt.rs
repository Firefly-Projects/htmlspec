//! The interactive prompt `init` opens, driven through a real pseudo-terminal.
//!
//! `prompt.rs`'s unit tests cover the decision to prompt and which rows open checked, and neither
//! reaches `inquire`. Three behaviours live only here: a fresh repository opens with every row
//! checked; confirming an empty selection is refused and the prompt asks again; and cancelling exits
//! 3 and writes nothing, the one exit code no other test reads from a real process. A fourth case
//! holds the asymmetry the refusal must not break, that `--tools none` still means no tools.
//!
//! **How the terminal is made.** The binary's three standard streams are the replica side of a
//! pseudo-terminal that `nix`'s `openpty`, a safe function, opens with a fixed window size. The
//! child gets no controlling terminal and needs none: the prompt asks standard input and output
//! whether they are terminals, and reads its keys from standard input when it is one. So the
//! workspace's `unsafe_code = "forbid"` holds here too.
//!
//! **How it waits.** Every read waits for the text it expects, or for the child to exit, and every
//! wait for a process to exit has the same deadline, after which the process is killed. The deadline
//! only bounds a failure, so a case is as fast as the binary paints, and nothing sleeps. The prompt
//! redraws only the lines a key changes, so each key waits for the line it changes: a space on a
//! checked row paints that row unchecked under the cursor, `> [ ]`, and a step down onto a checked
//! row paints `> [x]` there.
//!
//! **Why no case depends on the window.** The prompt reads the window's size from `/dev/tty`, the
//! controlling terminal, when it can open one, and from standard output otherwise. A test started
//! from a contributor's terminal has one, so the binary reads that window's width and not this
//! pseudo-terminal's, and in continuous integration it reads the pseudo-terminal's. Only the width
//! changes what the prompt writes: it breaks a line longer than the window, in the middle of a word.
//! So the cases read the text with its line breaks taken out, and look for nothing a break could
//! split, and one case runs in a window 12 columns wide, which continuous integration reads.

#![cfg(unix)]

mod support;

use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::mpsc;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use nix::errno::Errno;
use nix::poll::{poll, PollFd, PollFlags};
use nix::pty::{openpty, Winsize};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

/// How long any one wait may take before the case fails. Not a pause: every wait returns as soon
/// as what it waits for arrives, so only a case that is already failing pays it.
const DEADLINE: Duration = Duration::from_secs(30);

/// The window the pseudo-terminal opens with.
const WINDOW: Winsize = Winsize {
    ws_row: 24,
    ws_col: 80,
    ws_xpixel: 0,
    ws_ypixel: 0,
};

/// A window narrower than every line the prompt paints but its rows, so the prompt breaks its
/// question and its help across lines. The binary reads it where it has no controlling terminal,
/// as in continuous integration and the build sandbox.
const NARROW: Winsize = Winsize {
    ws_row: 6,
    ws_col: 12,
    ws_xpixel: 0,
    ws_ypixel: 0,
};

/// The tools the prompt lists, one row each.
const ROWS: usize = 5;

/// The question the prompt asks.
const QUESTION: &str = "Which tools should htmlspec set up?";

/// The cursor's row, unchecked and checked, as the prompt paints it.
const UNCHECKED_UNDER_THE_CURSOR: &str = "> [ ]";
const CHECKED_UNDER_THE_CURSOR: &str = "> [x]";

/// What the prompt says when a contributor confirms nothing.
const REFUSAL: &str = "select at least one tool";

/// What `init` says when it has established the project.
const READY: &str = "the project is ready";

const SPACE: &[u8] = b" ";
const DOWN: &[u8] = b"\x1b[B";
const ENTER: &[u8] = b"\r";
const ESC: &[u8] = b"\x1b";

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_htmlspec"))
}

/// Held by every case while it starts a process, and while it holds a pseudo-terminal descriptor
/// that a process started now would inherit. The cases run on threads of one process, so without it
/// one case's child could keep another case's terminal open.
fn starting() -> MutexGuard<'static, ()> {
    static STARTING: Mutex<()> = Mutex::new(());
    STARTING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Waits for `child` to exit, for [`DEADLINE`] at most, and gives what it wrote to any pipe. A child
/// that passes the deadline is killed, and the case fails naming `what`. The wait is a thread's, and
/// the deadline is a channel's, so nothing here sleeps.
fn within_deadline(child: Child, what: &str) -> Output {
    let pid = i32::try_from(child.id()).map(Pid::from_raw);
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(child.wait_with_output());
    });
    match receiver.recv_timeout(DEADLINE) {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => panic!("{what} could not be waited for: {error}"),
        Err(_) => {
            if let Ok(pid) = pid {
                let _ = kill(pid, Signal::SIGKILL);
            }
            panic!("{what} did not exit within {DEADLINE:?}");
        }
    }
}

/// Runs `command` with nothing on its standard input, and waits for it within the deadline.
fn finished(mut command: Command, what: &str) -> Output {
    let starting = starting();
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("{what} could not start: {error}"));
    drop(starting);
    within_deadline(child, what)
}

/// Whether this test process has a controlling terminal, whose window the binary would read in
/// place of the pseudo-terminal's.
fn has_a_controlling_terminal() -> bool {
    File::open("/dev/tty").is_ok()
}

/// A fresh Git repository in a temporary directory that removes itself, so the project the binary
/// resolves stops at its boundary.
struct Repository(PathBuf);

impl Repository {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("htmlspec-prompt-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        let mut git = Command::new("git");
        git.args(["init", "--quiet", "."]).current_dir(&path);
        let initialized = finished(git, "git init");
        assert!(
            initialized.status.success(),
            "git init, to make a repository for the project to stop at, exited {}",
            initialized.status
        );
        Self(path)
    }

    /// What `init` left, other than the repository itself.
    fn installed(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.0)
            .expect("the scratch directory is readable")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name != ".git")
            .collect();
        names.sort();
        names
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The binary, running with a pseudo-terminal for its three standard streams.
struct Terminal {
    controller: File,
    /// The child, until it is waited for.
    child: Option<Child>,
    arguments: Vec<String>,
    output: Vec<u8>,
    /// Whether the replica side has closed, which it does once the child has exited.
    closed: bool,
}

impl Terminal {
    fn start(directory: &Path, arguments: &[&str]) -> Self {
        Self::start_in(directory, arguments, WINDOW)
    }

    fn start_in(directory: &Path, arguments: &[&str], window: Winsize) -> Self {
        let starting = starting();
        let pty = openpty(&window, None).expect("a pseudo-terminal opens");
        // `openpty` leaves both descriptors open across `exec`, so a process another case started
        // now would hold this terminal open and this case would wait for it to exit. Each is
        // replaced by a copy closed on `exec`, before any other case can start a process.
        let duplicate = |descriptor: &OwnedFd| {
            descriptor
                .try_clone()
                .expect("a pseudo-terminal descriptor is duplicated")
        };
        let controller = duplicate(&pty.master);
        let replica = [
            duplicate(&pty.slave),
            duplicate(&pty.slave),
            duplicate(&pty.slave),
        ];
        drop(pty);
        let [input, output, error] = replica;
        // The command is a temporary, so the parent's copies of the replica close at the end of this
        // statement, and the controller reads an end once the child's close too.
        let child = Command::new(binary())
            .args(arguments)
            .current_dir(directory)
            .stdin(Stdio::from(input))
            .stdout(Stdio::from(output))
            .stderr(Stdio::from(error))
            .spawn()
            .expect("the binary starts");
        drop(starting);
        Self {
            controller: File::from(controller),
            child: Some(child),
            arguments: arguments
                .iter()
                .map(|argument| (*argument).to_owned())
                .collect(),
            output: Vec::new(),
            closed: false,
        }
    }

    /// Reads what the child has written, waiting until `until` at most. False when nothing came
    /// before the time was up.
    fn read(&mut self, until: Instant) -> bool {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return false;
        }
        let millis = i32::try_from(left.as_millis()).unwrap_or(i32::MAX);
        let mut descriptors = [PollFd::new(&self.controller, PollFlags::POLLIN)];
        match poll(&mut descriptors, millis) {
            Ok(0) => false,
            Ok(_) => {
                let mut chunk = [0_u8; 65536];
                match self.controller.read(&mut chunk) {
                    Ok(0) => self.closed = true,
                    Ok(count) => self.output.extend_from_slice(&chunk[..count]),
                    Err(error) if error.kind() == ErrorKind::Interrupted => {}
                    // Linux answers a read of the controller with EIO once the replica has closed.
                    Err(_) => self.closed = true,
                }
                true
            }
            Err(Errno::EINTR) => true,
            Err(error) => panic!("polling the pseudo-terminal failed: {error}"),
        }
    }

    /// Waits until what the child has written satisfies `done`, or fails the case naming `what`.
    fn wait_until(&mut self, what: &str, done: impl Fn(&str) -> bool) {
        let until = Instant::now() + DEADLINE;
        while !done(&self.text()) {
            assert!(
                !self.closed,
                "{:?} exited before it showed {what}:\n{}",
                self.arguments,
                self.text()
            );
            assert!(
                self.read(until),
                "{:?} did not show {what} within {DEADLINE:?}:\n{}",
                self.arguments,
                self.text()
            );
        }
    }

    /// Waits for the prompt to have drawn every row.
    fn wait_for_the_rows(&mut self) {
        self.wait_until("every row", |text| boxes(text).len() >= ROWS);
    }

    /// Presses `key`, and waits for the prompt to paint `painted` once more than it had.
    fn press(&mut self, key: &[u8], painted: &str) {
        let before = self.text().matches(painted).count();
        self.send(key);
        self.wait_until(&format!("{painted:?} after {key:?}"), |text| {
            text.matches(painted).count() > before
        });
    }

    fn send(&mut self, key: &[u8]) {
        self.controller
            .write_all(key)
            .expect("a key is written to the pseudo-terminal");
    }

    /// Waits for the child to exit, within the deadline, reading what it writes until it does.
    fn finish(mut self) -> (ExitStatus, String) {
        let until = Instant::now() + DEADLINE;
        while !self.closed {
            assert!(
                self.read(until),
                "{:?} did not exit within {DEADLINE:?}:\n{}",
                self.arguments,
                self.text()
            );
        }
        let what = format!("{:?}", self.arguments);
        let child = self.child.take().expect("the child is waited for once");
        let status = within_deadline(child, &what).status;
        let text = self.text();
        let arguments: Vec<&str> = self.arguments.iter().map(String::as_str).collect();
        support::held(&arguments, &text);
        (status, text)
    }

    /// What the child has written with the terminal's control sequences taken out, its line
    /// breaks kept.
    fn lines(&self) -> String {
        strip_controls(&String::from_utf8_lossy(&self.output))
    }

    /// What the child has written, as one line: the terminal's control sequences and every line
    /// break taken out, because the prompt breaks a line at the window's width.
    fn text(&self) -> String {
        self.lines()
            .chars()
            .filter(|character| !matches!(character, '\r' | '\n'))
            .collect()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // A case that failed partway leaves no process behind.
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Every checkbox in `text`, in the order the prompt drew them.
fn boxes(text: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find('[') {
        rest = &rest[at..];
        if rest.starts_with("[x]") {
            found.push("[x]");
        } else if rest.starts_with("[ ]") {
            found.push("[ ]");
        }
        rest = &rest[1..];
    }
    found
}

/// `text` without the escape sequences a terminal reads as commands: control sequences such as
/// `ESC [ 2 K`, which end at the first byte from `@` to `~`, and two-byte escapes.
fn strip_controls(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\u{1b}' {
            plain.push(character);
            continue;
        }
        if characters.peek() == Some(&'[') {
            characters.next();
            for next in characters.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            characters.next();
        }
    }
    plain
}

#[test]
fn a_fresh_repository_opens_with_every_row_checked() {
    let repository = Repository::new("fresh");
    let mut terminal = Terminal::start(&repository.0, &["init"]);
    terminal.wait_for_the_rows();
    let drawn = boxes(&terminal.text());
    terminal.send(ESC); // so this case writes nothing
    terminal.finish();

    assert_eq!(
        drawn,
        vec!["[x]"; ROWS],
        "a fresh repository opened with unchecked rows. Nothing detected must preselect every tool"
    );
}

/// Unchecks every row, confirms nothing, which must be refused with the prompt still open, then
/// checks a row again and confirms, in a terminal of `window`. Returns what the binary wrote, its
/// line breaks kept.
fn refuse_an_empty_confirmation(label: &str, window: Winsize) -> String {
    let repository = Repository::new(label);
    let mut terminal = Terminal::start_in(&repository.0, &["init"], window);
    terminal.wait_for_the_rows();

    // Uncheck every row: space, then down, five times.
    for row in 0..ROWS {
        terminal.press(SPACE, UNCHECKED_UNDER_THE_CURSOR);
        if row + 1 < ROWS {
            terminal.press(DOWN, CHECKED_UNDER_THE_CURSOR);
        }
    }
    assert!(
        !terminal.text().contains(REFUSAL),
        "the prompt refused before anything was confirmed:\n{}",
        terminal.text()
    );

    // Confirm nothing, which must be refused with the prompt still open.
    terminal.press(ENTER, REFUSAL);

    // Still in the list rather than exited: check a row again and confirm.
    terminal.press(SPACE, CHECKED_UNDER_THE_CURSOR);
    terminal.send(ENTER);
    let lines = terminal.lines();
    let (status, text) = terminal.finish();

    assert_eq!(
        status.code(),
        Some(0),
        "the run after the refusal exited {status}:\n{text}"
    );
    assert!(
        text.contains(READY),
        "the prompt did not ask again; it exited instead:\n{text}"
    );
    let left = repository.installed();
    assert!(
        left.iter().any(|name| name == "spec.nix") && left.iter().any(|name| name == "htmlspec"),
        "{left:?}"
    );
    lines
}

#[test]
fn an_empty_confirmation_is_refused_and_asks_again() {
    refuse_an_empty_confirmation("empty", WINDOW);
}

/// The same, in a window 12 columns wide, where the prompt breaks its question and its help in the
/// middle of words. Where this process has no controlling terminal, as in continuous integration,
/// the binary reads that width, and the question it painted is held to have been broken. From a
/// contributor's terminal it reads that window instead, and the case still holds the rest.
#[test]
fn an_empty_confirmation_is_refused_and_asks_again_in_a_narrow_window() {
    let lines = refuse_an_empty_confirmation("narrow", NARROW);
    if !has_a_controlling_terminal() {
        assert!(
            !lines.contains(QUESTION),
            "the question was painted whole, so the binary did not read the narrow window:\n{lines}"
        );
    }
}

#[test]
fn cancelling_writes_nothing_and_exits_three() {
    let repository = Repository::new("cancel");
    let mut terminal = Terminal::start(&repository.0, &["init"]);
    terminal.wait_for_the_rows();
    terminal.send(ESC);
    let (status, text) = terminal.finish();

    assert_eq!(
        status.code(),
        Some(3),
        "cancellation exited {status} rather than 3. Cancellation must differ from success, from a \
         document violation, and from an operational failure:\n{text}"
    );
    assert_eq!(
        repository.installed(),
        Vec::<String>::new(),
        "a cancelled run left files behind"
    );
}

/// The asymmetry the refusal must not break, measured through a real process with nothing on its
/// standard input.
#[test]
fn the_flag_still_means_no_tools() {
    let repository = Repository::new("flag");
    let arguments = ["init", "--tools", "none"];
    let mut command = Command::new(binary());
    command.args(arguments).current_dir(&repository.0);
    let finished = finished(command, "init --tools none");
    let stdout = String::from_utf8_lossy(&finished.stdout);
    support::held(&arguments, &stdout);

    assert_eq!(
        finished.status.code(),
        Some(0),
        "`--tools none` exited {}: {}",
        finished.status,
        String::from_utf8_lossy(&finished.stderr)
    );
    let left = repository.installed();
    assert!(left.iter().any(|name| name == "spec.nix"), "{left:?}");
    assert!(
        !left.iter().any(|name| name == ".pi" || name == ".claude"),
        "`--tools none` installed instructions: {left:?}"
    );
}
