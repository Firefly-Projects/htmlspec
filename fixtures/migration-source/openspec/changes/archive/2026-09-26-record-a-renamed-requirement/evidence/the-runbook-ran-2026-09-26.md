# The release runbook's run for 0.16.0, and whether it worked as written

Task 10.1, against `runbooks/release/tasks.html` at runbook version `0.1.0`. **It worked as written,
and the one known defect it carries was confirmed again.** Task 10.3 is left for archival, as the
runbook's fourth step says it will be.

## The increment (task 10.2)

**`0.15.0` → `0.16.0`, a minor increment, and the contributor confirmed it before the version
moved.** The change adds a contract field, six validation rules, an additive payload set, and new
stylesheet revisions, and nothing that validated before stops validating. That is the pattern of
every recent release. One reported fact moves: a requirement whose rename was never applied leaves
`reached`. That is the defect this change fixes rather than a reshaping of the payload, so it does not
argue for more than a minor increment. A patch would understate a new contract field, and `1.0.0`
is the README's separate decision.

## The four steps

1. **Advance the workspace version and relock offline.** `version = "0.16.0"` under
   `[workspace.package]`, then `cargo update --workspace --offline`. It locked exactly the seven
   workspace members, 0.15.0 → 0.16.0, and nothing else (`Cargo.lock`, 7 lines changed each way).
2. **Regenerate the installed instruction files.** `htmlspec init --tools claude,codex,pi`, run as
   `./target/debug/htmlspec` after a build, which reported `htmlspec 0.16.0`. The diff has the shape
   `req-version-before-regenerate` predicts: all 45 installed files move
   `generatedBy: htmlspec 0.15.0` → `0.16.0`, and the 15 files of the three workflows whose bodies
   changed (`sync`, `archive`, `migrate`, five install locations each) carry those bodies as well.
   `init` bootstrapped a stray `htmlspec/` and `spec.nix` at `0.0.0`, as
   `req-init-bootstraps-a-stray-project` says it will. Both were removed, and `scripts/check` ran
   afterwards with all 13 stages green, including `self-assets` and `installed`.
3. **Write the changelog entry.** `## 0.16.0 — a requirement can say what it used to be called`,
   opening with what a contributor gets and closing with what the release does not do.
4. **Confirm `scripts/check-release` passes.** `ok 21 archived change(s) record a version, and 16
   version(s) are in the changelog` and `ok declared version 0.16.0 has a changelog entry`. The
   archived change's `VERSION` is written at archival, so task 10.3 is where this is asked again.

## The known defect, still present

The previous run recorded that the regeneration step's bare `htmlspec init` resolves to the Nix
profile binary rather than the one just built. It still does, and the gap has widened:

```
$ which -a htmlspec
~/.nix-profile/bin/htmlspec
$ htmlspec --version
htmlspec 0.12.0 (unknown)
```

Following the step literally would stamp every installed file `0.12.0`, and it would also write the
instruction bodies of three releases ago over the ones this change edited. `scripts/check-installed`
would catch it, because it compares against `cargo run`. The run used the built binary by path, and
the runbook should say to. That is an edit to a canonical runbook document, and it owes a runbook
version and a changelog heading, so it is left for a change whose subject is the runbook.

## One observation about the environment

The runbook lists `cargo`, reached through `nix develop` or the rustup toolchain, and `python3` as
inputs. On this machine, `nix develop` supplies `cargo` and no `python3`, and the system has
`python3` and no `cargo`. `scripts/check` needs both, because `prompt`, `installed`, and `release`
are Python and the rest are cargo. It ran with the development shell's `cargo` directory prepended
to the system `PATH`. The runbook's inputs are right. They just do not arrive from one place.

## Task 10.3, after archival, and the step the runbook does not name

**The first `scripts/check-release` after the archive failed**, and it was right to:

```
!!! 2026-09-26-record-a-renamed-requirement records project version 0.16.0, at or after the 0.15.0
adoption of runbook publication, and carries no runbooks.json. A missing inventory after adoption is
a missing record rather than old history
```

Since 0.15.0 adopted runbook publication, every archive at or after that version carries a
`runbooks.json` inventory of the runbooks it published. `htmlspec archive` writes one. This
repository archives its changes with the OpenSpec workflow instead, which moves the directory and
knows nothing about runbooks, and so does the `VERSION` file it needs beside the archive. Neither the
runbook's fourth step nor the archive workflow says to write the inventory.

The inventory was generated from the committed runbook: name, `VERSION`, the SHA-256 of
`specification.html` and `tasks.html`, and the full text of `CHANGELOG.md`, encoded like the 0.15.0
inventory. **It is byte-identical to `2026-09-26-carry-a-runbook-to-another-machine/runbooks.json`.**
That is the expected result: this change did not touch the runbook, so it republishes release
`0.1.0` unchanged. The checker accepts that as one release recorded twice, not two releases at one
version. After it was written, `scripts/check-release` reported
`ok 22 archived change(s) record a version` and exited 0.

**A gap for the runbook's successor to close.** Step 4 says the check "passes on the declared version
now and the archived one later", and it does not say what the archive must hold for "later" to pass.
In this repository that is two files the archive workflow does not write, `VERSION` and
`runbooks.json`. Both are derivable, and the inventory is fully so. Changing the runbook is an edit
to a canonical document that owes the runbook a version, so it is left to a change whose subject is
the runbook.
