# The export: the ports' parity, the trees they export, and the guards they keep

Tasks 5.1 to 5.8, recorded on 2026-11-03 on the branch of the fifth pull request, which ports
`scripts/check-pull-request` to `cargo xtask check-pull-request` and switches the pull-request guard
to it, moves the private working repository's export and its scan to a Cargo workspace of their own
under the private repository's `publish/`, which never crosses, moves the publish runbook to 0.4.0,
and deletes the six Python files that remained but the four archived data files. The machine is the
one `the-suite-before-2026-10-30.md` names, an Apple M5 Max laptop, 18 cores and 64 GB, on macOS
27.2, with rustc 1.90.0 first on the search path. Nothing here is a goal or a gate: task 6.5
measures the suite after the last pull request, by task 1.2's method, against decision 18's goals.

The Python ran as the commit before the export's switch held it, `a577a6a`, from copies of
`publish/export`, `publish/publishing.py`, `publish/scan`, and `scripts/release_facts.py` in a
scratch directory, the export's copy matching the digest `publish/no-python` pinned, under the
development shell's Python 3.13.14. The Rust ran through the launcher, `publish/export`, as the
publish runbook runs it.

## The pull-request rules

`scripts/check-pull-request` and `cargo xtask check-pull-request` were each given the same title
with `--changed -` and `--labels-json`, and their standard output, standard error, and exit code
compared byte for byte.

| Cases | Agree |
| --- | ---: |
| This repository's last 50 merged titles, with `CHANGELOG.md` among the changed files | 50 of 50 |
| 22 titles: empty, work in progress in two spellings, a Conventional Commits prefix in two, 80 characters, too short by length and by words, opening with `#`, lowercase, a reporting opener in two forms, a full stop, a pull request's number, a quote in the title, a quote opening it, an accented capital, a dotless `i` in `wip`, five faults at once, white space at both ends, and the export's and the bootstrap's subjects | 22 of 22 |
| 7 of the changelog rule and the labels: untouched, nothing changed, the label with and without other labels, a title and changelog fault together, labels that are not JSON, labels that are not strings | 7 of 7 |

Against `origin/main` the two named the same one fault and exited 1, and against a base that does
not exist both printed `cannot diff against no-such-ref. Fetch it, or pass --changed with the file
list` and exited 2. The tool's `--title-only` with `--base` is a usage error, exit 2.

## The trees the two exports export

Both ran `--dry-run` of `1189745`, the head once the reads of what the pull request had changed were
recorded, one after the other under the suite lock, each running the whole suite inside the
exported tree.

| Target | Python | Rust | The tree, from both |
| --- | ---: | ---: | --- |
| An empty bare repository: the bootstrap's `README.md` and `LICENSE` | exit 0, 179 s | exit 0, 328 s | `0295794fd7accca473119020df89ae3e1914ef39` |
| A scratch bare repository holding the public repository's `main` at `4008948`, fetched read-only | exit 0, 183 s | exit 0, 328 s | `b6b3b5d81a655366cecab8cd37ce7deb01d7785c` |

Every line of their steps was the same: 1,671 files exported and 1 excluded, 24 rewrites making 213
occurrences in 126 file changes, a clean scan with 4 binary files each read at its blob id, `left
behind: publish`, and the suite passed. The one difference is the Rust's title step against the
public `main`, `Checking the title against the exported tree's own rules: cargo xtask
check-pull-request --title-only --title 'Export htmlspec 1.3.1 from 1189745cbdea'`, which built and
ran the exported tree's tools and took the title, where the Python ran the ref's script silently
before it exported anything. The Rust's extra time was its scan, whose expression engine was then
built unoptimised, as `cargo run` builds the dev profile; the next section gives the scan's time
since.

No exported text file holds a carriage return, so the Python's text-mode rewrite, which wrote every
rewritten file's line endings as `\n`, and the Rust's, which changes a file's bytes only where the
expected text stood, write the same bytes here.

