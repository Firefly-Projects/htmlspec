//! Reports recovered parse errors for the task 3.1 to 3.3 hand-authored documents.

use htmlspec_parse::parse_document;

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dogfood");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("dogfood directory is readable")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    paths.sort();
    for path in paths {
        let html = std::fs::read_to_string(&path).expect("readable");
        let outcome = parse_document(&html);
        println!(
            "{}: {} recovered, quirks={}",
            path.file_name().expect("named").to_string_lossy(),
            outcome.recovered.len(),
            outcome.quirks
        );
        for e in outcome.recovered {
            println!("  {} :: {}", e.code, e.message);
        }
    }
}
