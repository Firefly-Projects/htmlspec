//! The renderer's cases: what it fills, what it refuses, and how it ends.

use std::path::Path;

use super::*;

const DIGESTS: [(&str, char); 4] = [
    ("aarch64-apple-darwin", 'a'),
    ("x86_64-apple-darwin", 'b'),
    ("x86_64-unknown-linux-musl", 'c'),
    ("aarch64-unknown-linux-musl", 'd'),
];

/// A `SHA256SUMS` for every target at `version`, each digest one repeated digit.
fn sums(version: &str) -> String {
    DIGESTS
        .iter()
        .map(|(target, digit)| {
            format!(
                "{}  htmlspec-{version}-{target}.tar.gz\n",
                digit.to_string().repeat(64)
            )
        })
        .collect()
}

fn template() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(TEMPLATE),
    )
    .expect("the template reads")
}

fn base(address: &str) -> BaseUrl {
    BaseUrl::new(address.to_owned())
}

#[test]
fn the_formula_names_each_target_s_tarball_and_digest_at_the_release_s_address() {
    let facts = Facts::read().expect("the facts read");
    let formula = formula(
        &template(),
        &sums("9.8.7"),
        &BaseUrl::releases(&facts),
        &Target::ALL,
    )
    .expect("the formula renders");
    assert!(formula.contains(
        "url \"https://github.com/Firefly-Projects/htmlspec/releases/download/v9.8.7/htmlspec-9.8.7-aarch64-apple-darwin.tar.gz\""
    ));
    for (_, digit) in DIGESTS {
        assert!(formula.contains(&format!("sha256 \"{}\"", digit.to_string().repeat(64))));
    }
    assert!(!formula.contains('@'), "a placeholder is left: {formula}");
}

#[test]
fn a_base_url_names_the_version_where_it_asks_and_loses_its_trailing_slash() {
    let rendered = formula(
        &template(),
        &sums("1.2.3"),
        &base("file:///tmp/dist-{version}///"),
        &Target::ALL,
    )
    .expect("the formula renders");
    assert!(rendered
        .contains("url \"file:///tmp/dist-1.2.3/htmlspec-1.2.3-x86_64-apple-darwin.tar.gz\""));
}

#[test]
fn sums_that_are_not_one_release_s_every_tarball_are_refused() {
    let full = sums("1.2.3");
    let linux_only: String = full
        .lines()
        .skip(2)
        .map(|line| format!("{line}\n"))
        .collect();
    assert_eq!(
        read_sums(&linux_only, &Target::ALL),
        Err(SumsFault::Lacks(vec![
            Target::Aarch64AppleDarwin,
            Target::X8664AppleDarwin
        ]))
    );
    assert_eq!(
        read_sums("", &Target::ALL),
        Err(SumsFault::Lacks(Target::ALL.to_vec()))
    );
    let other = full.replace("x86_64-apple-darwin", "riscv64gc-unknown-linux-gnu");
    assert_eq!(
        read_sums(&other, &Target::ALL),
        Err(SumsFault::Unbuilt {
            target: String::from("riscv64gc-unknown-linux-gnu")
        })
    );
    let two = full.replacen("1.2.3", "1.2.4", 1);
    assert!(matches!(
        read_sums(&two, &Target::ALL),
        Err(SumsFault::Versions { .. })
    ));
    let twice = format!("{full}{}", full.lines().next().unwrap_or_default());
    assert_eq!(
        read_sums(&twice, &Target::ALL),
        Err(SumsFault::Twice(Target::Aarch64AppleDarwin))
    );
    for broken in [
        full.replacen("  htmlspec", " htmlspec", 1),
        full.replacen(&"a".repeat(64), &"A".repeat(64), 1),
        full.replacen(".tar.gz", ".tgz", 1),
        // A version not written as a release writes it, which the Python's pattern took.
        full.replace("1.2.3", "01.2.3"),
    ] {
        assert!(
            matches!(
                read_sums(&broken, &Target::ALL),
                Err(SumsFault::Line { number: 1, .. })
            ),
            "{broken}"
        );
    }
}

#[test]
fn a_template_that_is_not_the_four_targets_or_leaves_a_placeholder_is_refused() {
    let template = template();
    let three = template.replace(
        "x86_64-apple-darwin.tar.gz",
        "x86_64-unknown-linux-gnu.tar.gz",
    );
    assert!(matches!(
        formula(&three, &sums("1.2.3"), &base("x"), &Target::ALL),
        Err(Refusal::Template(TemplateFault::Tarballs { .. }))
    ));
    let extra = format!("{template}# @SOMETHING_ELSE@ and @lower@ and @@\n");
    assert_eq!(
        formula(&extra, &sums("1.2.3"), &base("x"), &Target::ALL),
        Err(Refusal::Unfilled(vec![String::from("@SOMETHING_ELSE@")]))
    );
}

#[test]
fn the_template_s_license_is_read_from_its_line() {
    assert_eq!(
        template_license("  license all_of: [\"MIT\", \"ISC\"]  \nlicense all_of: [\"Zlib\"]\n"),
        ["MIT", "ISC"]
    );
    assert_eq!(
        template_license("license all_of: [\"\", \"MIT\"]\n"),
        [", "],
        "as the Python's pattern read an empty string"
    );
    assert!(template_license("license any_of: [\"MIT\"]\n").is_empty());
    assert_eq!(
        credited_licenses("x\nCredited licenses: MIT AND Zlib\n"),
        Some(vec![String::from("MIT"), String::from("Zlib")])
    );
    assert_eq!(credited_licenses("x\n"), None);
}

/// The message a drifted template gives, as the Python's formula notice block gave it.
#[test]
fn a_license_that_drifted_is_named_with_both_lists() {
    let targets: String = Target::ALL
        .iter()
        .map(|target| format!("url \"@BASE_URL@/htmlspec-@VERSION@-{target}.tar.gz\"\n"))
        .collect();
    let template = format!("{targets}license all_of: [\"MIT\", \"ISC\"]\n");
    let faults = template_faults(
        &template,
        Some("MIT AND ISC"),
        "Credited licenses: MIT AND Zlib\n",
        &Target::ALL,
    );
    assert_eq!(
        faults.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["the template's license is all_of ['MIT', 'ISC'], and the binary credits ['ISC', 'MIT', \
          'Zlib']: crates/htmlspec-assets/Cargo.toml declares MIT AND ISC for what it embeds, and \
          CRATE-LICENSES credits MIT AND Zlib for the crates it is built from"]
    );
    assert_eq!(
        template_faults(&template, Some("MIT OR ISC"), "", &Target::ALL),
        [TemplateFault::AssetsLicense(Some(String::from(
            "MIT OR ISC"
        )))]
    );
    assert_eq!(
        template_faults(&template, Some("MIT"), "no header\n", &Target::ALL),
        [TemplateFault::NoCredited]
    );
}

/// A refusal exits 1, and inputs that cannot be read exit 2.
#[test]
fn each_way_rendering_stops_has_its_exit_code() {
    assert_eq!(
        Stopped::Refused(Refusal::Unfilled(Vec::new()))
            .exit()
            .code(),
        1
    );
    assert_eq!(Stopped::CouldNotAsk(String::new()).exit().code(), 2);
    let missing = from_files(Path::new("/no/such/SHA256SUMS"), None);
    assert!(matches!(missing, Err(Stopped::CouldNotAsk(_))));
}