## The scan

A scratch tree held a text file naming a Linux home directory by shape and Homebrew's excused
prefix, a PNG whose `tEXt` chunk names a macOS home directory by shape, a PNG whose `zTXt` chunk,
compressed, names a Linux one, a PNG with an innocent `tEXt` chunk, and a plain file, with a
readiness record that reads the innocent PNG at its blob, names one PNG without its blob, and leaves
the other unread. The home directories are of a made-up person, built at run time, and the patterns
are the real `publish/patterns.tsv`.

| Case | Exit, both | Output |
| --- | ---: | --- |
| `--redact`, with the readiness record | 1 | the same, byte for byte |
| without `--redact`, which prints each pattern, compared and not kept | 1 | the same, byte for byte |
| `--redact`, with no readiness record | 1 | the same, byte for byte |
| `--ref 1189745 --redact`, what the allowlist admits before any rewrite | 1 | the same, byte for byte: 281 matches, which the rewrites remove, and no binary unread |

The redacted report of the planted tree was the same from both: three matches, each named by its
file, its line, and its section, by two patterns that match home directories by shape, one in the
text file's second line, one in the `tEXt` chunk, and one in the `zTXt` chunk; the two binary
faults, the PNG named without its blob, with the blob to record, and the one left unread; and the
verdict, three private matches and two binary files not read as they are. On standard output both
listed the innocent PNG as read at its blob. No text of the pattern file is quoted here: what the
reasons say, like the expressions, stays in `publish/`.

The first comparison differed: the Rust exited 2 on the tree named by a relative path, because it
gave `git hash-object` each binary's path beneath the tree while running git in the tree. It names
them relative to the tree now, which `a_tree_named_by_a_relative_path_names_its_binaries_relative_to_it`
holds, and the four cases agree.

The first Rust scan, by `regress` alone, unoptimised, took over two minutes of each dry run above.
With the `regex` crate reading every expression it can, `regress` the one with look-ahead, both
optimised in the dev profile, and each line folded once, three rounds of `scan --ref 1189745` took 9
seconds each, against the Python's 6.

## The cases, pair by pair

**`publish/test`'s 65 cases** keep their names, the `test_` prefix dropped, but one.

- **`ExportTests`, 36,** in `export::tests`, from `bootstrap_makes_the_first_commit_of_an_empty_target`
  to `a_ref_without_the_title_rules_stops_the_export`. `test_a_title_the_guard_refuses_stops_the_export_before_anything_is_exported`
  is `a_title_the_guard_refuses_stops_the_export_before_anything_is_committed`, because the Rust
  checks the title in the exported tree; its title is too long by a version of three nineteen-digit
  numbers, a release's, where the Python's was too long by a pre-release suffix the Rust refuses as
  no release's version. The ref's own stricter rule of `the_title_rules_are_the_refs_and_not_the_checkouts`
  is the synthetic tree's `xtask/title-rules`, which the fake title runner applies, and the case also
  holds that the runner ran in the exported tree and not in the source.
- **`GitHubDeliveryTests`, 7,** in `export::tests`, from `a_github_delivery_names_the_account_before_it_pushes`
  to `a_target_off_github_is_pushed_as_before_without_asking_gh`, on a recorder of the real git that
  gives each command the refusing proxy and the system and global configuration switched off, and a
  fake GitHub, where the Python put a stub `gh` and a stub `git` first on `PATH`.
- **`HookTests`, 3,** in `export::tests`, driving `.githooks/pre-push` with bash as before.
- **`ScanTests`, 11,** in `scan::tests`, from `a_clean_tree_exits_zero` to
  `homebrews_linux_prefix_passes_the_real_patterns_and_another_home_does_not`, in-process where the
  Python ran the scan as a program.
