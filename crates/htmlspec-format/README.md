# htmlspec-format

The canonical serializer for HTMLSpec documents, so a later diff of a document shows a change in
meaning rather than a change in whitespace.

One of the six crates the `htmlspec` command is built from. They are published so the command
installs with `cargo install --locked htmlspec-cli`, they release together, and each pins the
others to exactly its own version. The project, its contract, and the other ways to install the
command are at https://github.com/Firefly-Projects/htmlspec.

Licensed under the MIT license, in `LICENSE`.
