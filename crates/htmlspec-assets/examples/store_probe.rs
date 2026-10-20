//! Reports this project's asset store, for the migration.
fn main() {
    let store = htmlspec_assets::Store::discover(std::path::Path::new("."));
    println!(
        "present: {}  addressed entries: {}",
        store.is_present(),
        store.len()
    );
    let dupes = store.duplicates();
    println!("duplicate sets: {}", dupes.len());
    let mut wasted = 0usize;
    for d in &dupes {
        println!("  {} at:", &d.sha256[..16]);
        for l in &d.locations {
            println!("      {l}");
        }
        wasted += d.locations.len() - 1;
    }
    println!("redundant copies: {wasted}");
}