- **`CredentialTests`, 6,** in `github::tests`, from
  `the_credential_helper_follows_the_entries_the_environment_carries` to
  `only_a_github_login_is_taken_for_the_account`.
- **`MetadataTests`, 2:** `identity::tests::only_a_noreply_address_is_accepted` and
  `github::tests::the_ruleset_state_is_read_from_the_api_payload`.

Twenty-six more hold what the Python's cases did not: the staged tree the suite runs on, each exit
code, a scan of a ref, that no test sets a process-wide environment variable, an identity's edges,
the GitHub addresses a repository is read from, the readiness record's reads, binaries, and
sign-off, the allowlist's reading and that it admits nothing `publish/no-python` names, the rewrite
table's escapes and refusals, the redacted report, every kind of PNG text chunk, a pattern file that
cannot be read, a tree named by a relative path, the lines and globs the files are read by, the
scratch directory, and the two lockfiles in step. The private workspace has 91 tests.

**`scripts/check-pull-request`** had no tests. The rules' tests, now in their own member's
`htmlspec_pull_request::tests`, hold nineteen of them: each title rule, the changelog rule, the
three-dot diff on a scratch repository and a base git does not hold, the quoting of a title as the
script quoted it, a list of changed files, each ending's exit code and lines, and the title alone;
a twentieth holds the command line the guard's binary and the tools' subcommand share. With the
report's test, which moved with it, the member has 21 tests and the tools crate 264, 284 between
them.

## The breaks

Each was made, run, recorded, and undone with `git checkout`:

| Group | Break | Failed |
| --- | --- | --- |
| the title rules' 19 | a title of 73 characters taken | `a_title_over_seventy_two_characters_is_refused` |
| the credential cases | `gh`'s helper written over the environment's own entries | 3: the credential case and two GitHub-delivery cases |
| the metadata cases | any address taken for a noreply one | 3: both identity cases and the export's |
| the scan's 11 | an allowed occurrence left in its line | 2: the allowed occurrence's case and the Homebrew prefix's |
| the scan's relative tree | the paths given to `git hash-object` as found | the test of its input |
| the export's 36 | a file changed since its read let through | 4: the changed, added, single, and many files' cases |
| the GitHub delivery's 7 | the scratch repository's fetch and push without `gh`'s credentials | `a_github_delivery_names_the_account_before_it_pushes` |
| the hook's 3 | a private checkout's push to the public repository let through | `a_private_checkout_cannot_push_anything_to_the_public_repository` |

The relative tree's break first went through its end-to-end case, which climbs past the root and
lands where it began whichever way the path is given, so the input is built by one function with a
unit test of its own, and that fails.

## The ratchet

Six Python files are deleted, 2,675 lines: `publish/test` (1,129), the export's Python (582),
`publish/publishing.py` (479), `scripts/check-pull-request` (312), `publish/scan` (135), and
`scripts/release_facts.py` (38). `publish/export` is now 23 lines of bash. Their six program entries
leave the lists, with `publish.yml`'s two pinned lines and the publish runbook's declaration, and
three of the flake's pinned comment lines become one with their rewording. The lists held 16 entries
and hold 5: in `xtask/python-debt`, the flake's 4 lines that name the interpreter; in
`publish/no-python`, the launcher's shell entry. No live Python file remains: the four allow-listed
archived data files are the repository's only Python.

**Group 1's negative cases,** rerun with the debt lowered, each made, staged where a case says
staged, run with the built tools binary, and undone. Every one failed the stage, exit 1:

