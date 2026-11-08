//! The crate index, asked whether it holds a crate's version.
//!
//! The real index is asked through `curl`, which every runner and the development shell have, so
//! the tools carry no HTTP client and no TLS stack for one GET. `curl` keeps what the decision
//! needs: `--write-out '%{http_code}'` gives the status, `--dump-header` the headers, and with no
//! `--fail` a completed request of any status reaches [`answer`], while a request that did not
//! complete is a refusal carrying curl's own message. `-q` comes first, so no `.curlrc` can add a
//! header, and `--max-time 30` bounds the whole request. There is no `--location`: the sparse index
//! answers its paths directly, so a redirect is refused rather than followed to an address the run
//! did not name, User-Agent and all.

use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::fmt;
use std::hash::{BuildHasher, Hasher};
use std::io::ErrorKind;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::retry::RetryAfter;
use crate::facts::UserAgent;
use crate::workspace::{CrateName, PackageVersion};

/// The sparse index crates.io serves.
pub const CRATES_IO_INDEX: &str = "https://index.crates.io";

/// What the index said about one crate's version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexAnswer {
    /// It holds the version.
    Held,
    /// It does not: it holds other versions, or answered 404, 410, or 451, as cargo reads a sparse
    /// index's answer for a crate it does not hold.
    Absent,
    /// It answered 429, and asks to be asked again.
    AskLater(RetryAfter),
    /// It could not say.
    Refused(IndexRefusal),
}

/// Why the index could not say whether it holds a crate's version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexRefusal {
    /// It answered a status that is none of the others, a redirect among them.
    Status { name: CrateName, status: u16 },
    /// A line of the crate's index file is not JSON, at this line of the file and this column.
    NotJson {
        name: CrateName,
        line: usize,
        column: usize,
    },
    /// `curl` could not be started.
    CurlNotStarted { name: CrateName, kind: ErrorKind },
    /// The request did not complete: `curl` exited with `exit`, and said `stderr`.
    Incomplete {
        name: CrateName,
        exit: Option<i32>,
        stderr: String,
    },
    /// `curl` wrote no status that is a number, but `written`.
    NoStatus { name: CrateName, written: String },
    /// The private directory for the answer's headers and body could not be made.
    NoScratch { name: CrateName, kind: ErrorKind },
    /// The headers or the body `curl` wrote could not be read back.
    Unreadable {
        name: CrateName,
        what: Written,
        kind: ErrorKind,
    },
}

/// What `curl` wrote to a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Written {
    Headers,
    Body,
}

impl fmt::Display for IndexRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { name, status } => {
                write!(formatter, "the index answered HTTP {status} for {name}")
            }
            Self::NotJson { name, line, column } => write!(
                formatter,
                "the index's entry for {name} is not JSON, at line {line}, column {column}"
            ),
            Self::CurlNotStarted { name, kind } => write!(
                formatter,
                "the index could not be asked about {name}: curl could not be run: {kind}"
            ),
            Self::Incomplete { name, exit, stderr } => {
                let exit = exit.map_or_else(|| String::from("a signal"), |code| code.to_string());
                write!(
                    formatter,
                    "the index could not be asked about {name}: curl exited {exit}: {stderr}"
                )
            }
            Self::NoStatus { name, written } => write!(
                formatter,
                "the index could not be asked about {name}: curl wrote the status {written:?}"
            ),
            Self::NoScratch { name, kind } => write!(
                formatter,
                "the index could not be asked about {name}: no private directory for its answer: \
                 {kind}"
            ),
            Self::Unreadable { name, what, kind } => write!(
                formatter,
                "the index's answer about {name} could not be read back: its {}: {kind}",
                match what {
                    Written::Headers => "headers",
                    Written::Body => "body",
                }
            ),
        }
    }
}

/// Asks the index.
pub trait Registry {
    fn ask(&self, name: &CrateName, version: &PackageVersion) -> IndexAnswer;
}

