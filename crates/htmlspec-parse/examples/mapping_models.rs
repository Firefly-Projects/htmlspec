//! Prints the projected model of each mapping fixture, for review.
//!
//! A fixture's `rules` and `faults` are authored by hand: they are the claim
//! about what the contract says. The `model` is what an implementation must
//! reproduce, and it is reviewed rather than trusted — this example prints it so
//! that a human compares it against `doc/projection.md` before it is committed.
//!
//! Run with `--write` to fill in each fixture's `model` field. Without it, the
//! example reports which fixtures would change, so it doubles as a check that
//! the committed models are current.

use std::path::PathBuf;

use htmlspec_parse::{parse_document, project};

fn main() {
    let write = std::env::args().any(|arg| arg == "--write");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/projection");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("the fixture directory is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();

    let mut stale = 0;
    for dir in dirs {
        let name = dir
            .file_name()
            .expect("named")
            .to_string_lossy()
            .into_owned();
        let html = std::fs::read_to_string(dir.join("input.html")).expect("input.html is readable");
        let outcome = parse_document(&html);
        assert!(
            outcome.is_recovery_free(),
            "{name}: the fixture's markup recovered: {:?}",
            outcome.recovered
        );
        let projection = project(&outcome.document);

        let model = serde_json::to_value(&projection.model).expect("the model serializes");
        let faults: Vec<&str> = projection.faults.iter().map(|f| f.rule).collect();

        let expected_path = dir.join("expected.json");
        let mut expected: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&expected_path).expect("readable"))
                .expect("expected.json parses");

        if expected["model"] == model {
            continue;
        }
        stale += 1;
        println!("--- {name} ---");
        println!("faults: {faults:?}");
        println!(
            "{}",
            serde_json::to_string_pretty(&model).expect("model prints")
        );
        for fault in &projection.faults {
            println!("  # {fault}");
        }

        if write {
            expected["model"] = model;
            std::fs::write(
                &expected_path,
                format!(
                    "{}\n",
                    serde_json::to_string_pretty(&expected).expect("expected prints")
                ),
            )
            .expect("expected.json is writable");
        }
    }

    if stale == 0 {
        println!("every fixture model is current");
    } else if write {
        println!("wrote {stale} fixture models");
    } else {
        println!("{stale} fixture models differ; re-run with --write after reviewing");
        std::process::exit(1);
    }
}