| Case | The stage's finding |
| --- | --- |
| `scripts/tidy.py` added and staged | a Python file name, and a file under `scripts/` |
| a staged file whose first line is `#!/usr/bin/env python3` | a first line that runs python3, and begins with `#!` |
| a step running `python3` added to `check.yml` | `.github/workflows/check.yml:72: names python3` |
| `python3` in a second preflight declaration of the publish runbook | a preflight declaration names python3 |
| an allow-listed `prove-listing.py` deleted | allow-listed and absent |
| a line naming Python added to `flake.nix`, beside its pinned ones | `flake.nix:291: names python3` |
| a pinned line of `flake.nix` changed | names `python313`, and a pin the file no longer holds |
| `publish/no-python` moved away | `publish/export` begins with `#!` and carries the executable bit |
| a staged bash script `scripts/tidy` | a file under `scripts/`, begins with `#!`, and carries the executable bit |
| a staged file with the executable bit and no `#!` | carries the executable bit |
| a new hook, `.githooks/post-commit` | begins with `#!`, and carries the executable bit |
| `scripts/install-hooks` deleted | allow-listed and absent |
| a step running `python3` added to the composite action | `.github/actions/xtask/action.yml:36: names python3` |
| this group's: `scripts/check-pull-request` put back as it was at the base | under `scripts/`, a first line that runs python3, `#!`, and the executable bit |
| this group's: `publish/scan` put back as it was at the base | a first line that runs python3, `#!`, and the executable bit |
| this group's: a step running `python3` added to `pr-guard.yml` | `.github/workflows/pr-guard.yml:91: names python3` |

An untracked `scripts/tidy.py` passes, as no tracked file holds it. The tree as it stands passes: 18
findings excused by the allow-list's 7 paths, and 6 by the 5 entries.

## How long they take

**The stages,** in two passes of `scripts/check` by task 1.2's method, under the suite lock, at
`f26219c`, the first after the tools crate's library target was built and the second warm. Both
passed all 14 stages. Before, the release workflow's pull request's passes,
`the-release-workflow-2026-11-02.md`'s first and warm.

| Stage | Before, first | Before, warm | After, first | After, warm |
| --- | ---: | ---: | ---: | ---: |
| `format` | 0.4 | 0.5 | 0.6 | 0.5 |
| `lint` | 4.8 | 0.2 | 5.1 | 0.2 |
| `test` | 142.7 | 123.1 | 148.4 | 123.7 |
| `doctest` | 0.6 | 0.6 | 0.8 | 0.7 |
| `self-conformance` | 1.5 | 0.8 | 0.9 | 0.7 |
| `self-format` | 0.5 | 0.5 | 0.5 | 0.5 |
| `self-runbooks` | 2.2 | 2.2 | 2.2 | 2.2 |
| `self-runbooks-format` | 2.1 | 2.1 | 2.1 | 2.1 |
| `self-assets` | 0.8 | 0.9 | 0.8 | 0.8 |
| `no-python` | 1.3 | 0.7 | 1.5 | 0.7 |
| `readme-install` | 0.4 | 0.4 | 0.4 | 0.4 |
| `installed` | 1.2 | 1.2 | 1.1 | 1.1 |
| `notices` | 0.7 | 0.7 | 0.7 | 0.7 |
| `release` | 0.3 | 0.3 | 0.3 | 0.3 |
| **The pass** | **159.7** | **134.1** | **165.3** | **134.7** |

This group changes no stage's question. `test` gains the pull-request rules' 19 tests, and the
tools crate's tests now build as its library's; `no-python` reads the private workspace's sources, and five entries where it read sixteen.

**The tests.** The tools crate's 284 tests, before the rules moved to their own member, ran in 0.67
seconds in one process, twice; under `cargo
nextest`, one process per test, the pull-request rules' 19 took 0.55 seconds the slowest, the
three-dot diff, which makes a scratch repository. The private workspace's 91 tests ran in 13.3 and
13.7 seconds in one process, and in 14.6 under `cargo nextest`, where `publish/test`'s 65 took 40.8
seconds at the base. Its slowest are the export's cases, which make a source and a target repository
and run the export against them, each running git again and again while 90 others do the
same: `a_second_export_of_one_source_replaces_its_branch` at 11.4 seconds,
`a_delivery_exports_only_a_commit_main_holds` at 11.0, and `a_push_needs_the_tree_a_person_reviewed`
at 10.0. None sleeps. From an empty target directory the workspace built in 12 seconds, 189 crates,
and its tests in 15 more. `cargo nextest` marked one of the 91 "leaky"; the pull request's review
traced that to nextest's own concurrent spawning of processes on macOS, not to the test, and found
it harmless, so nothing was changed for it.

