//! The survey, run against this repository's own OpenSpec project.
//!
//! This repository is the migration's test case: it ships a program arguing HTML is the better
//! format for a specification, and it kept its own specifications in Markdown until it migrated
//! them. So the survey has a real source project to run against without constructing one, and
//! these tests use it. Since the migration, the project is governed from `htmlspec/`, and the
//! OpenSpec project it came from is kept, as it stood when it was migrated, in
//! `fixtures/migration-source/openspec/`.
//!
//! **The counts are recomputed rather than written down, and that is the point.** The change's
//! own tasks document asked for the measured figures to be asserted literally — 161 markdown
//! files, 9 capabilities, 16 archived changes. Those figures were themselves a correction: the
//! tasks document had been written against a snapshot four weeks old and every one of its seven
//! counts was wrong by the time the work started. A test carrying literals would be the same
//! defect one layer down, failing the next time anybody adds an evidence file — which happened
//! five times while this change was being implemented.
//!
//! So each test below states a property that stays true as the project grows: the survey's
//! enumeration matches an independent walk of the same tree, every file classifies, the version
//! ordering is numeric, and the entities a plan will struggle with are named. A survey that
//! misses a file fails here. A survey that is merely counting a project that changed does not.
//! The source no longer grows, and the properties are still the right things to assert.

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// This repository's root, from the crate the test lives in.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The root the survey reads: the OpenSpec project this repository was migrated from, in
/// `openspec/` beneath it.
///
/// A fixture rather than the repository, because the repository's project is now `htmlspec/`,
/// which is not an OpenSpec project, and the survey refuses one. What the tests cross-check the
/// source against — the changelog, the release script, the schema, and `migrate.md` — is still
/// read from the repository.
fn source() -> PathBuf {
    repository().join("fixtures/migration-source")
}

