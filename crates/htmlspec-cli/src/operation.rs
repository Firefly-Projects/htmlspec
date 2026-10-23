//! Every operation the command line offers, and the two spellings of each.
//!
//! toolchain-packaging's "The command surface names its operations as OpenSpec names them" fixes
//! two spellings for every operation. The **OpenSpec spelling** carries OpenSpec's name for an
//! operation OpenSpec also provides, and this project's own name for one it does not: `validate`,
//! `store add`, `migrate promote`. The **request-verb spelling** says what the operation does to the
//! files it acts on, with one of htmx's five request keywords and an object: `get validity`, `post
//! registration`, `post promotion`.
//!
//! The two are stated here and nowhere else, as two exhaustive matches over one closed type, so an
//! operation added later does not compile until it has both. The command definition in `main.rs`
//! declares the same spellings to clap, and its tests hold the two to each other: every spelling
//! clap offers is one operation's, and every operation's two spellings parse to that operation with
//! the same arguments.
//!
//! The OpenSpec spelling stays the primary one. The program's own messages name it, and so do the
//! workflow instructions it installs, so a contributor who arrives from OpenSpec has one spelling to
//! learn. The request-verb spelling is for a reader who wants the effect on the face of the command.

/// Every operation the command line offers.
///
/// An operation is one thing the program does, whichever spelling asked for it. `new`, `store`, and
/// `migrate` are not operations: each groups two or three, and each of those is one here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Operation {
    Validate,
    Format,
    Assets,
    Archive,
    Init,
    List,
    Run,
    Preflight,
    Status,
    Sync,
    Show,
    StoreAdd,
    StoreList,
    StoreRemove,
    Instructions,
    NewChange,
    NewDocument,
    NewRunbook,
    MigrateSurvey,
    MigratePromote,
}

/// What an operation does to the files it acts on, named by one of htmx's request keywords.
///
/// htmx's `hx-get`, `hx-post`, `hx-put`, `hx-patch`, and `hx-delete` each issue the HTTP request of
/// that verb, and HTTP fixes what each does. The words are borrowed for their meaning and nothing
/// else: no operation sends a request, and none needs a server or a network.
///
/// **The verb names the most an operation writes in any invocation.** `get` is a promise about every
/// invocation, so an operation that writes only under an option takes the verb of that write, and
/// the option means under the verb what it means under the OpenSpec spelling: `put format --check`
/// writes nothing, as `format --check` does.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Verb {
    /// Never writes.
    Get,
    /// Creates something that was not there.
    Post,
    /// Creates or replaces a whole thing, and leaves it as one run did when run again.
    Put,
    /// Changes something that exists.
    Patch,
    /// Removes something.
    Delete,
}

impl Verb {
    /// The five verbs, in the order help lists them.
    pub const ALL: [Self; 5] = [Self::Get, Self::Post, Self::Put, Self::Patch, Self::Delete];

    /// The verb as a contributor types it.
    pub const fn word(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Patch => "patch",
            Self::Delete => "delete",
        }
    }

    /// What an operation spelled with this verb does to the files it acts on, as HTTP gives the
    /// verb its meaning.
    pub const fn meaning(self) -> &'static str {
        match self {
            Self::Get => "never writes",
            Self::Post => "creates something that was not there",
            Self::Put => {
                "creates or replaces a whole thing, and leaves it as one run did when run again"
            }
            Self::Patch => "changes something that exists",
            Self::Delete => "removes something",
        }
    }

    /// The verb a contributor typed, if the word is one.
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|verb| verb.word() == word)
    }
}

/// What every verb's help says about all five: they are a second spelling, and they send nothing.
const THE_VERBS_SEND_NOTHING: &str = "Both spellings run the same operation, with the same \
    arguments and options, and print the same output and exit status. The program's messages and \
    the instructions it installs name the operation's own spelling. The verbs name the effect, as \
    htmx's request keywords do, and send nothing: no request is made, and no server or network is \
    needed.";

impl Operation {
    /// Every operation, in the order the command line's help lists them.
    pub const ALL: [Self; 20] = [
        Self::Validate,
        Self::Format,
        Self::Assets,
        Self::Archive,
        Self::Init,
        Self::List,
        Self::Run,
        Self::Preflight,
        Self::Status,
        Self::Sync,
        Self::Show,
        Self::StoreAdd,
        Self::StoreList,
        Self::StoreRemove,
        Self::Instructions,
        Self::NewChange,
        Self::NewDocument,
        Self::NewRunbook,
        Self::MigrateSurvey,
        Self::MigratePromote,
    ];

