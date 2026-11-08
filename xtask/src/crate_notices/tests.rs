//! The notices' cases, on synthetic crates built for each in a scratch directory, with the graph
//! shaped as `cargo metadata` writes it. The four cases of the license choice are in `license.rs`.
//! Nothing here reaches cargo or a network but the two cases that read this repository's own
//! committed notices and the formula's template.

use std::path::{Path, PathBuf};

use super::*;
use crate::scratch::Scratch;
use crate::workspace::graph::Graph;

const MIT_TEXT: &str = "Permission is hereby granted, free of charge.\n";

/// One crate on disk, and how `cargo metadata` describes it.
struct Spec<'a> {
    name: &'a str,
    version: &'a str,
    license: Option<&'a str>,
    files: &'a [(&'a str, &'a str)],
    authors: &'a [&'a str],
    repository: &'a str,
}

impl<'a> Spec<'a> {
    fn new(name: &'a str) -> Self {
        Self {
            name,
            version: "1.0.0",
            license: Some("MIT"),
            files: &[],
            authors: &[],
            repository: "",
        }
    }
    fn version(mut self, version: &'a str) -> Self {
        self.version = version;
        self
    }
    fn license(mut self, license: &'a str) -> Self {
        self.license = Some(license);
        self
    }
    fn no_license(mut self) -> Self {
        self.license = None;
        self
    }
    fn files(mut self, files: &'a [(&'a str, &'a str)]) -> Self {
        self.files = files;
        self
    }
    fn authors(mut self, authors: &'a [&'a str]) -> Self {
        self.authors = authors;
        self
    }
    fn repository(mut self, repository: &'a str) -> Self {
        self.repository = repository;
        self
    }
}

/// Crates on disk, and `cargo metadata` describing them.
struct Registry {
    root: PathBuf,
    packages: Vec<serde_json::Value>,
    edges: Vec<(String, String, Option<&'static str>)>,
    members: Vec<String>,
}

impl Registry {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            packages: Vec::new(),
            edges: Vec::new(),
            members: Vec::new(),
        }
    }

    fn add(&mut self, spec: Spec<'_>, member: bool) -> String {
        let directory = self.root.join(format!("{}-{}", spec.name, spec.version));
        std::fs::create_dir_all(&directory).expect("the crate's directory is made");
        std::fs::write(
            directory.join("Cargo.toml"),
            format!("[package]\nname = \"{}\"\n", spec.name),
        )
        .expect("its manifest is written");
        for (name, text) in spec.files {
            std::fs::write(directory.join(name), text).expect("its file is written");
        }
        let id = format!("{} {}", spec.name, spec.version);
        self.packages.push(serde_json::json!({
            "id": id, "name": spec.name, "version": spec.version, "license": spec.license,
            "authors": spec.authors, "repository": spec.repository,
            "manifest_path": directory.join("Cargo.toml"),
        }));
        if member {
            self.members.push(id.clone());
        }
        id
    }

    fn krate(&mut self, spec: Spec<'_>) -> String {
        self.add(spec, false)
    }

    fn member(&mut self, spec: Spec<'_>) -> String {
        self.add(spec, true)
    }

    fn depends(&mut self, dependent: &str, dependency: &str, kind: Option<&'static str>) {
        self.edges
            .push((dependent.to_owned(), dependency.to_owned(), kind));
    }

    fn forget(&mut self, dependency: &str) {
        self.edges.retain(|(_, to, _)| to != dependency);
    }

    fn graph(&self, reverse: bool) -> Graph {
        let mut packages = self.packages.clone();
        let mut nodes: Vec<serde_json::Value> = self
            .packages
            .iter()
            .map(|package| {
                let id = package["id"].as_str().unwrap_or_default();
                let deps: Vec<serde_json::Value> = self
                    .edges
                    .iter()
                    .filter(|(from, _, _)| from == id)
                    .map(|(_, to, kind)| {
                        serde_json::json!({ "pkg": to, "dep_kinds": [{ "kind": kind, "target": null }] })
                    })
                    .collect();
                serde_json::json!({ "id": id, "deps": deps })
            })
            .collect();
        if reverse {
            packages.reverse();
            nodes.reverse();
        }
        Graph::parse(
            &serde_json::json!({
                "packages": packages,
                "workspace_members": self.members,
                "workspace_root": self.root,
                "resolve": { "nodes": nodes },
            })
            .to_string(),
        )
        .expect("the synthetic graph reads")
    }
}