After the rules moved to their own member, a third pass, the first after that build, passed all 14
stages in 153.2 seconds, `test` 139.1.

**The jobs the design's risk on slower guards names,** before, by GitHub's timestamps: `pr-guard`'s
`guard` took 7 seconds at the base, and 8, 7, 7, 6, and 8 in its five latest runs, on the fourth
pull request; `publish.yml`'s `test` took 31 seconds at the base, and 33, 29, 31, 26, and 30 in its
five latest runs.

## The pull request's runs

Pull request 103, labelled `release-check` when it was opened, so the `opened` and `labeled` events
started two runs of `pr-guard` and of `release`, and each workflow's concurrency group cancelled the
first, 41670858014 and 41670858088. Every other run passed, at both heads.

**The guard,** whose cold time decides the design's risk:

| Run | Head | Cache | Seconds | Building |
| --- | --- | --- | ---: | ---: |
| 41670881207 | `6b4a8b4`, the tools crate | none | 137 | 109 |
| 41674418544 | `5af651a`, the rules' own member | restored from the first run's key | 38 | 4 |
| 41674418544, run again | `5af651a` | none, the pull request's two guard caches deleted first | 34 | 12 |
| 41674418544, run again | `5af651a` | its own | 30 | 0 |

The toolchain's install takes 10 seconds of each, and saving a cache 3 to 8. The Python's guard took
6 to 8 seconds, with no toolchain to install.

**`publish.yml`,** the private workspace's tests: 165 seconds at `6b4a8b4` from no cache, 138 of them
building and testing, and 63 at `5af651a` with the first run's cache, 23 of them testing. The
Python's took 26 to 33.

**The suites,** at `5af651a`: `check`, run 41674418531, passed all 14 stages in 901.4 seconds by its
markers, `test` 786.8, `lint` 75.7, and `no-python` 6.7; `check-without-nix`, run 41674418807,
passed them in 760.3, `test` 653.2 and `no-python` 6.5. Against the fourth pull request's last
runs, 883.7 and 508.7, this adds the rules' 21 tests and builds the rules' member; a shared runner's
load varies by more than either. At `6b4a8b4`, `check` took 1,160 seconds as a job and
`check-without-nix` 710.

**The release workflow,** run 41674418604 at `5af651a`: verify 386 seconds, plan 21, the builds 73,
320, 183, and 162, sums 22, and formula 36; at `6b4a8b4`, run 41670881454, verify 304, plan 22, the
builds 67, 462, 222, and 172, sums 26, and formula 41. This pull request changes none of its jobs.

## After the review

The pull request's review, focused on the export's safety, found that the scan's word boundary did
not fall where the Python's had, so lines the Python caught passed, and that a pattern the main
engine could not compile fell back silently to one whose word and digit escapes are ASCII alone. It
reproduced both by running the two scans on one planted tree. Nothing in the tree as it stood was
affected. It found six more gaps, three of them the Python's as well; design decision 3's note
after the review gives how each was settled.

**The scan's parity, again.** The Python's scan as `b59db05` held it, copied out of that commit, and
the Rust through the launcher, with the real pattern file, on a planted tree holding every repro the
review gave: the private repository's address followed by a combining mark; a letter, a combining
mark, and the organization's short name; a spacing mark before the name spelled apart; the short
name followed by each of a zero-width joiner, a zero-width non-joiner, an undertie, a fullwidth low
line, and a circled letter; a sibling repository's name written with a dotless `i`; an excused home
prefix followed by a dotless `i`; and, from before, home directories by shape in a text file and in
a `tEXt` and a compressed `zTXt` chunk, an excused line, and a word that continues past the name. A
PNG whose compressed text chunk does not inflate joined them. The private names were built in the
script from pieces.