/// The path of a crate's file in a sparse index, by the registry's layout rule.
pub fn index_path(name: &CrateName) -> String {
    let name = name.as_str().to_ascii_lowercase();
    // A crate's name is ASCII, so each byte is a character.
    match name.len() {
        1 | 2 => format!("{}/{name}", name.len()),
        3 => format!("3/{}/{name}", &name[..1]),
        _ => format!("{}/{}/{name}", &name[..2], &name[2..4]),
    }
}

/// What an answer of `status`, with `headers` and `body`, says about `name` at `version`.
pub fn answer(
    status: u16,
    headers: &str,
    body: &[u8],
    name: &CrateName,
    version: &PackageVersion,
) -> IndexAnswer {
    match status {
        200 => {
            let body = String::from_utf8_lossy(body);
            for (number, line) in body.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let entry: serde_json::Value = match serde_json::from_str(line) {
                    Ok(entry) => entry,
                    Err(error) => {
                        return IndexAnswer::Refused(IndexRefusal::NotJson {
                            name: name.clone(),
                            line: number + 1,
                            column: error.column(),
                        })
                    }
                };
                if entry.get("vers").and_then(serde_json::Value::as_str) == Some(version.as_str()) {
                    return IndexAnswer::Held;
                }
            }
            IndexAnswer::Absent
        }
        404 | 410 | 451 => IndexAnswer::Absent,
        429 => IndexAnswer::AskLater(RetryAfter::from_header(header(headers, "retry-after"))),
        status => IndexAnswer::Refused(IndexRefusal::Status {
            name: name.clone(),
            status,
        }),
    }
}

/// The value of the header `name` in the last response of `headers`, as `--dump-header` writes
/// them, matched without regard to case.
fn header<'a>(headers: &'a str, name: &str) -> Option<&'a str> {
    let last = headers
        .split("\r\n\r\n")
        .filter(|block| !block.trim().is_empty())
        .last()?;
    last.lines().skip(1).find_map(|line| {
        let (field, value) = line.split_once(':')?;
        field
            .trim()
            .eq_ignore_ascii_case(name)
            .then_some(value.trim())
    })
}

/// Whether `curl` reads the proxy the environment names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proxies {
    /// As the process's environment names them, as for every request a release makes.
    FromTheEnvironment,
    /// None: each request goes straight to the index, as a test's listener on the loopback
    /// interface needs whatever a contributor's environment says.
    Removed,
}

/// The variables `curl` reads a proxy from.
const PROXY_VARIABLES: [&str; 6] = [
    "http_proxy",
    "HTTP_PROXY",
    "https_proxy",
    "HTTPS_PROXY",
    "all_proxy",
    "ALL_PROXY",
];

/// The index, asked through `curl`.
pub struct CurlRegistry {
    index: String,
    user_agent: UserAgent,
    proxies: Proxies,
}

impl CurlRegistry {
    pub fn new(index: &str, user_agent: UserAgent, proxies: Proxies) -> Self {
        Self {
            index: index.trim_end_matches('/').to_owned(),
            user_agent,
            proxies,
        }
    }