/// A case: a scratch directory holding the crates and the vendored standard texts, and a
/// workspace whose one member is `htmlspec-cli`.
struct Case {
    scratch: Scratch,
    registry: Registry,
    cli: String,
}

impl Case {
    fn new() -> Self {
        let scratch = Scratch::new("notices");
        scratch.write("licenses/MIT", MIT_TEXT);
        scratch.write("licenses/MPL-2.0", "Mozilla Public License Version 2.0\n");
        let mut registry = Registry::new(scratch.path().join("registry"));
        let cli = registry.member(Spec::new("htmlspec-cli"));
        Self {
            scratch,
            registry,
            cli,
        }
    }

    fn standard(&self) -> PathBuf {
        self.scratch.path().join("licenses")
    }

    fn depend(&mut self, dependency: &str) {
        let cli = self.cli.clone();
        self.registry.depends(&cli, dependency, None);
    }

    fn generate_with(&self, table: &str, reverse: bool) -> Result<String, Vec<NoticeFault>> {
        generate(&[self.registry.graph(reverse)], table, &self.standard())
    }

    fn generate(&self, table: &str) -> String {
        self.generate_with(table, false)
            .unwrap_or_else(|faults| panic!("refused: {}", said(&faults)))
    }

    fn refusal(&self, table: &str) -> String {
        match self.generate_with(table, false) {
            Ok(_) => panic!("the notices were written"),
            Err(faults) => said(&faults),
        }
    }
}

fn said(faults: &[NoticeFault]) -> String {
    faults
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The reviewed table: the header, and each row's cells.
fn table(rows: &[&[&str]]) -> String {
    let mut lines = vec![clarifications::COLUMNS.join("\t")];
    lines.extend(rows.iter().map(|row| row.join("\t")));
    format!("{}\n", lines.join("\n"))
}

#[test]
fn a_file_whose_name_states_the_license_is_its_text() {
    let mut case = Case::new();
    let dual = case
        .registry
        .krate(Spec::new("dual").license("MIT OR Apache-2.0").files(&[
            ("LICENSE-APACHE", "apache\n"),
            ("LICENSE-MIT", "the MIT text\n"),
        ]));
    let single = case.registry.krate(
        Spec::new("single")
            .license("Zlib")
            .files(&[("LICENSE.md", "the zlib text\n")]),
    );
    case.depend(&dual);
    case.depend(&single);
    let text = case.generate("");
    assert!(text.contains("--- MIT: LICENSE-MIT, as the crate ships it ---\n\nthe MIT text\n"));
    assert!(text.contains("--- Zlib: LICENSE.md, as the crate ships it ---\n\nthe zlib text\n"));
    assert!(!text.contains("apache"));
}

#[test]
fn a_plain_license_file_does_not_name_one_of_two_licenses() {
    let mut case = Case::new();
    let dual = case.registry.krate(
        Spec::new("dual")
            .license("MIT OR Apache-2.0")
            .files(&[("LICENSE", "which one?\n")]),
    );
    case.depend(&dual);
    assert!(case
        .refusal("")
        .contains("dual 1.0.0 is credited under MIT and ships no file whose name states it"));
}

#[test]
fn a_crate_with_no_file_and_no_row_is_refused() {
    let mut case = Case::new();
    let bare = case.registry.krate(Spec::new("bare"));
    case.depend(&bare);
    assert!(case
        .refusal("")
        .contains("bare 1.0.0 is credited under MIT"));
}

/// A manifest that states no license, which `cargo metadata` writes as null, is refused naming the
/// crate, as one that states an empty one is.
#[test]
fn a_crate_that_states_no_license_is_refused() {
    let mut case = Case::new();
    let silent = case.registry.krate(
        Spec::new("silent")
            .no_license()
            .files(&[("LICENSE", "text\n")]),
    );
    let blank = case.registry.krate(
        Spec::new("blank")
            .license(" ")
            .files(&[("LICENSE", "text\n")]),
    );
    case.depend(&silent);
    case.depend(&blank);
    assert_eq!(
        case.refusal(""),
        "blank 1.0.0 declares no license\nsilent 1.0.0 declares no license"
    );
}

#[test]
fn a_refused_license_names_the_crate() {
    let mut case = Case::new();
    let gpl = case.registry.krate(
        Spec::new("copyleft")
            .license("GPL-3.0-only")
            .files(&[("LICENSE", "GPL\n")]),
    );
    case.depend(&gpl);
    assert!(case
        .refusal("")
        .contains("copyleft 1.0.0 is licensed GPL-3.0-only"));
}

#[test]
fn a_row_supplies_a_vendored_text() {
    let mut case = Case::new();
    let bare = case
        .registry
        .krate(Spec::new("bare").authors(&["Ann Author"]));
    case.depend(&bare);
    let text = case.generate(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "licenses/MIT",
        "a reviewer",
    ]]));
    assert!(text.contains(&format!(
        "vendored as licenses/MIT, reviewed by a reviewer ---\n\nCopyright (c) Ann Author\n\n{MIT_TEXT}"
    )));
}