/// Every file beneath `directory`, computed independently of the survey.
///
/// Deliberately a second implementation. A test comparing the survey against itself proves
/// nothing, and this is simple enough to be obviously right: descend, collect files, sort.
fn every_file(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![directory.to_owned()];
    while let Some(next) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The survey enumerates exactly the files an independent walk finds.
///
/// The property the coverage gate rests on. A plan is checked against this enumeration, so a
/// file the survey misses is a file the check cannot report — it goes missing with every check
/// passing, which is the failure that makes an omission here worse than a missing mapping.
#[test]
fn the_survey_enumerates_every_file_the_project_holds() {
    let root = source();
    let survey = htmlspec_cli_migrate_survey(&root);
    let independently = every_file(&root.join("openspec"));

    let surveyed: Vec<PathBuf> = survey
        .entities
        .iter()
        .map(|entity| root.join(&entity.path))
        .collect();
    let mut surveyed_sorted = surveyed.clone();
    surveyed_sorted.sort();

    assert_eq!(
        surveyed_sorted.len(),
        independently.len(),
        "the survey found {} files and an independent walk found {}",
        surveyed_sorted.len(),
        independently.len()
    );
    assert_eq!(
        surveyed_sorted, independently,
        "the survey and an independent walk disagree about which files the project holds"
    );
    // And the survey's own order is the sorted one, because a coverage failure is diagnosed by
    // a person diffing two payloads.
    assert_eq!(
        surveyed, surveyed_sorted,
        "the entities are not path-ordered"
    );
}

/// Every file in this repository's project classifies as a known kind.
///
/// `unrecognized` is a real variant and a plan must account for one, but a source project as
/// ordinary as this one should not produce any. A count above zero here means a kind exists
/// that the mapping table has no row for — which is exactly the condition that produced this
/// change's largest correction, when four entity kinds turned out to be unmapped.
#[test]
fn every_file_in_this_repository_classifies() {
    let survey = htmlspec_cli_migrate_survey(&source());
    let counts = survey.counts();
    assert_eq!(
        counts.get("unrecognized").copied().unwrap_or(0),
        0,
        "some files classify as no known kind: {:?}",
        survey
            .entities
            .iter()
            .filter(|entity| entity.kind.label() == "unrecognized")
            .map(|entity| entity.path.display().to_string())
            .collect::<Vec<_>>()
    );
    // And the kinds this repository is known to hold are all present, so a classifier that
    // collapsed two kinds into one would fail rather than merely counting differently.
    //
    // **A kind absent here has two possible causes, and the message names both.** The classifier
    // can have stopped recognizing it, or the files can be absent from the tree being surveyed.
    // The second is not hypothetical: the build sandbox receives a filtered snapshot rather than
    // the working tree, and `flake.nix` dropped every `.yaml` until this test failed in
    // continuous integration while passing locally. A message naming only the classifier sends a
    // reader to the wrong file, so it names the filter too.
    for kind in [
        "configuration",
        "accepted-specification",
        "change-document",
        "delta-specification",
        "recorded-version",
        "evidence",
        "converted-document",
        "change-configuration",
    ] {
        assert!(
            counts.get(kind).copied().unwrap_or(0) > 0,
            "no entity classified as {kind}, out of {counts:?}. Either the classifier stopped \
             recognizing that kind, or the files it reads are not in the tree being surveyed — \
             check `srcFilter` in flake.nix, which gives the build sandbox a filtered snapshot \
             and has to keep every extension a test reads"
        );
    }
}

/// Every file extension this repository's project holds reaches the tree being surveyed.
///
/// **The check that catches a source filter dropping a file, without needing a kind to disappear
/// first.** `every_file_in_this_repository_classifies` catches that case only where a whole kind
/// goes missing. An extension holding files of a kind that also arrives by another route would
/// vanish silently, and the survey would report a smaller project with every test passing.
///
/// This is the failure that reached continuous integration: `flake.nix` kept `.md`, `.html`,
/// `.json`, `.css`, `.js`, `scripts/`, and `VERSION`, and dropped `.yaml`. The comment above that
/// filter already stated the rule — "A file the suite reads must be in the source or the suite
/// passes locally and fails here" — one release before this arrived.
#[test]
fn every_extension_the_project_holds_reaches_the_survey() {
    let survey = htmlspec_cli_migrate_survey(&source());
    let mut extensions: BTreeMap<String, usize> = BTreeMap::new();
    for entity in &survey.entities {
        let extension = entity
            .path
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            // A file with no extension is its own case, and `VERSION` is why the filter has a
            // rule for it. Named rather than skipped, so dropping that rule fails here too.
            .unwrap_or_else(|| String::from("<none>"));
        *extensions.entry(extension).or_insert(0) += 1;
    }

    // Every extension the source's `openspec/` is known to hold. A file kind reaching the
    // survey through one extension and not another is the silent case this pins.
    for extension in ["md", "yaml", "html", "txt", "py", "<none>"] {
        assert!(
            extensions.contains_key(extension),
            "the survey found no {extension} file, out of {extensions:?}. This repository holds \
             them, so either they are absent from the tree being surveyed — check `srcFilter` in \
             flake.nix — or the survey stopped enumerating them"
        );
    }
}

/// The counts sum to the total, so a classifier miscounting two kinds in opposite directions
/// cannot pass by accident.
#[test]
fn the_counts_account_for_every_entity() {
    let survey = htmlspec_cli_migrate_survey(&source());
    let summed: usize = survey.counts().values().sum();
    assert_eq!(summed, survey.entities.len());
}

/// The greatest recorded version is ordered numerically, and is one this repository shipped and
/// recorded in its changelog.
///
/// `0.10.0` is ordered after `0.9.0`, and text comparison says otherwise. A survey comparing
/// version strings would report `0.9.0` here, a migrated project's declaration would be set to
/// it, and the next archival would pass without advancing past the project's real history —
/// which is the ordering the version gate exists to enforce.
#[test]
fn the_greatest_recorded_version_is_the_projects_latest() {
    let survey = htmlspec_cli_migrate_survey(&source());
    let greatest = survey
        .greatest_version()
        .expect("this repository's archived changes record versions");

    let changelog = std::fs::read_to_string(repository().join("CHANGELOG.md"))
        .expect("the changelog is readable");
    assert!(
        changelog.contains(&greatest),
        "the greatest surveyed version {greatest} appears in no changelog entry"
    );
    // Ordered after every version the survey found, which is what the declaration is set to.
    for change in &survey.changes {
        if let Some(version) = &change.version {
            assert!(
                numeric(version) <= numeric(&greatest),
                "{version} in {} is ordered after the greatest {greatest}",
                change.name
            );
        }
    }
}

/// This repository's shared version is reported, with every change that records it.
///
/// Seven changes record one version, and the destination requires recorded versions to be
/// distinct — with a release-record entry naming the changes that shipped together. A migrated
/// project whose release record lost that entry fails its own release check on arrival, so the
/// survey reports the collision rather than leaving migration to discover it afterwards.
#[test]
fn the_version_several_changes_share_is_reported_with_all_of_them() {
    let survey = htmlspec_cli_migrate_survey(&source());
    let shared = survey.shared_versions();
    assert!(
        !shared.is_empty(),
        "this repository has a shared version and none was reported"
    );

    // Cross-checked against the release script's own baseline, so the two cannot disagree about
    // which version the exception covers.
    let script = std::fs::read_to_string(repository().join("scripts/check-release"))
        .expect("the release script is readable");
    for version in shared.keys() {
        let baseline = version.replace('.', ", ");
        assert!(
            script.contains(&format!("({baseline})")),
            "version {version} is shared by several changes but is not the release script's \
             baseline, so a migrated project would fail its own release check"
        );
    }
}

/// The renames this repository records are surfaced, and the destination can now express them.
///
/// This was a tripwire: the survey reported the RENAMED sections, and the test asserted the schema
/// had no `"renamed"` state, so that the day one arrived the plan would stop calling a rename open.
/// The destination gained a rename as a requirement's `renamedFrom` field rather than as a state,
/// and a field named `renamedFrom` does not contain the string `"renamed"` in quotes, so the old
/// tripwire would have kept passing and kept calling the rename open. So it asserts both halves
/// directly now: the survey still reports every RENAMED section, because the plan has to carry each
/// one, and the destination has a place to carry it to.
#[test]
fn a_renamed_section_is_surfaced_and_the_destination_can_express_it() {
    let survey = htmlspec_cli_migrate_survey(&source());
    let renaming: Vec<&str> = survey
        .changes
        .iter()
        .filter(|change| {
            change
                .delta_sections
                .iter()
                .any(|section| section.contains("RENAMED"))
        })
        .map(|change| change.name.as_str())
        .collect();
    for expected in [
        "2026-08-17-project-bootstrap-cli",
        "2026-08-30-finish-the-archival",
    ] {
        assert!(
            renaming.contains(&expected),
            "{expected} renames a requirement and the survey did not report it: {renaming:?}"
        );
    }

    // Read from the published schema rather than asserted, so this fails if the field is withdrawn.
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repository().join("crates/htmlspec-schema/schema/document.schema.json"),
        )
        .expect("the schema is readable"),
    )
    .expect("the schema parses");
    assert!(
        schema["definitions"]["requirement"]["properties"]["renamedFrom"].is_object(),
        "the destination's schema has no renamedFrom field, so a rename has nowhere to go"
    );

    // And the instruction a migrating agent follows maps the section to that field rather than
    // to an open question, so the schema and the instructions cannot disagree about it.
    let instructions = std::fs::read_to_string(
        repository().join("crates/htmlspec-assets/instructions/migrate.md"),
    )
    .expect("the migrate instructions are readable");
    let row = instructions
        .lines()
        .find(|line| line.starts_with("| `## RENAMED Requirements`"))
        .expect("the mapping table has a row for a RENAMED section");
    assert!(
        row.contains("renamed-from"),
        "the mapping row does not carry a rename to the renamed-from field: {row}"
    );
    assert!(
        !row.contains("open question") && !row.contains("no state exists"),
        "the mapping row still records a rename as open: {row}"
    );
}

