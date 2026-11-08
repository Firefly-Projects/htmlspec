//! The tools' binary, `xtask`, which `cargo xtask TOOL` runs. Everything it does is the library's,
//! behind one entry point, so the binary's modules have one home.

fn main() -> std::process::ExitCode {
    htmlspec_xtask::run()
}
