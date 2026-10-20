//! Validates one file and prints the human-readable report.
//!
//! A hand-verification aid, used by task 6.9 before the CLI of group 7 exists.
//! `cargo run -p htmlspec-validate --example validate_file -- <path>`

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: validate_file <path>");
    let source = std::fs::read_to_string(&path).expect("the file is readable");
    let outcome = htmlspec_parse::parse_document(&source);
    let projection = htmlspec_parse::project(&outcome.document);
    let report = htmlspec_validate::engine::run(
        &outcome,
        &projection,
        &htmlspec_assets::Store::Absent,
        &htmlspec_validate::Scope::loose(),
        htmlspec_validate::rules::ALL,
        htmlspec_validate::Strictness::Default,
    );
    print!("{}", htmlspec_validate::render_human(&report));
}