/// Every change's own configuration reaches the survey, as an independent reading of its file finds it.
///
/// project-migration's "A source change's own configuration is surveyed, and its predecessor becomes
/// its declaration": the survey reports, for each change, whether it holds a `.openspec.yaml`, every
/// key the file sets, and the value of `follows:`. A later stage carries `FOLLOWS` from this payload
/// rather than reading the file again, so a value the survey drops or misreads is one no later check
/// compares with the source.
///
/// A property rather than counts, as this file's header asks: the archive grows by a configuration
/// file with every change archived, and each one is compared as it arrives.
#[test]
fn every_change_configuration_reaches_the_survey() {
    let root = source();
    let survey = htmlspec_cli_migrate_survey(&root);
    let mut stating_a_predecessor = 0;
    let mut without_a_file = 0;
    for change in &survey.changes {
        let directory = if change.archived {
            root.join("openspec/changes/archive").join(&change.name)
        } else {
            root.join("openspec/changes").join(&change.name)
        };
        let file = directory.join(".openspec.yaml");
        let expected = std::fs::read_to_string(&file)
            .ok()
            .map(|text| independently_read(&text));
        if expected.is_none() {
            assert!(
                !file.exists(),
                "{} holds a configuration file this test could not read",
                change.name
            );
            without_a_file += 1;
        }
        if expected
            .as_ref()
            .is_some_and(|configuration| configuration.follows.is_some())
        {
            stating_a_predecessor += 1;
        }
        assert_eq!(
            change.configuration, expected,
            "the survey's configuration for {} differs from its file",
            change.name
        );
    }
    // So the comparison reaches both halves rather than passing over an archive that has neither.
    assert!(
        stating_a_predecessor > 0,
        "no change states a predecessor, so follows was never compared"
    );
    assert!(
        without_a_file > 0,
        "every change has a file, so a change without one was never compared"
    );
}

