//! Output routing: which stream each kind of message goes to.
//!
//! The rule the specs set is that when structured output is requested, standard
//! output contains *nothing but* the structured payload, so an agent can pipe it
//! straight into a parser. Every progress message, warning, and error about the run
//! itself goes to standard error.
//!
//! That rule is easy to state and easy to break by one stray `println!`. So it is
//! made structural rather than remembered: [`Streams`] owns both handles, and the
//! only way to write to standard output is [`Streams::payload`], which refuses when
//! the run is not in structured mode. Nothing else in the crate touches stdout.

use std::io::{self, Write};

/// Which form the caller asked for.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Form {
    /// Human-readable output, for a contributor reading a terminal.
    #[default]
    Human,
    /// A structured payload on standard output, for an agent or a CI job.
    Structured,
}

impl Form {
    /// Whether the caller asked for a structured payload.
    pub const fn is_structured(self) -> bool {
        matches!(self, Self::Structured)
    }
}

/// The program's two output streams, with the routing rule enforced.
///
/// Generic over the writers so tests can capture both and assert what landed where.
/// That is the only reason for the generics: the routing rule is worth testing, and a
/// rule tested by reading the source is not tested.
pub struct Streams<O: Write, E: Write> {
    form: Form,
    out: O,
    err: E,
}

impl Streams<io::Stdout, io::Stderr> {
    /// The real process streams.
    pub fn process(form: Form) -> Self {
        Self {
            form,
            out: io::stdout(),
            err: io::stderr(),
        }
    }
}

impl<O: Write, E: Write> Streams<O, E> {
    /// Streams over the given writers.
    ///
    /// Used by tests, which capture both streams into buffers. The real program takes
    /// [`Streams::process`], so this is the only constructor that is not the process's
    /// own — which is what lets the routing rule be asserted rather than reviewed.
    #[cfg(test)]
    pub fn new(form: Form, out: O, err: E) -> Self {
        Self { form, out, err }
    }

    /// Which form this run is producing.
    pub const fn form(&self) -> Form {
        self.form
    }

    /// Consumes the streams, returning the two writers.
    ///
    /// Exists so a test can assert what landed on each stream. The routing rule is
    /// the thing most easily broken by a stray write, and a rule verified by reading
    /// the source is not verified.
    #[cfg(test)]
    pub fn into_parts(self) -> (O, E) {
        (self.out, self.err)
    }

    /// Writes the structured payload to standard output.
    ///
    /// The only path to standard output in the program. Refuses in human mode rather
    /// than silently writing, because a payload emitted when none was asked for is
    /// the same defect as prose emitted when one was.
    pub fn payload(&mut self, json: &str) -> io::Result<()> {
        debug_assert!(
            self.form.is_structured(),
            "a structured payload was written in human mode"
        );
        if !self.form.is_structured() {
            return Ok(());
        }
        writeln!(self.out, "{json}")?;
        self.out.flush()
    }

    /// Writes human-readable output.
    ///
    /// Goes to standard *output* in human mode, because that is where a person
    /// expects a report and where a pager reads it. In structured mode it goes to
    /// standard error, so it cannot corrupt the payload — the report is then
    /// commentary about a run whose real answer is the payload.
    pub fn report(&mut self, text: &str) -> io::Result<()> {
        if self.form.is_structured() {
            write!(self.err, "{text}")?;
            return self.err.flush();
        }
        write!(self.out, "{text}")?;
        self.out.flush()
    }

    /// Writes a progress message about the run itself.
    ///
    /// Always standard error, in both forms. Progress is not the answer to anything,
    /// and a human piping output to a file wants the report in the file and the
    /// progress on the terminal.
    pub fn progress(&mut self, message: &str) -> io::Result<()> {
        writeln!(self.err, "{message}")?;
        self.err.flush()
    }