    /// The OpenSpec spelling, as the words a contributor types after `htmlspec`: `["store", "add"]`.
    pub const fn spelling(self) -> &'static [&'static str] {
        match self {
            Self::Validate => &["validate"],
            Self::Format => &["format"],
            Self::Assets => &["assets"],
            Self::Archive => &["archive"],
            Self::Init => &["init"],
            Self::List => &["list"],
            Self::Run => &["run"],
            Self::Preflight => &["preflight"],
            Self::Status => &["status"],
            Self::Sync => &["sync"],
            Self::Show => &["show"],
            Self::StoreAdd => &["store", "add"],
            Self::StoreList => &["store", "list"],
            Self::StoreRemove => &["store", "remove"],
            Self::Instructions => &["instructions"],
            Self::NewChange => &["new", "change"],
            Self::NewDocument => &["new", "document"],
            Self::NewRunbook => &["new", "runbook"],
            Self::MigrateSurvey => &["migrate", "survey"],
            Self::MigratePromote => &["migrate", "promote"],
        }
    }

    /// The request-verb spelling: the verb, and the object that follows it.
    ///
    /// The verb is the most the operation writes in any invocation, measured rather than read
    /// from help (the change's `evidence/the-measurement-repeated-2026-10-09.md`). The object is
    /// the operation's own name where that name already says what it acts on, and otherwise what
    /// it creates, lists, or removes: `init` makes or refreshes a `project`, and the `store`
    /// operations act on a `registration`, never on the project it names.
    pub const fn request(self) -> (Verb, &'static str) {
        match self {
            Self::Validate => (Verb::Get, "validity"),
            Self::List => (Verb::Get, "list"),
            Self::Status => (Verb::Get, "status"),
            Self::Sync => (Verb::Get, "sync"),
            Self::Show => (Verb::Get, "show"),
            Self::Assets => (Verb::Get, "assets"),
            Self::Preflight => (Verb::Get, "preflight"),
            Self::Instructions => (Verb::Get, "instructions"),
            Self::StoreList => (Verb::Get, "registrations"),
            Self::MigrateSurvey => (Verb::Get, "survey"),
            Self::NewChange => (Verb::Post, "change"),
            Self::NewDocument => (Verb::Post, "document"),
            Self::NewRunbook => (Verb::Post, "runbook"),
            Self::StoreAdd => (Verb::Post, "registration"),
            Self::Run => (Verb::Post, "run"),
            // With `--staged` it puts a project where there was none.
            Self::MigratePromote => (Verb::Post, "promotion"),
            // A second run leaves the tree as the first did: measured, task 1.2.
            Self::Init => (Verb::Put, "project"),
            Self::Format => (Verb::Put, "format"),
            Self::Archive => (Verb::Patch, "archive"),
            Self::StoreRemove => (Verb::Delete, "registration"),
        }
    }

    /// The operation a request-verb spelling names, if it names one.
    pub fn requested(verb: Verb, object: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|operation| operation.request() == (verb, object))
    }

    /// The OpenSpec spelling as one string: `store add`.
    pub fn spelled(self) -> String {
        self.spelling().join(" ")
    }
}

/// The command line, with the help of its request verbs written from [`Operation`].
///
/// `main.rs` declares the verbs and their objects to clap. What each says in help is written here,
/// from the table above and from the help each operation's own spelling already carries, so an
/// operation is described once:
///
/// - the top-level help lists the operations as it always has, and then the five verbs under a
///   heading of their own, each with what it means and its objects;
/// - each verb's help lists its objects, each with the OpenSpec spelling it runs;
/// - each object's help is its operation's, after a line naming that operation. Its arguments and
///   options are its operation's already, because both spellings carry one `Args` struct.
///
/// Total over any command: a verb or an object the table does not name is left as clap declared it,
/// and `main.rs`'s tests hold the declaration to the table.
pub fn with_request_verbs(command: clap::Command) -> clap::Command {
    let described: Vec<(Operation, Option<String>, Option<String>)> = Operation::ALL
        .into_iter()
        .map(|operation| {
            let found = beneath(&command, operation.spelling());
            let about = found
                .and_then(clap::Command::get_about)
                .map(ToString::to_string);
            let long_about = found
                .and_then(|spelled| spelled.get_long_about().or(spelled.get_about()))
                .map(ToString::to_string);
            (operation, about, long_about)
        })
        .collect();

    let overview = overview(&command);
    command.after_help(overview).mut_subcommands(|declared| {
        let Some(verb) = Verb::named(declared.get_name()) else {
            return declared;
        };
        let about = format!("Run an operation that {}", verb.meaning());
        declared
            .about(about.clone())
            .long_about(format!(
                "{about}. Each object below runs the operation named beside it.\n\n\
                     {THE_VERBS_SEND_NOTHING}"
            ))
            .mut_subcommands(|object| {
                let Some((operation, about, long_about)) =
                    Operation::requested(verb, object.get_name()).and_then(|requested| {
                        described
                            .iter()
                            .find(|(operation, _, _)| *operation == requested)
                    })
                else {
                    return object;
                };
                let runs = format!("Runs `htmlspec {}`.", operation.spelled());
                object
                    .about(match about {
                        Some(about) => format!("{runs} {about}"),
                        None => runs.clone(),
                    })
                    .long_about(match long_about {
                        Some(long_about) => format!(
                            "{runs} The same operation, with the same arguments and \
                                 options.\n\n{long_about}"
                        ),
                        None => runs,
                    })
            })
    })
}

