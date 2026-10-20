# htmlspec-assets

The files an HTMLSpec document renders with, embedded at build time: the templates a new
document starts from, the stylesheets and enhancement script, and the pinned Mermaid and Vega
libraries that draw its diagrams and charts. `assets/vendor.lock.json` records where each
vendored file came from and its digest.

One of the six crates the `htmlspec` command is built from. They are published so the command
installs with `cargo install --locked htmlspec-cli`, they release together, and each pins the
others to exactly its own version. The project, its contract, and the other ways to install the
command are at https://github.com/Firefly-Projects/htmlspec.

This crate's own files are under the MIT license, in `LICENSE`. Mermaid is under the MIT license,
and Vega, Vega-Lite, and Vega-Embed are under the BSD-3-Clause license. Each is a bundle that
carries libraries of its own, under MIT, BSD-3-Clause, ISC, Apache-2.0, and the Unlicense, so
the crate declares `MIT AND BSD-3-Clause AND ISC AND Apache-2.0 AND Unlicense`. Every text is in
`THIRD-PARTY-LICENSES`, with the packages each one covers.