#[test]
fn a_row_for_a_version_the_lockfile_has_moved_past_is_refused() {
    let mut case = Case::new();
    let bare = case.registry.krate(Spec::new("bare").version("1.0.1"));
    case.depend(&bare);
    let message = case.refusal(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "licenses/MIT",
        "a reviewer",
    ]]));
    assert!(
        message.contains("a row for bare 1.0.0 under MIT, which the lockfile no longer credits")
    );
    assert!(message.contains("bare 1.0.1 is credited under MIT"));
}

#[test]
fn a_row_may_join_a_crate_file_and_a_vendored_text() {
    let mut case = Case::new();
    let bare = case.registry.krate(
        Spec::new("bare")
            .license("MIT/Apache-2.0")
            .files(&[("COPYING", "Copyright 2016 Ann.\n")])
            .authors(&["Ann"]),
    );
    case.depend(&bare);
    let text = case.generate(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "COPYING + licenses/MIT",
        "a reviewer",
    ]]));
    let crate_file = text.find("Copyright 2016 Ann.");
    let vendored = text.find("Copyright (c) Ann\n");
    assert!(crate_file.is_some() && vendored.is_some() && crate_file < vendored);
}

#[test]
fn an_authors_address_is_removed_from_the_copyright_line() {
    let mut case = Case::new();
    let bare = case
        .registry
        .krate(Spec::new("bare").authors(&["Ann Author <ann@example.org>", "Bo <bo@example.org>"]));
    case.depend(&bare);
    let text = case.generate(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "licenses/MIT",
        "a reviewer",
    ]]));
    assert!(text.contains("Copyright (c) Ann Author, Bo\n"));
    assert!(!text.contains("@example.org"));
}

#[test]
fn a_crate_with_no_authors_names_its_repository() {
    let mut case = Case::new();
    let bare = case
        .registry
        .krate(Spec::new("bare").repository("https://example.org/bare"));
    case.depend(&bare);
    let text = case.generate(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "licenses/MIT",
        "a reviewer",
    ]]));
    assert!(
        text.contains("Copyright the authors of bare, as https://example.org/bare names them\n")
    );
}