| Case | Python | Rust |
| --- | --- | --- |
| the planted tree, redacted | exit 1, 14 match lines on 13 lines of 3 files | exit 1, the same 14 match lines byte for byte and the same standard output, and the chunk it cannot read |
| a custom pattern, `\w{3000}` before the organization's name, on 3,000 accented letters before it | exit 1, 1 match | exit 2: the pattern file refused, the expression too big to compile |
| a custom pattern, `\bfoo\d(?!x)`, on `foo` and an Arabic-Indic digit | exit 1, 1 match | exit 2: the pattern file refused, look-around in the pattern column |

So every line the Python caught the Rust catches, or the Rust refuses the pattern file rather than
report the tree clean. The excused line and the word that continues past the name pass both.

**The breaks.** Each fix was undone once, the private workspace's tests run, and the edit taken back:

| Fix undone | Failed |
| --- | --- |
| `\b` written as the `regex` crate's own | 2: `each_escape_is_written_with_the_meaning_it_had` and `a_boundary_falls_where_the_python_s_did` |
| look-around read in the pattern column | 2 |
| a compile error falling back to `regress` | `an_expression_the_automaton_cannot_read_is_refused_and_never_falls_back` |
| the dotless `i` left as it is | 2 |
| a path's components taken as given | `a_path_through_a_repository_s_metadata_is_refused` |
| a link's real destination left unchecked | `a_link_that_leads_outside_the_tree_is_refused_and_one_inside_is_kept` |
| the tree after the suite left uncompared | 2: the suite's and the title rules' writes |
| an undecodable chunk read as empty text | 2: the scan's case and the export's |
| any target taken as local | 4 |
| the user left in a shown address | 3 |
| metadata of two fields taken | `the_commit_s_metadata_is_its_author_its_committer_and_its_message` |
| a package only the private lockfile names let through | `a_package_resolved_another_way_is_a_disagreement` |

The path case's refusal was proved on trees made with `git mktree`: `docs/../../escape.txt` is
refused before anything is written, and nothing appears beside the scratch repository; so are
`docs/.git/config`, `docs/.GIT/config`, and `docs/./config`; and a link to a file outside, to
nowhere, to the scratch repository's `.git`, or to an absolute path is refused, as is a link that
reads as staying inside but leads outside through another link. The private workspace has 111 tests.

## After the second review

The second review swept every code point for the classes and the boundaries, gave every real row of
the pattern file a line it should catch, and found the scan missing nothing the Python caught. It
found four gaps, settled as design decision 3's note after the second review gives: an escape that
names a character passed the rule that an expression hold ASCII alone, and its hex digits were read
as the ASCII letters a `\b` stood beside; a local target that a `url.*.insteadOf` rewrite sends to
GitHub was reached with none of a GitHub delivery's guards; `\W` and `\D` matched less than the
Python's; and no path or link target was scanned, the Python's gap as well. Probing the `regex`
crate beside the first found two more readings of the Python's text it took silently, a brace such
as `{ 2}` read as a repetition and the `x` flag dropping a class's spaces, and `\<` and `\>` read
as word boundaries; the first two are refused, and the last two written as the characters.

**The scan's parity, a third time.** The Python's scan as `b59db05` held it, copied out, and the
Rust through the launcher, with the real pattern file, on the planted tree of the first review's
repros, to which were added a file whose name holds the organization's short name, a link whose
target holds a home directory by its shape, a PNG whose `eXIf` chunk holds one, and a PNG with bytes
after its `IEND` chunk.