    /// The command line, but for the program's name.
    fn arguments(&self, address: &str, headers: &Path, body: &Path) -> Vec<OsString> {
        let mut arguments: Vec<OsString> = [
            "-q",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "--user-agent",
            self.user_agent.as_str(),
            "--dump-header",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        arguments.push(headers.into());
        arguments.push("--output".into());
        arguments.push(body.into());
        arguments.push("--write-out".into());
        arguments.push("%{http_code}".into());
        arguments.push(address.into());
        arguments
    }
}

impl Registry for CurlRegistry {
    fn ask(&self, name: &CrateName, version: &PackageVersion) -> IndexAnswer {
        let refused = |refusal: IndexRefusal| IndexAnswer::Refused(refusal);
        let scratch = match Scratch::new() {
            Ok(scratch) => scratch,
            Err(error) => {
                return refused(IndexRefusal::NoScratch {
                    name: name.clone(),
                    kind: error.kind(),
                })
            }
        };
        let headers = scratch.0.join("headers");
        let body = scratch.0.join("body");
        let address = format!("{}/{}", self.index, index_path(name));
        let mut command = Command::new("curl");
        command.args(self.arguments(&address, &headers, &body));
        if self.proxies == Proxies::Removed {
            for variable in PROXY_VARIABLES {
                command.env_remove(variable);
            }
        }
        let output = match command.output() {
            Ok(output) => output,
            Err(error) => {
                return refused(IndexRefusal::CurlNotStarted {
                    name: name.clone(),
                    kind: error.kind(),
                })
            }
        };
        if !output.status.success() {
            return refused(IndexRefusal::Incomplete {
                name: name.clone(),
                exit: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        let written = String::from_utf8_lossy(&output.stdout);
        let Ok(status) = written.trim().parse::<u16>() else {
            return refused(IndexRefusal::NoStatus {
                name: name.clone(),
                written: written.into_owned(),
            });
        };
        // Read as bytes, so a header that is not UTF-8 costs its own characters and no other
        // header's, Retry-After among them.
        let headers = match std::fs::read(&headers) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(error) => {
                return refused(IndexRefusal::Unreadable {
                    name: name.clone(),
                    what: Written::Headers,
                    kind: error.kind(),
                })
            }
        };
        let body = match std::fs::read(&body) {
            Ok(bytes) => bytes,
            // curl writes no body file for an answer with an empty body.
            Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                return refused(IndexRefusal::Unreadable {
                    name: name.clone(),
                    what: Written::Body,
                    kind: error.kind(),
                })
            }
        };
        answer(status, &headers, &body, name, version)
    }
}

/// A private directory for one request's headers and body, made new for it and removed when the
/// request is done. Its name holds the process's id, a count, and a random number, it is made with
/// `mkdir`, which refuses a path that exists, and only its owner may enter it, so nothing another
/// user put in a shared temporary directory is ever read or written through.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> std::io::Result<Self> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let mut last = None;
        for _ in 0..8 {
            let mut random = RandomState::new().build_hasher();
            random.write_usize(NEXT.fetch_add(1, Ordering::Relaxed));
            let path = std::env::temp_dir().join(format!(
                "htmlspec-index-{}-{:016x}",
                std::process::id(),
                random.finish()
            ));
            match std::fs::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => last = Some(error),
                Err(error) => return Err(error),
            }
        }
        Err(last.unwrap_or_else(|| ErrorKind::AlreadyExists.into()))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeSet;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex, PoisonError};
    use std::thread::JoinHandle;

    use super::*;
    use crate::publish_crates::retry::HttpDate;

    /// Spelled out rather than read from the facts: this is what every request must carry, and a
    /// test that read it from the code under test would pass whatever that code said.
    pub(crate) const USER_AGENT: &str =
        "htmlspec-release (https://github.com/Firefly-Projects/htmlspec)";

    pub(crate) fn user_agent() -> UserAgent {
        crate::facts::Facts::read()
            .expect("the facts read")
            .user_agent()
            .clone()
    }

    fn name(text: &str) -> CrateName {
        CrateName::parse(text).expect("a crate's name")
    }

    fn version(text: &str) -> PackageVersion {
        PackageVersion::parse(text).expect("a version")
    }

    /// One request as the listener received it.
    #[derive(Clone, Debug)]
    pub(crate) struct Received {
        pub path: String,
        /// Each header's name, in lower case, and its value.
        pub headers: Vec<(String, String)>,
    }

    /// A server on the loopback interface that answers each request at once with what `respond`
    /// gives for its path, and records what it received.
    pub(crate) struct Listener {
        address: SocketAddr,
        received: Arc<Mutex<Vec<Received>>>,
        stopping: Arc<AtomicBool>,
        serving: Option<JoinHandle<()>>,
    }

    impl Listener {
        pub(crate) fn answering(respond: impl Fn(&str) -> Vec<u8> + Send + 'static) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("a port on the loopback");
            let address = listener.local_addr().expect("the listener's address");
            let received = Arc::new(Mutex::new(Vec::new()));
            let stopping = Arc::new(AtomicBool::new(false));
            let serving = {
                let received = Arc::clone(&received);
                let stopping = Arc::clone(&stopping);
                std::thread::spawn(move || {
                    for stream in listener.incoming() {
                        if stopping.load(Ordering::SeqCst) {
                            break;
                        }
                        let Ok(mut stream) = stream else { continue };
                        let Some(request) = read_request(&mut stream) else {
                            continue;
                        };
                        let reply = respond(&request.path);
                        received
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .push(request);
                        let _ = stream.write_all(&reply);
                    }
                })
            };
            Self {
                address,
                received,
                stopping,
                serving: Some(serving),
            }
        }

        pub(crate) fn address(&self) -> String {
            format!("http://{}", self.address)
        }

        pub(crate) fn received(&self) -> Vec<Received> {
            self.received
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
        }

        pub(crate) fn registry(&self) -> CurlRegistry {
            CurlRegistry::new(&self.address(), user_agent(), Proxies::Removed)
        }
    }

    impl Drop for Listener {
        fn drop(&mut self) {
            self.stopping.store(true, Ordering::SeqCst);
            // One connection wakes the accept loop, which then sees it is stopping.
            let _ = TcpStream::connect(self.address);
            if let Some(serving) = self.serving.take() {
                let _ = serving.join();
            }
        }
    }

    /// Reads a request's line and headers.
    fn read_request(stream: &mut TcpStream) -> Option<Received> {
        let mut head = Vec::new();
        let mut chunk = [0_u8; 4096];
        while !head.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = stream.read(&mut chunk).ok()?;
            if count == 0 {
                return None;
            }
            head.extend_from_slice(&chunk[..count]);
        }
        let head = String::from_utf8_lossy(&head).into_owned();
        let mut lines = head.split("\r\n");
        let path = lines.next()?.split(' ').nth(1)?.to_owned();
        let headers = lines
            .take_while(|line| !line.is_empty())
            .filter_map(|line| line.split_once(':'))
            .map(|(field, value)| (field.trim().to_ascii_lowercase(), value.trim().to_owned()))
            .collect();
        Some(Received { path, headers })
    }

    /// An answer of `status`, with `headers` beside its length, and `body`.
    pub(crate) fn reply(status: u16, headers: &[(&str, &str)], body: &str) -> Vec<u8> {
        let raw: Vec<(&str, &[u8])> = headers
            .iter()
            .map(|(field, value)| (*field, value.as_bytes()))
            .collect();
        reply_bytes(status, &raw, body)
    }

    /// An answer whose headers' values are bytes as they stand, UTF-8 or not.
    fn reply_bytes(status: u16, headers: &[(&str, &[u8])], body: &str) -> Vec<u8> {
        let mut bytes = format!(
            "HTTP/1.1 {status} Answer\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        )
        .into_bytes();
        for (field, value) in headers {
            bytes.extend_from_slice(field.as_bytes());
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(value);
            bytes.extend_from_slice(b"\r\n");
        }
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(body.as_bytes());
        bytes
    }

    /// An index file holding `name` at each of `versions`.
    pub(crate) fn entries(name: &str, versions: &[&str]) -> String {
        versions
            .iter()
            .map(|version| {
                format!(
                    "{{\"name\":\"{name}\",\"vers\":\"{version}\",\"deps\":[],\"cksum\":\"{}\"}}\n",
                    "0".repeat(64)
                )
            })
            .collect()
    }

    /// The header names a request carries beyond the User-Agent's, which curl adds of its own.
    pub(crate) fn assert_only_the_user_agent(request: &Received) {
        let names: BTreeSet<&str> = request
            .headers
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(
            names,
            BTreeSet::from(["accept", "host", "user-agent"]),
            "{request:?}"
        );
        let agent = request
            .headers
            .iter()
            .find(|(name, _)| name == "user-agent")
            .map(|(_, value)| value.as_str());
        assert_eq!(agent, Some(USER_AGENT), "{request:?}");
    }

    fn ask_once(reply_with: Vec<u8>, crate_name: &str) -> (IndexAnswer, Received) {
        let listener = Listener::answering(move |_| reply_with.clone());
        let answer = listener
            .registry()
            .ask(&name(crate_name), &version("9.8.7"));
        let received = listener.received();
        assert_eq!(received.len(), 1, "{received:?}");
        let request = received.into_iter().next().expect("one request");
        assert_only_the_user_agent(&request);
        (answer, request)
    }

    #[test]
    fn an_index_holding_the_version_answers_held_on_the_wire() {
        let body = entries("htmlspec-schema", &["0.0.1", "9.8.7"]);
        let (answer, request) = ask_once(reply(200, &[], &body), "htmlspec-schema");
        assert_eq!(answer, IndexAnswer::Held);
        assert_eq!(request.path, "/ht/ml/htmlspec-schema");
    }

    #[test]
    fn an_index_holding_other_versions_answers_absent_on_the_wire() {
        let body = entries("htmlspec-schema", &["0.0.1", "9.8.6"]);
        let (answer, _) = ask_once(reply(200, &[], &body), "htmlspec-schema");
        assert_eq!(answer, IndexAnswer::Absent);
    }

    #[test]
    fn a_404_answers_absent_on_the_wire() {
        let (answer, _) = ask_once(reply(404, &[], ""), "htmlspec-cli");
        assert_eq!(answer, IndexAnswer::Absent);
    }

    #[test]
    fn a_429_with_seconds_asks_later_on_the_wire() {
        let (answer, _) = ask_once(reply(429, &[("Retry-After", "17")], ""), "htmlspec-cli");
        assert_eq!(answer, IndexAnswer::AskLater(RetryAfter::Seconds(17)));
    }

    #[test]
    fn a_429_with_a_date_asks_later_on_the_wire() {
        let date = "Sat, 31 Oct 2026 00:01:30 GMT";
        let (answer, _) = ask_once(reply(429, &[("Retry-After", date)], ""), "htmlspec-cli");
        assert_eq!(
            answer,
            IndexAnswer::AskLater(RetryAfter::Date(HttpDate::parse(date).expect("a date")))
        );
    }

    /// One byte that is not UTF-8 in another header costs that header's characters, not the
    /// answer's Retry-After.
    #[test]
    fn a_header_that_is_not_utf_8_leaves_retry_after_readable_on_the_wire() {
        let headers: [(&str, &[u8]); 2] = [("X-Junk", b"caf\xe9"), ("Retry-After", b"17")];
        let (answer, _) = ask_once(reply_bytes(429, &headers, ""), "htmlspec-cli");
        assert_eq!(answer, IndexAnswer::AskLater(RetryAfter::Seconds(17)));
    }

    #[test]
    fn a_500_is_refused_naming_the_status_on_the_wire() {
        let (answer, _) = ask_once(reply(500, &[], "down"), "htmlspec-cli");
        let refusal = IndexRefusal::Status {
            name: name("htmlspec-cli"),
            status: 500,
        };
        assert_eq!(
            refusal.to_string(),
            "the index answered HTTP 500 for htmlspec-cli"
        );
        assert_eq!(answer, IndexAnswer::Refused(refusal));
    }

    /// The Python's `urlopen` followed a redirect. This does not: the sparse index answers its paths
    /// directly, so a redirect is refused, and the run stops, rather than send the request on to an
    /// address the run did not name.
    #[test]
    fn a_redirect_is_refused_rather_than_followed_on_the_wire() {
        let listener = Listener::answering(|path| {
            if path == "/elsewhere" {
                reply(200, &[], &entries("htmlspec-cli", &["9.8.7"]))
            } else {
                reply(301, &[("Location", "/elsewhere")], "")
            }
        });
        let answer = listener
            .registry()
            .ask(&name("htmlspec-cli"), &version("9.8.7"));
        assert_eq!(
            answer,
            IndexAnswer::Refused(IndexRefusal::Status {
                name: name("htmlspec-cli"),
                status: 301,
            })
        );
        assert_eq!(listener.received().len(), 1, "the redirect was followed");
    }

    #[test]
    fn a_request_that_does_not_complete_is_refused_with_curls_message() {
        // The listener reads the request and closes the connection without a word.
        let listener = Listener::answering(|_| Vec::new());
        match listener
            .registry()
            .ask(&name("htmlspec-cli"), &version("9.8.7"))
        {
            IndexAnswer::Refused(IndexRefusal::Incomplete {
                name: asked,
                exit,
                stderr,
            }) => {
                assert_eq!(asked, name("htmlspec-cli"));
                assert!(matches!(exit, Some(code) if code != 0), "{exit:?}");
                assert!(stderr.starts_with("curl:"), "{stderr}");
            }
            other => panic!("a request with no answer answered {other:?}"),
        }
    }

    /// The command line, as the design states it: `-q` first, a timeout of the whole request, the
    /// status written out, and nothing that drops an answer's status or headers or follows it
    /// elsewhere.
    #[test]
    fn the_command_line_keeps_the_status_and_the_headers() {
        let registry =
            CurlRegistry::new(CRATES_IO_INDEX, user_agent(), Proxies::FromTheEnvironment);
        let arguments: Vec<String> = registry
            .arguments(
                "https://index.crates.io/ht/ml/htmlspec-cli",
                Path::new("/s/headers"),
                Path::new("/s/body"),
            )
            .into_iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            arguments,
            [
                "-q",
                "--silent",
                "--show-error",
                "--max-time",
                "30",
                "--user-agent",
                USER_AGENT,
                "--dump-header",
                "/s/headers",
                "--output",
                "/s/body",
                "--write-out",
                "%{http_code}",
                "https://index.crates.io/ht/ml/htmlspec-cli",
            ]
        );
        for refused in ["--fail", "-f", "--fail-with-body", "--location", "-L"] {
            assert!(
                !arguments.iter().any(|argument| argument == refused),
                "{refused}"
            );
        }
    }

    /// Each request's directory is new, its owner's alone, and gone when the request is done.
    #[test]
    fn each_request_gets_a_private_directory_of_its_own() {
        let one = Scratch::new().expect("a directory");
        let two = Scratch::new().expect("another");
        assert_ne!(one.0, two.0);
        let mode = std::fs::metadata(&one.0)
            .expect("it exists")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700, "{mode:o}");
        let path = one.0.clone();
        drop(one);
        assert!(!path.exists());
    }

    #[test]
    fn the_index_path_follows_the_registry_layout() {
        for (crate_name, path) in [
            ("a", "1/a"),
            ("ab", "2/ab"),
            ("abc", "3/a/abc"),
            ("Serde", "se/rd/serde"),
            ("htmlspec-cli", "ht/ml/htmlspec-cli"),
        ] {
            assert_eq!(index_path(&name(crate_name)), path);
        }
    }

    #[test]
    fn the_last_response_s_headers_are_read() {
        let headers =
            "HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 429 Too Many\r\nretry-after: 5\r\n\r\n";
        assert_eq!(header(headers, "Retry-After"), Some("5"));
        assert_eq!(header("HTTP/1.1 429 X\r\n\r\n", "Retry-After"), None);
    }

    #[test]
    fn a_line_of_the_index_that_is_not_json_is_refused() {
        let answer = answer(
            200,
            "",
            b"{\"vers\":\"0.1.0\"}\nnot json\n",
            &name("htmlspec-cli"),
            &version("9.8.7"),
        );
        assert_eq!(
            answer,
            IndexAnswer::Refused(IndexRefusal::NotJson {
                name: name("htmlspec-cli"),
                line: 2,
                column: 2,
            })
        );
    }
}