#[test]
fn a_rows_copyright_line_replaces_the_one_built_from_authors() {
    let mut case = Case::new();
    let bare = case
        .registry
        .krate(Spec::new("bare").authors(&["Ann <ann@example.org>"]));
    case.depend(&bare);
    let text = case.generate(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "licenses/MIT",
        "a reviewer",
        "Copyright (c) 2021 Ann Author",
    ]]));
    assert!(text.contains(&format!(
        "reviewed by a reviewer ---\n\nCopyright (c) 2021 Ann Author\n\n{MIT_TEXT}"
    )));
    assert!(!text.contains("Copyright (c) Ann\n"));
}

#[test]
fn a_row_may_leave_the_copyright_cell_empty() {
    let mut case = Case::new();
    let bare = case.registry.krate(Spec::new("bare").authors(&["Ann"]));
    case.depend(&bare);
    let text = case.generate(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "licenses/MIT",
        "a reviewer",
        "",
    ]]));
    assert!(text.contains("Copyright (c) Ann\n"));
}

#[test]
fn a_copyright_line_without_a_vendored_text_is_refused() {
    let mut case = Case::new();
    let bare = case
        .registry
        .krate(Spec::new("bare").files(&[("COPYING", "Copyright Ann.\n")]));
    case.depend(&bare);
    let message = case.refusal(&table(&[&[
        "bare",
        "1.0.0",
        "MIT",
        "COPYING",
        "a reviewer",
        "Copyright (c) Ann",
    ]]));
    assert!(message.contains("only a vendored standard text carries"));
}

#[test]
fn a_source_outside_the_crate_and_the_vendored_texts_is_refused() {
    let mut case = Case::new();
    let bare = case
        .registry
        .krate(Spec::new("bare").files(&[("COPYING", "Copyright Ann.\n")]));
    case.depend(&bare);
    for source in [
        "/etc/passwd",
        "../other/LICENSE",
        "licenses/../../LICENSE",
        "COPYING + ../LICENSE",
        "sub\\..\\LICENSE",
    ] {
        let message = case.refusal(&table(&[&["bare", "1.0.0", "MIT", source, "a reviewer"]]));
        assert!(
            message.contains("which is not a path within the crate or under licenses/"),
            "{source}: {message}"
        );
    }
}

/// This repository's committed notices and template, read where the suite reads them.
fn repository_file(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path))
        .unwrap_or_else(|error| panic!("{path} cannot be read: {error}"))
}

/// The license `htmlspec-assets` declares, as `cargo metadata` reads this workspace.
fn assets_license() -> String {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let graph =
        Graph::read(&cargo, &root, Target::X8664UnknownLinuxMusl).expect("this workspace reads");
    let license = graph
        .members()
        .find(|(_, package)| package.name.as_str() == ASSETS_CRATE)
        .and_then(|(_, package)| package.license.clone());
    license.expect("htmlspec-assets declares a license")
}

#[test]
fn the_check_fails_on_a_formula_that_drifted_from_the_notices() {
    let committed = repository_file(OUTPUT);
    let template = repository_file(render_formula::TEMPLATE);
    let assets = assets_license();
    let formula = |template| Formula {
        template,
        assets_license: Some(&assets),
        targets: &Target::ALL,
    };
    let passed = check(&committed, &committed, &formula(&template));
    assert_eq!(passed.exit, Exit::Clean, "{passed:?}");
    let drifted = template
        .replace("\"MPL-2.0\", ", "")
        .replace(", \"MPL-2.0\"", "");
    assert_ne!(drifted, template, "the template names MPL-2.0");
    let failed = check(&committed, &committed, &formula(&drifted));
    assert_eq!(failed.exit, Exit::Faults);
    assert!(
        failed.err.iter().any(
            |line| line.contains("the Homebrew formula's template") && line.contains("MPL-2.0")
        ),
        "{failed:?}"
    );
}

#[test]
fn the_check_fails_on_notices_the_lockfile_no_longer_matches() {
    let committed = repository_file(OUTPUT);
    let failed = check(
        &committed,
        &format!("{committed}\n"),
        &Formula {
            template: "",
            assets_license: None,
            targets: &Target::ALL,
        },
    );
    assert_eq!(failed.exit, Exit::Faults);
    assert!(failed.err[0].contains("does not match what Cargo.lock reaches"));
}