/// A configuration file's keys and `follows:` value, read by a second implementation.
///
/// A key is a line that starts in the first column, is not a comment, and holds a colon. The value of
/// `follows:` is what follows its colon, less a trailing comment and one pair of matching quotes.
fn independently_read(text: &str) -> harness::Configuration {
    let mut keys = Vec::new();
    let mut follows = None;
    let text = text.trim_start_matches('\u{feff}');
    for line in text.lines() {
        let first = line.chars().next();
        if first.is_none() || first.is_some_and(|c| c == '#' || c.is_whitespace()) {
            continue;
        }
        let Some(colon) = line.find(':') else {
            continue;
        };
        let key = line[..colon].trim();
        keys.push(key.to_owned());
        if key == "follows" && follows.is_none() {
            // Read by tokens rather than by the survey's scan, so the two can disagree: a quoted
            // value runs to its closing quote, and a plain one stops at the first token that opens
            // a comment. A change name holds no whitespace, so joining tokens loses nothing here.
            let rest = line[colon + 1..].trim();
            let value = match rest.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    rest[1..].split(quote).next().unwrap_or("").to_owned()
                }
                _ => rest
                    .split_whitespace()
                    .take_while(|token| !token.starts_with('#'))
                    .collect::<Vec<_>>()
                    .join(" "),
            };
            if !value.is_empty() {
                follows = Some(value);
            }
        }
    }
    harness::Configuration { keys, follows }
}

/// The schema field or value each new mapping row carries a source distinction to.
///
/// Read from the published schema, as the rename test reads `renamedFrom`, so a row naming a field the
/// schema withdrew or renamed fails here rather than passing.
#[test]
fn the_destination_expresses_each_distinction_migrate_md_maps() {
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repository().join("crates/htmlspec-schema/schema/document.schema.json"),
        )
        .expect("the schema is readable"),
    )
    .expect("the schema parses");
    let definitions = &schema["definitions"];
    assert!(
        definitions["task"]["properties"]["deviation"].is_object(),
        "a task has no deviation field, so a task done another way has nowhere to go"
    );
    let outcomes: Vec<&str> = definitions["verification"]["properties"]["outcome"]["enum"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect()
        })
        .unwrap_or_default();
    assert!(
        outcomes.contains(&"partial"),
        "the verification outcome does not enumerate partial: {outcomes:?}"
    );
    assert!(
        definitions["verification"]["properties"]["unchecked"].is_object(),
        "a verification has no unchecked field, so a partial check cannot say what it left"
    );
    let responses: Vec<&str> = definitions["risk"]["properties"]["response"]["enum"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect()
        })
        .unwrap_or_default();
    assert!(
        responses.contains(&"accepted") && responses.contains(&"mitigated"),
        "a risk's response does not enumerate accepted and mitigated: {responses:?}"
    );
}

