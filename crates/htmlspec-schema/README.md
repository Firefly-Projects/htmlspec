# htmlspec-schema

The JSON Schema of the HTMLSpec document model, the specification of how HTML maps onto it, and
the Rust types generated from the schema.

One of the six crates the `htmlspec` command is built from. They are published so the command
installs with `cargo install --locked htmlspec-cli`, they release together, and each pins the
others to exactly its own version. The project, its contract, and the other ways to install the
command are at https://github.com/Firefly-Projects/htmlspec.

Licensed under the MIT license, in `LICENSE`.