#[test]
fn an_mpl_crate_says_where_its_source_form_is() {
    let mut case = Case::new();
    let mpl = case.registry.krate(
        Spec::new("mozilla")
            .version("0.3.0")
            .license("MPL-2.0")
            .files(&[("LICENSE", "MPL\n")]),
    );
    case.depend(&mpl);
    assert!(case
        .generate("")
        .contains("Source form: https://crates.io/crates/mozilla/0.3.0\n"));
}

#[test]
fn an_apache_crates_notice_is_reproduced() {
    let mut case = Case::new();
    let apache = case.registry.krate(
        Spec::new("apache")
            .license("Apache-2.0")
            .files(&[("LICENSE-APACHE", "apache\n"), ("NOTICE", "Made by Ann.\n")]),
    );
    case.depend(&apache);
    assert!(case
        .generate("")
        .contains("--- NOTICE: NOTICE, as the crate ships it ---\n\nMade by Ann.\n"));
}

#[test]
fn a_workspace_build_dependency_is_credited_and_a_third_partys_is_not() {
    let mut case = Case::new();
    let schema = case.registry.member(Spec::new("htmlspec-schema"));
    let generator = case.registry.krate(
        Spec::new("typify")
            .license("Apache-2.0")
            .files(&[("LICENSE-APACHE", "apache\n")]),
    );
    let its_dependency = case
        .registry
        .krate(Spec::new("prettyplease").files(&[("LICENSE-MIT", "mit\n")]));
    let library = case
        .registry
        .krate(Spec::new("library").files(&[("LICENSE-MIT", "mit\n")]));
    let compiler_probe = case
        .registry
        .krate(Spec::new("cc").files(&[("LICENSE-MIT", "mit\n")]));
    let test_only = case
        .registry
        .krate(Spec::new("proptest").files(&[("LICENSE-MIT", "mit\n")]));
    case.depend(&schema);
    case.registry.depends(&schema, &generator, Some("build"));
    case.registry.depends(&generator, &its_dependency, None);
    case.depend(&library);
    case.registry
        .depends(&library, &compiler_probe, Some("build"));
    let cli = case.cli.clone();
    case.registry.depends(&cli, &test_only, Some("dev"));
    let found: Vec<String> = credited(&[case.registry.graph(false)])
        .expect("the binary crate is there")
        .into_iter()
        .map(|krate| krate.name.as_str().to_owned())
        .collect();
    assert_eq!(found, ["library", "prettyplease", "typify"]);
}

#[test]
fn the_output_does_not_depend_on_the_order_of_the_metadata() {
    let mut case = Case::new();
    for (name, file) in [("zeta", "zeta\n"), ("alpha", "alpha\n"), ("mid", "mid\n")] {
        let files = [("LICENSE-MIT", file)];
        let id = case.registry.krate(Spec::new(name).files(&files));
        case.depend(&id);
    }
    assert_eq!(case.generate_with("", false), case.generate_with("", true));
}

#[test]
fn the_union_over_targets_credits_each_targets_crates() {
    let mut case = Case::new();
    let mut linux = Registry::new(case.scratch.path().join("linux"));
    let linux_cli = linux.member(Spec::new("htmlspec-cli"));
    let linux_only = linux.krate(Spec::new("linux-only").files(&[("LICENSE-MIT", "mit\n")]));
    linux.depends(&linux_cli, &linux_only, None);
    let mac = case
        .registry
        .krate(Spec::new("mac-only").files(&[("LICENSE-MIT", "mit\n")]));
    case.depend(&mac);
    let found: Vec<String> = credited(&[case.registry.graph(false), linux.graph(false)])
        .expect("the binary crate is there")
        .into_iter()
        .map(|krate| krate.name.as_str().to_owned())
        .collect();
    assert_eq!(found, ["linux-only", "mac-only"]);
}