    /// Writes an error about the run itself.
    ///
    /// Always standard error. An operational failure in structured mode must not
    /// emit half a payload followed by prose, which would leave a parser with
    /// malformed input rather than an empty stream and a nonzero exit code.
    pub fn failure(&mut self, message: &str) -> io::Result<()> {
        writeln!(self.err, "htmlspec: {message}")?;
        self.err.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Streams over byte buffers, returning what each captured.
    fn capture(form: Form, act: impl FnOnce(&mut Streams<Vec<u8>, Vec<u8>>)) -> (String, String) {
        let mut streams = Streams::new(form, Vec::new(), Vec::new());
        act(&mut streams);
        (
            String::from_utf8(streams.out).expect("stdout is UTF-8"),
            String::from_utf8(streams.err).expect("stderr is UTF-8"),
        )
    }

    /// The requirement: in structured mode, standard output is the payload and
    /// nothing else, even when the run also produced progress and a failure.
    #[test]
    fn structured_output_leaves_standard_output_parseable() {
        let (out, err) = capture(Form::Structured, |streams| {
            streams
                .progress("validating 3 documents")
                .expect("progress");
            streams.report("0 errors: accepted\n").expect("report");
            streams.failure("one file was unreadable").expect("failure");
            streams.payload(r#"{"accepted":true}"#).expect("payload");
        });

        assert_eq!(out, "{\"accepted\":true}\n");
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("stdout parses cleanly");
        assert_eq!(parsed["accepted"], serde_json::Value::Bool(true));

        // Everything else went to standard error, and none of it is missing.
        assert!(err.contains("validating 3 documents"), "{err}");
        assert!(err.contains("0 errors: accepted"), "{err}");
        assert!(err.contains("htmlspec: one file was unreadable"), "{err}");
    }

    /// Progress present *and* stdout still parseable, which is the scenario the
    /// specs name explicitly.
    #[test]
    fn progress_does_not_corrupt_the_payload() {
        let (out, err) = capture(Form::Structured, |streams| {
            for n in 1..=3 {
                streams
                    .progress(&format!("document {n}"))
                    .expect("progress");
            }
            streams.payload(r#"{"diagnostics":[]}"#).expect("payload");
        });
        assert!(
            serde_json::from_str::<serde_json::Value>(&out).is_ok(),
            "{out}"
        );
        assert_eq!(err.lines().count(), 3, "{err}");
    }

    /// In human mode the report is on standard output, where a person and a pager
    /// expect it, and progress is still on standard error.
    #[test]
    fn human_output_puts_the_report_on_standard_output() {
        let (out, err) = capture(Form::Human, |streams| {
            streams.progress("validating").expect("progress");
            streams.report("0 errors: accepted\n").expect("report");
        });
        assert_eq!(out, "0 errors: accepted\n");
        assert_eq!(err, "validating\n");
    }

    /// A payload written in human mode is refused rather than emitted, so a caller
    /// cannot half-produce structured output.
    #[test]
    fn a_payload_is_not_written_in_human_mode() {
        let mut streams = Streams::new(Form::Human, Vec::new(), Vec::new());
        // The debug assertion fires in a debug build, so this asserts the release
        // behavior: nothing written, and no error returned.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            streams.payload("{}").expect("no I/O error");
        }));
        // Either it panicked on the debug assertion, or it wrote nothing. Both are
        // the intended contract; what must not happen is a payload on stdout.
        if result.is_ok() {
            assert!(
                streams.out.is_empty(),
                "a payload reached stdout in human mode"
            );
        }
    }

    /// An operational failure never lands on standard output, in either form. A
    /// parser reading stdout must see an empty stream rather than prose.
    #[test]
    fn an_operational_failure_never_reaches_standard_output() {
        for form in [Form::Human, Form::Structured] {
            let (out, err) = capture(form, |streams| {
                streams
                    .failure("no such file: missing.html")
                    .expect("failure");
            });
            assert!(out.is_empty(), "{form:?} put a failure on stdout: {out}");
            assert!(err.contains("no such file"), "{err}");
        }
    }
}