| Case | Python | Rust |
| --- | --- | --- |
| the planted tree, redacted | exit 1, 14 match lines on 13 lines of 3 files | exit 1, the same 14 match lines byte for byte and the same standard output; 3 more matches, the path, the link's target, and the `eXIf` chunk; the chunk it cannot read; and the image with bytes after its end |

The redacted report printed the path and the link's target as `the tree's path 4 (its path,
withheld)` and `the tree's path 3 (its link's target, withheld)`, and nowhere the short name.

Then each custom pattern of both reviews, each on the lines the Python caught with it. Four of the
second review's spell a letter past ASCII by its code point, the dotted capital `I` or the dotless
`i` before the rest of a private name, or the `e` with an acute accent before a `\b`, in the
`\u`, `\U`, and `\x` forms; the Python matched the name in lower case, in capitals, and with the
dotted `I`, and `café` before a space but not before a letter.

| Custom pattern | Python | Rust |
| --- | --- | --- |
| `\w{3000}` before a name, on 3,000 accented letters before it | exit 1, 1 match | exit 2: too big to compile |
| `\bfoo\d(?!x)`, on `foo` and an Arabic-Indic digit | exit 1, 1 match | exit 2: look-around in the pattern column |
| a dotted `I` by its code point, `\u` form, before a name | exit 1, 3 matches | exit 2: the escape `\u` is not read |
| the same, `\U` form | exit 1, 3 matches | exit 2: the escape `\U` is not read |
| a dotless `i` by its code point, `\u` form, in a name | exit 1, 1 match | exit 2: the escape `\u` is not read |
| `caf`, an accented `e` by its code point, `\b`: `\u`, `\x`, and `\U` forms | exit 1, 1 match each | exit 2 each: the escape is not read |
| `a\Wb`, on `a`, U+0345, `b` | exit 1, 1 match | exit 2: `\W` is refused |
| `a{ 2}`, on itself | exit 1, 1 match | exit 2: a brace that opens no repetition |
| `x\<y`, on `x<y` | exit 1, 1 match | exit 1, the same match |

So every line the Python caught the Rust catches, or the Rust refuses the pattern file rather than
report the tree clean.

**Today's tree.** The scan of `HEAD`'s whole tree, before the rewrites, found the same 281 matches
of file text as before, and no path, no link target, no `eXIf` chunk, and no image with bytes after
its end. The four screenshots each carry an `eXIf` chunk of 68 bytes that holds no run of four
printable characters, and each ends at its `IEND`.

**The breaks.** Each fix undone once, the private workspace's tests run, and the file taken back
from the commit:

| Fix undone | Failed |
| --- | --- |
| an escape that names a character written as given | 2: `an_escape_that_names_a_character_is_refused` and the scan's case of the review's repros |
| `\W` written as a class | `a_complement_of_words_or_digits_is_refused` |
| a brace that opens no repetition taken as text | `a_brace_or_a_flag_read_otherwise_is_refused` |
| any inline flag taken | `a_brace_or_a_flag_read_otherwise_is_refused` |
| a local target's addresses left unresolved | `a_local_target_rewritten_to_github_is_refused_before_anything_is_reached` |
| a local target's address taken whatever it is | 2: that case and `a_local_target_must_stay_on_this_machine_both_ways` |
| no path or link target scanned | 2: the scan's case and the export's |
| a matched path printed in a redacted report | 2: the scan's case and the export's |
| an `eXIf` chunk skipped | `a_png_s_exif_metadata_is_scanned_as_text` |
| bytes after `IEND` let through | `bytes_after_a_png_s_end_refuse_the_tree` |

The local target's case gives the rewrite through `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_0`, and
`GIT_CONFIG_VALUE_0` to each git command alone, behind a proxy that refuses every connection, and
the export refuses before any `ls-remote`, `fetch`, or `push`, in a dry run as well as a bootstrap,
whether the rewrite sends the fetch to GitHub or only the push; a rewrite from one local path to
another still passes. The private workspace has 121 tests.