#[test]
fn a_comparison_names_each_crate_added_removed_or_changed() {
    let mut case = Case::new();
    let kept = case
        .registry
        .krate(Spec::new("kept").files(&[("LICENSE-MIT", "mit\n")]));
    let gone = case
        .registry
        .krate(Spec::new("gone").files(&[("LICENSE-MIT", "mit\n")]));
    case.depend(&kept);
    case.depend(&gone);
    let before = case.generate("");
    case.registry.forget(&gone);
    let new = case
        .registry
        .krate(Spec::new("new").files(&[("LICENSE-MIT", "mit\n")]));
    case.depend(&new);
    let after = case.generate("");
    assert_eq!(
        differences(&before, &after),
        [
            "added: new 1.0.0, credited under MIT",
            "removed: gone 1.0.0, which the lockfile no longer reaches",
        ]
    );
}

#[test]
fn a_second_version_beside_one_that_stays_is_not_an_upgrade() {
    let mut case = Case::new();
    let old = case.registry.krate(
        Spec::new("rand")
            .version("0.8.7")
            .files(&[("LICENSE-MIT", "mit\n")]),
    );
    case.depend(&old);
    let before = case.generate("");
    let new = case.registry.krate(
        Spec::new("rand")
            .version("0.9.5")
            .files(&[("LICENSE-MIT", "mit\n")]),
    );
    case.depend(&new);
    let after = case.generate("");
    assert_eq!(
        differences(&before, &after),
        ["added: rand 0.9.5 (beside 0.8.7), credited under MIT"]
    );
    assert_eq!(
        differences(&after, &before),
        ["removed: rand 0.9.5 (beside 0.8.7, which stays)"]
    );
}

/// A header that is not the tool's is named as the header, at its first differing line, and not as
/// a crate's license file; each is still named when both differ.
#[test]
fn a_header_that_differs_is_named_apart_from_a_text_that_does() {
    let mut case = Case::new();
    let only = case
        .registry
        .krate(Spec::new("only").files(&[("LICENSE-MIT", "the MIT text\n")]));
    case.depend(&only);
    let generated = case.generate("");
    let old_header = generated.replacen(
        "Generated by cargo xtask crate-notices from Cargo.lock. Change the tool or the table",
        "Generated by scripts/crate-notices from Cargo.lock. Change the script or the table",
        1,
    );
    assert_ne!(old_header, generated);
    assert_eq!(
        differences(&old_header, &generated),
        [
            "the header differs from the one the tool writes, first at its line 10: it reads \
          \"Generated by scripts/crate-notices from Cargo.lock. Change the script or the table, \
          never this\", and the tool writes \"Generated by cargo xtask crate-notices from \
          Cargo.lock. Change the tool or the table, never this\""
        ]
    );
    let old_text = generated.replace("the MIT text", "an older MIT text");
    assert_eq!(
        differences(&old_text, &generated),
        ["the crates are the same, and a text differs: a crate's license file changed"]
    );
    let both = old_header.replace("the MIT text", "an older MIT text");
    let said = differences(&both, &generated);
    assert_eq!(said.len(), 2, "{said:?}");
    assert!(said[0].starts_with("the header differs"));
    assert_eq!(
        said[1],
        "the crates are the same, and a text differs: a crate's license file changed"
    );
}

#[test]
fn a_version_replaced_by_another_is_an_upgrade() {
    let mut case = Case::new();
    let old = case.registry.krate(
        Spec::new("rand")
            .version("0.8.7")
            .files(&[("LICENSE-MIT", "mit\n")]),
    );
    case.depend(&old);
    let before = case.generate("");
    case.registry.forget(&old);
    let new = case.registry.krate(
        Spec::new("rand")
            .version("0.9.5")
            .files(&[("LICENSE-MIT", "mit\n")]),
    );
    case.depend(&new);
    assert_eq!(
        differences(&before, &case.generate("")),
        [
            "now at rand 0.9.5, credited under MIT",
            "no longer at rand 0.8.7"
        ]
    );
}

