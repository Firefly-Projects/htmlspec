## 1. The check, before anything satisfies it

- [x] 1.1 Write `scripts/check-release` in Python, standard library only, with `#!/usr/bin/env python3` so `patchShebangs` rewrites it. Follow `scripts/check-prompt` as the precedent — including its habit of explaining in the file why it is a script
- [x] 1.2 Have it read three things from files and nothing from git: the workspace version from `Cargo.toml`, a recorded version per archived change, and the version headings in `CHANGELOG.md`. `flake.nix:39` gives the derivation a snapshot with no `.git`, so a check using `git log` would pass locally and fail in CI
- [x] 1.3 Assert three properties of the **archive**, not of the working change: every archived change records a version, every recorded version is distinct, and every recorded version has a changelog entry. A rule about work in progress cannot work in a stage with no warning level
- [x] 1.4 Make each failure name the archive directory, the version it recorded, and which of the three properties failed. Across seven archives "release hygiene failed" is not actionable
- [x] 1.5 Run it and **watch it fail** on all three properties: no archive records a version, so nothing is distinct and nothing has an entry. This is the only moment the check can be seen working against a repository that violates every rule
- [x] 1.6 Record the failure output as evidence

## 2. Record a version per archived change

- [x] 2.1 Record a version inside each archived change directory rather than in its name. It survives a rename and does not compete with the date prefix for position
- [x] 2.2 Record `0.1.0` for all seven existing archives, which is the truth — they all shipped under that version. Do not invent `0.1.0` through `0.7.0`: that writes a release history that never happened
- [x] 2.3 Do **not** rename the seven archive directories. Their names stay as they are
- [x] 2.4 Do **not** adopt the shipped program's `2026-08-14-0.2.0-name` convention. That convention is read by `last_archived_version` for projects `htmlspec init` bootstraps, and this repository has no `spec.nix` for that gate to read — matching it here would copy the product's shape into its own repository for no benefit
- [~] 2.5 **Dropped.** State in `archive.md` that archiving records the version. `archive.md` is `crates/htmlspec-assets/instructions/archive.md` — shipped prose compiled into the binary and installed into every project by `init` — so doing this breaks task 6.5, which requires no crate to change. It would also instruct every consumer to record a `VERSION` file, a convention invented for this repository that their own `htmlspec archive` does not read: the shipped gate recovers the version from the archive directory name. The boundary this marks is worth stating in the specification produced at archival — an obligation on this repository must not be written into prose the product ships

## 3. The changelog

- [x] 3.1 Write `CHANGELOG.md` at the repository root, one entry per version, each heading naming its version so the check can locate it
- [x] 3.2 Write the `0.1.0` entry as a **baseline** naming the seven changes it contains, and say in the entry that it is a reconstruction written from what the archives record. A reconstruction labelled as one beats an absence and beats a confident fiction
- [x] 3.3 Have the check treat versions at or below the baseline as one release rather than seven, so recording the truth about the seven does not fail the distinctness rule
- [x] 3.4 Comment that special case in the script as self-retiring: it stops applying the moment the version advances past `0.1.0`, and a later reader must not delete it while the seven archives exist
- [x] 3.5 Keep the entry prose. The check locates a heading and reads nothing inside it

## 4. Wire it into the suite

- [x] 4.1 Add the stage to `scripts/check` as `release`, **not** `lint`. That name is `cargo clippy`, and a second one would make `FAILED: lint` ambiguous
- [x] 4.2 Place it last, after `prompt`. The script's ordering comment says cheapest-first so a contributor hears about a missing newline quickly, and this stage is fast but least likely to be what someone is iterating on
- [x] 4.3 Confirm it needs **no new build input**: `pkgs.python3` is already in `nativeBuildInputs` for `check-prompt`. The last stage added to this suite took CI red for five merges over a missing input
- [x] 4.4 Confirm `patchShebangs` covers it. The derivation names each script explicitly, so a new one must be added to that list — this is the same fault that hid the `check-prompt` failure one level in
- [x] 4.5 Re-run the check from task 1 and confirm it now passes

## 5. Prove it in the sandbox, not just locally

- [x] 5.1 Run `nix build .#checks.aarch64-darwin.suite` and confirm it exits 0
- [x] 5.2 Read the build log and confirm `=== release ===` appears and the stage ran. A stage that silently does nothing also exits 0, which is the failure mode this project has already met once
- [x] 5.3 Confirm the suite reports nine checks rather than eight
- [x] 5.4 Run `./scripts/check` locally and confirm the same result, since a stage that disagrees between the two is the divergence this suite exists to prevent

## 6. Prove it catches the thing it exists for

- [x] 6.1 Temporarily remove a changelog entry and confirm the stage fails naming the version with no entry. Restore it
- [x] 6.2 Temporarily record a duplicate version on two archives above the baseline and confirm the distinctness rule fails. Restore them
- [x] 6.3 Temporarily remove a recorded version and confirm the stage fails naming that archive. Restore it
- [x] 6.4 Record all three as evidence. A check whose failures were never observed is a check that has only been seen passing
- [x] 6.5 Confirm no crate changed: `git diff --stat crates/` is empty. This change touches the repository's suite and not the shipped program
- [x] 6.6 Run `cargo test` and confirm nothing regressed

## 7. The contract

- [x] 7.1 Add to `openspec/specs/toolchain-packaging/spec.md` the obligation that the project's own suite checks release hygiene: an archived change records its version, recorded versions are distinct, and each has a changelog entry
- [x] 7.2 State that the suite's checks read the working tree rather than git history, because the sandbox the suite runs in receives a source snapshot with no repository. This is a constraint on every future stage, not only this one
- [x] 7.3 State that the project keeps a changelog with one entry per version. This is an obligation on this repository, and it is separate from anything the shipped program requires of a project it bootstraps
- [x] 7.4 Apply as a partial update, leaving every requirement this change does not name intact, and run `openspec validate --all`
