//! Task 1.5 spike: enumerate every parse error the pinned `html5ever` recovers
//! from, for benign and for damaged input.
//!
//! Run with `cargo run -p htmlspec-parse --example parse_error_spike`.

use std::cell::RefCell;
use std::rc::Rc;

use html5ever::tendril::TendrilSink;
use kuchikiki::parse_html_with_options;
use kuchikiki::ParseOpts;

fn quirks_mode(html: &str) -> String {
    let document = kuchikiki::parse_html().one(html);
    format!(
        "{:?}",
        document
            .document_node
            .as_document()
            .expect("document node")
            .quirks_mode()
    )
}

fn collect(html: &str, exact: bool) -> Vec<String> {
    let errors: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&errors);
    let mut opts = ParseOpts::default();
    opts.tokenizer.exact_errors = exact;
    opts.tree_builder.exact_errors = exact;
    opts.on_parse_error = Some(Box::new(move |message| {
        sink.borrow_mut().push(message.to_string());
    }));
    let _document = parse_html_with_options(opts).one(html);
    let collected = errors.borrow().clone();
    collected
}

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parse-errors")
        .canonicalize()
        .expect("fixture directory exists");
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("fixture directory is readable")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    entries.sort();

    for path in entries {
        let html = std::fs::read_to_string(&path).expect("fixture is readable");
        let name = path.file_name().expect("named file").to_string_lossy();
        println!("## {name}");
        println!("quirks: {}", quirks_mode(&html));
        let terse = collect(&html, false);
        let exact = collect(&html, true);
        println!("terse ({}):", terse.len());
        for message in &terse {
            println!("  - {message}");
        }
        println!("exact ({}):", exact.len());
        for message in &exact {
            println!("  - {message}");
        }
        println!();
    }
}