/// The subcommand `words` name beneath `command`, if every word names one.
fn beneath<'a>(command: &'a clap::Command, words: &[&str]) -> Option<&'a clap::Command> {
    words
        .iter()
        .try_fold(command, |current, word| current.find_subcommand(word))
}

/// The heading the top-level help lists the verbs under, after the operations.
///
/// Each verb's objects are listed in the order `command` declares them, which is the order the
/// verb's own help lists them in, and only those the table spells.
fn overview(command: &clap::Command) -> String {
    let width = Verb::ALL
        .into_iter()
        .map(|verb| verb.word().len())
        .max()
        .unwrap_or(0);
    let mut text = format!(
        "Request verbs:\n  Every operation also answers to a verb that says what it does to the \
         files it acts on, followed by an object. {THE_VERBS_SEND_NOTHING}\n\n"
    );
    for verb in Verb::ALL {
        let objects: Vec<&str> = command
            .find_subcommand(verb.word())
            .into_iter()
            .flat_map(clap::Command::get_subcommands)
            .map(clap::Command::get_name)
            .filter(|object| Operation::requested(verb, object).is_some())
            .collect();
        text.push_str(&format!(
            "  {:width$}  {}: {}\n",
            verb.word(),
            sentence(verb.meaning()),
            objects.join(", ")
        ));
    }
    text.push_str("\n  `htmlspec <verb> --help` names the operation each object runs.");
    text
}