/// Each distinction the destination expresses has a row in `migrate.md`'s mapping table, naming the
/// vocabulary it maps to, and none of them is recorded as open.
///
/// Modeled on `a_renamed_section_is_surfaced_and_the_destination_can_express_it`. project-migration's
/// "A source distinction the destination cannot express is recorded as open, not mapped to the
/// nearest state" now says that a distinction the destination can express is mapped, and the
/// instructions a migrating agent follows are where that reaches the agent. The schema half is
/// `the_destination_expresses_each_distinction_migrate_md_maps`.
#[test]
fn each_distinction_the_destination_expresses_has_a_row() {
    let instructions = std::fs::read_to_string(
        repository().join("crates/htmlspec-assets/instructions/migrate.md"),
    )
    .expect("the migrate instructions are readable");
    let row = |opening: &str| -> String {
        instructions
            .lines()
            .find(|line| line.starts_with(opening))
            .unwrap_or_else(|| panic!("the mapping table has no row opening {opening}"))
            .to_owned()
    };
    for (opening, words) in [
        (
            "| `## MODIFIED Requirements`",
            &["`ins`", "`del`", "`unchanged`", "`removed`"][..],
        ),
        ("| `## REMOVED Requirements`", &["`removed`"][..]),
        // The marker inside its list item, and the field inside its paragraph, so these two are
        // matched without the backticks a bare word would carry.
        (
            "| a task's checkbox",
            &["[~]", "`dropped`", "deviation"][..],
        ),
        ("| `.openspec.yaml`", &["`FOLLOWS`", "`created:`"][..]),
    ] {
        let found = row(opening);
        for word in words {
            assert!(
                found.contains(word),
                "the row opening {opening} does not name {word}: {found}"
            );
        }
        assert!(
            !found.contains("open question") && !found.contains("no state exists"),
            "the row opening {opening} records a distinction the destination expresses as open: \
             {found}"
        );
    }
    for word in ["`partial`", "`unchecked`", "`data-refs`", "`data-response`"] {
        assert!(
            instructions.contains(word),
            "migrate.md never names {word}, so an agent is not told the destination expresses it"
        );
    }
}

/// The survey writes nothing to the project it surveys.
#[test]
fn surveying_this_repository_writes_nothing() {
    let project = source().join("openspec");
    let before = every_file(&project);
    let _ = htmlspec_cli_migrate_survey(&source());
    assert_eq!(
        before,
        every_file(&project),
        "surveying this repository changed it"
    );
}