#[test]
fn render_formula_reads_the_credited_licenses() {
    let mut case = Case::new();
    let zlib = case.registry.krate(
        Spec::new("zlib")
            .license("Zlib")
            .files(&[("LICENSE-ZLIB", "z\n")]),
    );
    case.depend(&zlib);
    let text = case.generate("");
    assert_eq!(
        render_formula::credited_licenses(&text),
        Some(vec![String::from("Zlib")])
    );
    let targets: String = Target::ALL
        .iter()
        .map(|target| format!("url \"@BASE_URL@/htmlspec-@VERSION@-{target}.tar.gz\"\n"))
        .collect();
    let right = format!("{targets}license all_of: [\"MIT\", \"ISC\", \"Zlib\"]\n");
    let wrong = format!("{targets}license all_of: [\"MIT\", \"ISC\"]\n");
    assert_eq!(
        render_formula::template_faults(&right, Some("MIT AND ISC"), &text, &Target::ALL),
        []
    );
    assert!(
        !render_formula::template_faults(&wrong, Some("MIT AND ISC"), &text, &Target::ALL)
            .is_empty()
    );
}

/// Beyond the Python's cases: the table refuses a row whose license this project does not accept,
/// which could never be used, and a second row for one crate, version, and license.
#[test]
fn a_row_for_a_license_no_crate_is_credited_under_or_named_twice_is_refused() {
    let mut case = Case::new();
    let bare = case.registry.krate(Spec::new("bare"));
    case.depend(&bare);
    let message = case.refusal(&table(&[&[
        "bare",
        "1.0.0",
        "GPL-3.0-only",
        "licenses/MIT",
        "a reviewer",
    ]]));
    assert!(
        message.contains("names the license \"GPL-3.0-only\""),
        "{message}"
    );
    let row: &[&str] = &["bare", "1.0.0", "MIT", "licenses/MIT", "a reviewer"];
    let message = case.refusal(&table(&[row, row]));
    assert!(message.contains("names bare 1.0.0 MIT twice"), "{message}");
    let message = case.refusal("crate\tversion\n");
    assert!(message.contains("line 1 should be the header"), "{message}");
}

/// The file names each crate in its table and its text, and its header names every license.
#[test]
fn the_file_lists_each_crate_and_every_credited_license() {
    let mut case = Case::new();
    let dual = case.registry.krate(
        Spec::new("dual")
            .license("Apache-2.0 OR MIT")
            .files(&[("LICENSE-MIT", "the MIT text\n")]),
    );
    case.depend(&dual);
    let text = case.generate("");
    assert!(text.starts_with(HEADER));
    assert!(text.contains("\nCredited licenses: MIT\n\nCrates: 1\n\n  Crate  Version  License\n  dual   1.0.0    MIT\n\n"));
    assert!(text
        .contains("\ndual 1.0.0\nLicense: MIT\nLicensed by its authors as: Apache-2.0 OR MIT\n"));
    assert!(text.ends_with("the MIT text\n"));
}

/// The check on standard output, exit 0; a difference or a fault on standard error, exit 1.
#[test]
fn each_ending_of_the_check_has_its_exit_code() {
    let committed = repository_file(OUTPUT);
    let template = repository_file(render_formula::TEMPLATE);
    let assets = assets_license();
    let formula = Formula {
        template: &template,
        assets_license: Some(&assets),
        targets: &Target::ALL,
    };
    let clean = check(&committed, &committed, &formula);
    assert_eq!(clean.exit.code(), 0);
    assert!(clean.err.is_empty());
    assert_eq!(
        clean.out[1],
        "  ok  the Homebrew formula's template declares what the binary credits"
    );
    let stale = check(&committed, "", &formula);
    assert_eq!(stale.exit.code(), 1);
    assert!(stale.out.is_empty());
    assert_eq!(
        stale.err.last().map(String::as_str),
        Some(
            "crate-notices: run cargo xtask crate-notices to write it again, and read what changed"
        )
    );
}