/// `phrase` with its first letter capitalized, to open a line.
fn sentence(phrase: &str) -> String {
    let mut sentence = phrase.to_owned();
    if let Some(first) = sentence.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    sentence
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    /// [`Operation::ALL`] holds every operation, once, in its place.
    ///
    /// The match has no wildcard, so an operation added to the type does not compile here until this
    /// test places it, and placing it past the end of the list fails the index. `main.rs`'s tests
    /// close the other side: every spelling clap offers belongs to an operation in the list.
    #[test]
    fn every_operation_is_listed() {
        for (index, operation) in Operation::ALL.into_iter().enumerate() {
            let placed = match operation {
                Operation::Validate => 0,
                Operation::Format => 1,
                Operation::Assets => 2,
                Operation::Archive => 3,
                Operation::Init => 4,
                Operation::List => 5,
                Operation::Run => 6,
                Operation::Preflight => 7,
                Operation::Status => 8,
                Operation::Sync => 9,
                Operation::Show => 10,
                Operation::StoreAdd => 11,
                Operation::StoreList => 12,
                Operation::StoreRemove => 13,
                Operation::Instructions => 14,
                Operation::NewChange => 15,
                Operation::NewDocument => 16,
                Operation::NewRunbook => 17,
                Operation::MigrateSurvey => 18,
                Operation::MigratePromote => 19,
            };
            assert_eq!(
                placed, index,
                "{operation:?} is out of its place in the list"
            );
        }
        assert_eq!(Operation::ALL.len(), 20);
    }

    /// No two operations share an OpenSpec spelling, so each spelling runs one operation.
    #[test]
    fn the_openspec_spellings_are_distinct() {
        let mut seen: BTreeMap<&[&str], Operation> = BTreeMap::new();
        for operation in Operation::ALL {
            if let Some(first) = seen.insert(operation.spelling(), operation) {
                panic!(
                    "{first:?} and {operation:?} are both spelled `{}`",
                    operation.spelling().join(" ")
                );
            }
        }
    }

    /// No two operations share a request-verb spelling, so each spelling runs one operation.
    ///
    /// The object alone may repeat under two verbs, and does: `post registration` adds one and
    /// `delete registration` removes one. What must not repeat is the verb and the object together.
    #[test]
    fn the_request_verb_spellings_are_distinct() {
        let mut seen: BTreeMap<(Verb, &str), Operation> = BTreeMap::new();
        for operation in Operation::ALL {
            let (verb, object) = operation.request();
            if let Some(first) = seen.insert((verb, object), operation) {
                panic!(
                    "{first:?} and {operation:?} are both spelled `{} {object}`",
                    verb.word()
                );
            }
        }
    }

    /// Each operation's request-verb spelling is the one design decisions 2 and 3 fixed, and the one
    /// toolchain-packaging lists.
    ///
    /// The table is stated a second time here on purpose: it is what the implementation is checked
    /// against, so it cannot be read from the implementation. The match has no wildcard, so an
    /// operation added later has to be given its expected spelling here too.
    #[test]
    fn each_request_verb_spelling_is_the_one_the_design_fixed() {
        for operation in Operation::ALL {
            let expected = match operation {
                Operation::Validate => (Verb::Get, "validity"),
                Operation::List => (Verb::Get, "list"),
                Operation::Status => (Verb::Get, "status"),
                Operation::Sync => (Verb::Get, "sync"),
                Operation::Show => (Verb::Get, "show"),
                Operation::Assets => (Verb::Get, "assets"),
                Operation::Preflight => (Verb::Get, "preflight"),
                Operation::Instructions => (Verb::Get, "instructions"),
                Operation::StoreList => (Verb::Get, "registrations"),
                Operation::MigrateSurvey => (Verb::Get, "survey"),
                Operation::NewChange => (Verb::Post, "change"),
                Operation::NewDocument => (Verb::Post, "document"),
                Operation::NewRunbook => (Verb::Post, "runbook"),
                Operation::StoreAdd => (Verb::Post, "registration"),
                Operation::Run => (Verb::Post, "run"),
                Operation::MigratePromote => (Verb::Post, "promotion"),
                Operation::Init => (Verb::Put, "project"),
                Operation::Format => (Verb::Put, "format"),
                Operation::Archive => (Verb::Patch, "archive"),
                Operation::StoreRemove => (Verb::Delete, "registration"),
            };
            assert_eq!(
                operation.request(),
                expected,
                "{operation:?} is not spelled as the design fixed it"
            );
        }
    }

    /// No message the program prints names a request-verb spelling.
    ///
    /// toolchain-packaging's scenario "The program names the OpenSpec spelling": the program's own
    /// messages name `init`, never `put project`. Read from the source, because a message is
    /// printed only on the path that reaches it and most paths are never run by a test; the
    /// command-line tests in `tests/verbs.rs` check the messages the cases they run do print.
    ///
    /// A spelling counts where it is named as a command: after the program's name, or opening a
    /// backtick. The words alone are not enough, because prose uses them: `plan.rs` says a
    /// declaration has "nowhere to put project context". Comments are skipped, and so is this file,
    /// which states the spellings and writes the verbs' help.
    #[test]
    fn no_program_message_names_a_request_verb_spelling() {
        let sources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut pending = vec![sources];
        let mut files = 0;
        let mut found = Vec::new();
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).expect("the source directory reads") {
                let path = entry.expect("the entry reads").path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                let is_rust = path.extension().is_some_and(|extension| extension == "rs");
                let is_this_file = path.file_name().is_some_and(|name| name == "operation.rs");
                if !is_rust || is_this_file {
                    continue;
                }
                files += 1;
                let text = std::fs::read_to_string(&path).expect("the source reads");
                for (index, line) in text.lines().enumerate() {
                    let code = line.trim_start();
                    if code.starts_with("//") {
                        continue;
                    }
                    let invoked = Verb::ALL
                        .into_iter()
                        .map(|verb| format!("htmlspec {} ", verb.word()));
                    let quoted = Operation::ALL.into_iter().map(|operation| {
                        let (verb, object) = operation.request();
                        format!("`{} {object}", verb.word())
                    });
                    for spelling in invoked.chain(quoted) {
                        if code.contains(&spelling) {
                            found.push(format!(
                                "{}:{}: {}",
                                path.display(),
                                index + 1,
                                spelling.trim()
                            ));
                        }
                    }
                }
            }
        }
        assert!(files > 20, "only {files} source files were read");
        assert!(
            found.is_empty(),
            "the program names a request-verb spelling where it should name the OpenSpec one:\n  {}",
            found.join("\n  ")
        );
    }
}