/// A version's numeric components, for ordering.
fn numeric(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

// The survey type lives in the binary crate, which an integration test cannot import. Rather
// than move it to a library for the sake of a test, the test re-runs the compiled program and
// reads its payload — which also exercises the command surface and the `--json` form that the
// coverage gate consumes, rather than only the function beneath them.
mod harness {
    use super::{BTreeMap, Path, PathBuf};

    /// A change, as the payload reports it.
    pub struct Change {
        pub name: String,
        pub archived: bool,
        pub version: Option<String>,
        pub delta_sections: Vec<String>,
        /// `None` where the payload carries `null`: the change has no configuration file.
        pub configuration: Option<Configuration>,
    }

    /// A change's own configuration, as the payload reports it.
    #[derive(Debug, Eq, PartialEq)]
    pub struct Configuration {
        pub keys: Vec<String>,
        pub follows: Option<String>,
    }

    /// An entity, as the payload reports it.
    pub struct Entity {
        pub path: PathBuf,
        pub kind: Kind,
    }

    /// An entity's kind, as the payload reports it.
    pub struct Kind(pub String);

    impl Kind {
        pub fn label(&self) -> &str {
            &self.0
        }
    }

    /// A survey, read back from the program's structured payload.
    pub struct Survey {
        pub entities: Vec<Entity>,
        pub changes: Vec<Change>,
        counts: BTreeMap<String, usize>,
        greatest_version: Option<String>,
        shared_versions: BTreeMap<String, Vec<String>>,
    }

    impl Survey {
        pub fn counts(&self) -> BTreeMap<&str, usize> {
            self.counts
                .iter()
                .map(|(kind, count)| (kind.as_str(), *count))
                .collect()
        }

        pub fn greatest_version(&self) -> Option<String> {
            self.greatest_version.clone()
        }

        pub fn shared_versions(&self) -> BTreeMap<String, Vec<String>> {
            self.shared_versions.clone()
        }
    }

    /// Runs `htmlspec migrate survey --json` against `root` and parses the payload.
    pub fn survey(root: &Path) -> Survey {
        let program = Path::new(env!("CARGO_BIN_EXE_htmlspec"));
        let output = std::process::Command::new(program)
            .args(["migrate", "survey", "--json"])
            .arg(root)
            .output()
            .expect("the program runs");
        assert!(
            output.status.success(),
            "the survey failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        crate::support::held(
            &["migrate", "survey", "--json", &root.to_string_lossy()],
            &String::from_utf8_lossy(&output.stdout),
        );
        let payload: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("the payload is JSON");

        Survey {
            entities: payload["entities"]
                .as_array()
                .expect("entities is an array")
                .iter()
                .map(|entity| Entity {
                    path: PathBuf::from(entity["path"].as_str().expect("a path")),
                    kind: Kind(entity["kind"].as_str().expect("a kind").to_owned()),
                })
                .collect(),
            changes: payload["changes"]
                .as_array()
                .expect("changes is an array")
                .iter()
                .map(|change| Change {
                    name: change["name"].as_str().expect("a name").to_owned(),
                    archived: change["archived"].as_bool().expect("archived is a boolean"),
                    version: change["version"].as_str().map(str::to_owned),
                    delta_sections: change["deltaSections"]
                        .as_array()
                        .expect("an array")
                        .iter()
                        .map(|section| section.as_str().expect("a section").to_owned())
                        .collect(),
                    configuration: configuration(change),
                })
                .collect(),
            counts: payload["counts"]
                .as_object()
                .expect("counts is an object")
                .iter()
                .map(|(kind, count)| {
                    (
                        kind.clone(),
                        usize::try_from(count.as_u64().expect("a count")).expect("a small count"),
                    )
                })
                .collect(),
            greatest_version: payload["greatestVersion"].as_str().map(str::to_owned),
            shared_versions: payload["sharedVersions"]
                .as_object()
                .expect("sharedVersions is an object")
                .iter()
                .map(|(version, changes)| {
                    (
                        version.clone(),
                        changes
                            .as_array()
                            .expect("an array")
                            .iter()
                            .map(|name| name.as_str().expect("a name").to_owned())
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    /// A change's `configuration`, which the payload carries as `null` or as an object.
    ///
    /// A payload that omits the field fails here rather than reading as `null`: "this change has no
    /// configuration file" is a fact the plan carries, and an omitted field cannot state it.
    fn configuration(change: &serde_json::Value) -> Option<Configuration> {
        let field = change
            .as_object()
            .and_then(|change| change.get("configuration"))
            .unwrap_or_else(|| panic!("{} carries no configuration field", change["name"]));
        if field.is_null() {
            return None;
        }
        let present = field
            .as_object()
            .expect("a configuration is null or an object");
        assert!(
            present.contains_key("follows"),
            "{}'s configuration omits follows rather than carrying null",
            change["name"]
        );
        Some(Configuration {
            keys: present["keys"]
                .as_array()
                .expect("keys is an array")
                .iter()
                .map(|key| key.as_str().expect("a key").to_owned())
                .collect(),
            follows: present["follows"].as_str().map(str::to_owned),
        })
    }
}

use harness::survey as htmlspec_cli_migrate_survey;
