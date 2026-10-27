# HTMLSpec

> Spec-driven development for AI coding agents, with specifications you read in a browser.

<p align="center">
  <img src="img/hero.png" width="560" alt="A requirement from this repository's artifact-kinds specification: its SHALL statements, its four numbered steps, and the Mermaid flowchart drawn from them, with validation as a decision that loops back until it reports no error.">
</p>

Your agent writes the specification. You read it the way you read any good document: in a browser, with real typography, diagrams that render, and a stable link to every requirement. When something is wrong, you say so, and the agent fixes the file. You reload.

HTMLSpec is a spinoff of [OpenSpec](https://github.com/Fission-AI/OpenSpec/). It keeps OpenSpec's discipline — agree on the change, then build it — and makes HTML the canonical format for specifications, proposals, designs, tasks, and runbooks. The page above is not rendered from Markdown. An agent edited that file, and a browser opened it.

**The HTML is the source of truth.**

## Why HTML

Markdown made sense when people typed their documents by hand. Agent-driven development runs a different loop:

1. You tell an agent what you want to build.
2. The agent writes or updates the specification.
3. You review the result.
4. You tell the agent what to change.
5. You reload the browser.

Once the agent owns the editing, the question changes. It is no longer "What is easiest to type?" It is "What is easiest to understand, inspect, discuss, and approve?" HTML is the better answer.

A serious specification needs more than headings and bullet points. It needs architecture diagrams, state transitions, timelines, charts, exact requirements, and warnings. Markdown approximates them. HTML states them directly, and every HTMLSpec project carries a pinned, offline toolkit for it:

- [Mermaid](https://mermaid.js.org/) for architecture, sequence, state, and workflow diagrams
- [Vega-Lite](https://vega.github.io/vega-lite/) for charts and quantitative evidence
- native MathML for equations
- semantic HTML for requirements, scenarios, decisions, risks, tasks, and evidence
- print styles for a clean PDF or a paper review

The house style is flat and editorial: serif text, high information density, muted color, no dashboard clutter, no tracking, and no network connection. The result is still a directory of plain files in Git. It looks like a document when a person opens it.

Why not write Markdown and render it? Because two representations raise questions one never does. Which file is canonical? Did the renderer keep the meaning? Can the agent use a diagram the pipeline does not support? HTMLSpec has one representation. The file the agent edits is the file you review.

## What a project keeps

An HTMLSpec project lives in `htmlspec/` and keeps three kinds of document. Each is a page a person reads. Each is also structured so that an agent, a validator, and a Git diff can address every part of it.

### Specifications: the contract

Accepted specifications state what the project promises. A specification is built from requirement units, and every requirement carries the scenarios that make it checkable.

<p align="center">
  <img src="img/specification.png" width="720" alt="The runbook-portability specification: its masthead names the kind, contract version, and subject, a contents column sits on the left, and the first requirement shows its SHALL statements above two scenarios set out as When and Then.">
</p>

Every requirement and every scenario has a stable identifier, so the agent, the reviewer, and the diff all point at the same thing:

```html
<section class="requirement" id="req-theme-selection" data-change="added">
  <h2>Theme selection</h2>
  <p>The application must let the user select a light or dark theme.</p>

  <article class="scenario" id="scenario-theme-persists">
    <h3>The selected theme persists</h3>
    <dl>
      <dt>When</dt>
      <dd>The user selects the dark theme.</dd>
      <dt>Then</dt>
      <dd>The application uses the dark theme after the next launch.</dd>
    </dl>
  </article>
</section>
```

The browser gets a readable requirement. The agent gets explicit structure. Git gets stable, reviewable text.

### Changes: agree first, then build

A change is how a specification moves. It is a directory of four documents, and each one is a stage of a single argument:

| Document | What it carries |
| --- | --- |
| `proposal.html` | Why: the value, the need, and the scope |
| `design.html` | How: the decisions, the alternatives rejected, and the risks |
| `tasks.html` | The work, each task with its state |
| `specification.html` | The contract as built, written at archival |

<p align="center">
  <img src="img/change.png" width="720" alt="The tasks document of this repository's govern-this-repository-from-htmlspec change: numbered sections in a contents column, and each task with its identifier and a green DONE state.">
</p>

You drive the workflow from your agent. In Claude Code it looks like this:

```text
You:   /htmlspec:explore dark mode
Agent: I compared three approaches and drew the data flow.
       Option two fits best.

You:   /htmlspec:ff add-dark-mode
Agent: Created htmlspec/changes/add-dark-mode/ with a proposal, a design,
       and tasks. Each one validates. Open it in a browser.

You:   Show the failure path in the design.
Agent: Added a sequence diagram. Reload the page.

You:   /htmlspec:apply
Agent: Implemented the tasks and marked each one done.

You:   /htmlspec:verify
Agent: Checked the code against the tasks and the design.
       Every task is done, and nothing contradicts a decision.

You:   /htmlspec:archive
Agent: Wrote the specification, raised spec.nix to 0.2.0, merged the
       contract into the accepted specifications, and archived the change.
```

Archival is where a change becomes the contract. `htmlspec archive` refuses unless the project's version advances, and it refuses a specification that fails validation. Then it moves the whole change, all four documents, into `htmlspec/changes/archive/<date>-<name>/`. Nothing is thrown away: years later, a reader still finds the argument, the reasoning, and the work behind every requirement.

`htmlspec init` installs the same workflows for Codex, Cursor, and Pi, or into a plain `AGENTS.md`.

### Runbooks: procedures you run again

Some work is never finished: a release, a month-end close, an onboarding. A change archives once. A procedure runs again and again, so HTMLSpec keeps it as a runbook in `htmlspec/runbooks/<name>/`.

A runbook is two ordinary documents. Its specification states what the procedure is for and the hazards its history taught, as requirements. Its tasks document holds the steps, in the order those hazards force.

<p align="center">
  <img src="img/runbook.png" width="520" alt="This repository's release runbook as the change that established it drew it: four steps in a flowchart, with the hazard that puts the version bump first written on the edge between steps one and two.">
</p>

`htmlspec new runbook <name>` starts one. It writes both documents, a `VERSION` at 0.1.0, and a changelog, so the runbook validates, passes preflight, and can run straight away. The tasks document holds the steps and a figure of their order, and a runbook can carry any figure, chart, or formula a specification can. `htmlspec list --runbooks` lists the runbooks a project holds, each with its version and title, and `htmlspec run` named with no runbook lists them too. A new runbook's name is a plain directory name: `new runbook` refuses a path, a leading dot, `assets`, and the names a project's walk passes over, such as `target`.

Runbooks carry their own discipline:

- **Versioned.** A runbook has its own `VERSION` and `CHANGELOG.md`. Archival refuses changed steps that kept the old version.
- **Checked before they start.** `htmlspec preflight <runbook>` reports a missing tool, the wrong working directory, or a prerequisite run that has not passed, before step one, not at the step that needed it.
- **Recorded.** `htmlspec run <runbook> --as <record>` starts a run. The record copies the procedure it ran, so you always know which version ran and what it produced.
- **Ordered.** A runbook can require a finished, passed run of another runbook before its own run starts. `htmlspec run` refuses to start it until one exists, and names each record it found and why that record does not count.
- **Portable.** Copy a runbook to another machine and run it there. See [Carrying a runbook to another machine](#carrying-a-runbook-to-another-machine).

This repository releases itself with one: [`htmlspec/runbooks/release/`](htmlspec/runbooks/release/).

#### Running runbooks in order

`migrate` must not run before a backup has finished and passed. So a requirement in `migrate`'s specification declares it, in a note: `preflight/v3 {"kind":"run","runbook":"backup","atLeast":"1.4.0"}`. A finished run of `backup` at release 1.4.0 or later, within major 1, satisfies it, when it has a step done, every step is done or dropped, and every requirement it carries is recorded as passed. [Carrying a runbook to another machine](#carrying-a-runbook-to-another-machine) describes the declaration in full.

`htmlspec list --runbooks` shows what each runbook requires, beneath it, and whether that is met now. Here a backup has run, and nothing else has:

```text
backup              1.4.2  specification, tasks  Back up the database
create-account      0.1.0  specification, tasks  Create the account
deploy-service      0.1.0  specification, tasks  Deploy the service
  requires provision-database: unmet, no finished and passed run
migrate             0.1.0  specification, tasks  Migrate the schema
  requires backup at least 1.4.0: met by nightly
provision-database  0.1.0  specification, tasks  Provision the database
  requires create-account: unmet, no finished and passed run
```

So `migrate` can start now, and `provision-database` waits for an account. A prerequisite can read three other ways. It is `missing` when no runbook has the name, `self` or `cycle` when the requirements lead back to the runbook, and `unavailable` when a runbook on the way cannot have its declarations read, as preflight reports it. The listing also counts a prerequisite's open runs, met or not, so a backup still under way beside a passed one is in view, and `htmlspec run` with no runbook prints the same listing. `htmlspec --json list --runbooks` carries every state, the records that meet each prerequisite, and one cycle for each group of runbooks that lead back to one another. "Met" covers runs of other runbooks only: `htmlspec preflight <runbook>` reports machine checks.

`htmlspec list --runbooks --graph` draws the same order as a Mermaid flowchart, ready to paste into a document's `<pre class="mermaid">`. Each arrow points from a prerequisite to the runbook that requires it, solid green when met and dashed red when not. For the project above:

```mermaid
flowchart LR
  r1["backup"]
  r2["create-account"]
  r3["deploy-service"]
  r4["migrate"]
  r5["provision-database"]
  r1 -->|"met, at least 1.4.0"| r4
  r2 -->|"unmet"| r5
  r5 -->|"unmet"| r3
  linkStyle 0 stroke:#2e6b3a,stroke-width:2px
  linkStyle 1,2 stroke:#a33333,stroke-width:2px,stroke-dasharray:5 4
```

The output is the same on every machine, because nodes and arrows are sorted by name. A name reaches a label escaped, `%` included, so no name can break the diagram or act as a Mermaid directive.

## Getting started

`htmlspec` is one static binary. Install it the way your platform makes easy. Nix is one route among several, and nothing needs it.

### Linux, and Windows through WSL

Native Windows is not supported. Everything around htmlspec assumes a POSIX shell: the check scripts, the git hooks, the Nix shell, and the way preflight finds programs on `PATH`. On Windows, install WSL 2 by running `wsl --install` in an administrator terminal, open the Ubuntu it installs, and follow the Linux steps inside it. [Working in WSL](#working-in-wsl) says where projects belong and how to read documents from Windows.

**From a release.** This block installs the latest release into `~/.local/bin`, after checking the download against the release's `SHA256SUMS`. It names no version, so it stays right for every release, and running it again upgrades.

<!-- install: linux-release -->
```sh
(
  set -eu
  target="$(uname -m)-unknown-linux-musl"
  base=https://github.com/Firefly-Projects/htmlspec/releases/latest/download
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  cd "$tmp"
  curl -fsSLO --retry 3 "$base/SHA256SUMS"
  file="$(grep -- "-$target.tar.gz\$" SHA256SUMS | cut -d' ' -f3)"
  [ -n "$file" ] || { echo "no htmlspec build for $(uname -m)" >&2; exit 1; }
  curl -fsSLO --retry 3 "$base/$file"
  sha256sum --check --ignore-missing SHA256SUMS
  tar -xzf "$file"
  mkdir -p "$HOME/.local/bin" "$HOME/.local/share/doc/htmlspec"
  install -m 755 "${file%.tar.gz}/htmlspec" "$HOME/.local/bin/htmlspec"
  cp "${file%.tar.gz}"/*LICENSE* "$HOME/.local/share/doc/htmlspec/"
  "$HOME/.local/bin/htmlspec" --version
)
```

It needs `curl`, `ca-certificates`, and GNU coreutils, which Ubuntu, under WSL or not, already has. A minimal image may lack the first two, and `sudo apt-get update && sudo apt-get install -y curl ca-certificates` adds them. BusyBox's `sha256sum`, as on Alpine, has no `--ignore-missing`. To install one release rather than the latest, replace `releases/latest/download` with `releases/download/vX.Y.Z`.

Then run `htmlspec --version`. Ubuntu's `~/.profile` puts `~/.local/bin` on your `PATH`, but only in a new login shell, and only once the directory exists, which the block just made sure of. So:

- **Under WSL,** close the Ubuntu window and open it again. Each one starts a login shell.
- **In a desktop Ubuntu terminal,** which starts a shell that is not a login shell, log out of the desktop and back in, so `~/.profile` runs again.

If `htmlspec` is still not found after that, add the line yourself. Bash reads `~/.bash_profile` instead of `~/.profile` when the first one exists, so add it there if you have one:

```sh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.profile    # or ~/.bash_profile, if it exists
```

In the terminal you are in now, `export PATH="$HOME/.local/bin:$PATH"` works at once.

**With Homebrew on Linux.** Homebrew installs into `/home/linuxbrew/.linuxbrew`, which is not on your `PATH` until its environment is loaded, so the block loads it first:

<!-- install: linux-homebrew -->
```sh
eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
brew install Firefly-Projects/tap/htmlspec
htmlspec --version
```

To keep `brew` and `htmlspec` on your `PATH` in every new terminal, add Homebrew's line to your shell's start-up file:

```sh
echo 'eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"' >> ~/.bashrc
```

**With cargo.** Install Rust with [rustup](https://rustup.rs/), and a C linker, which Ubuntu does not ship until you ask for it:

<!-- install: cargo -->
```sh
sudo apt-get update && sudo apt-get install -y build-essential
rustup toolchain install stable
cargo install --locked htmlspec-cli
htmlspec --version
```

`--locked` builds with the dependency versions the release was tested with, rather than the newest ones each requirement allows. The binary is named `htmlspec`, and cargo puts it in `~/.cargo/bin`, which rustup adds to your `PATH`.

### macOS

<!-- install: macos-homebrew -->
```sh
brew install Firefly-Projects/tap/htmlspec
htmlspec --version
```

Or download the tarball for your Mac from the [releases page](https://github.com/Firefly-Projects/htmlspec/releases): `aarch64-apple-darwin` for Apple silicon, `x86_64-apple-darwin` for Intel. Check it with `shasum -a 256 --check --ignore-missing SHA256SUMS` beside the downloaded `SHA256SUMS`, then put `htmlspec` anywhere on your `PATH`.

### Nix, anywhere, if you use it

```sh
nix run github:Firefly-Projects/htmlspec -- --version   # run it once
nix profile install github:Firefly-Projects/htmlspec    # keep it
```

### Working in WSL

**Keep projects in the Linux file system,** under your home directory, not under `/mnt/c`, and use git from inside WSL. Under `/mnt/c` every file access crosses into Windows and is slow, and a project checked out by Windows' own git may have its line endings rewritten, the asset store's files included. Those files are named by their digests, so `htmlspec validate` then reports `asset/mismatch` and points at the store, not at where the project was checked out.

**To read the documents from a Windows browser,** the route to try is serving the project from WSL. In the directory that holds `htmlspec/`:

```sh
python3 -m http.server
# then open http://localhost:8000/htmlspec/specs/ in the Windows browser
```

This rests on WSL 2 forwarding `localhost` from Windows to the Linux side, which WSL documents and turns on by default. It has not been tried on a Windows machine for this README, so if the page does not load, the forwarding is the first thing to look at. Opening a document straight from the Linux file system in a Windows browser is not described here either, for the same reason: nobody has checked it.

**Run your coding agent inside WSL,** as its Linux build, so it runs where htmlspec and the project are. `htmlspec init --tools …` writes its instructions there, and editors reach the project through their WSL integration.

### After installing

`htmlspec --version`, or `-v`, prints the release you installed and the commit it was built from. To move to a newer release later, and to bring your projects up to it, see [Upgrading](#upgrading).

A binary ships with three license files. `LICENSE` is htmlspec's own MIT license. `THIRD-PARTY-LICENSES` holds the notices of the JavaScript libraries the binary embeds, Mermaid and Vega among them. `CRATE-LICENSES` holds those of the Rust crates it is built from. Each release tarball holds them beside the binary, the Linux block above copies them to `~/.local/share/doc/htmlspec/`, Homebrew installs them in the formula's prefix, and the Nix package installs them under `share/doc/htmlspec/`.

To build htmlspec from source, with Nix or without it, see [Building](#building).

Then set up a project and start a change:

```sh
htmlspec init --tools claude          # htmlspec/, spec.nix, and your agent's workflows
htmlspec new change add-dark-mode     # a change: proposal, design, and tasks
htmlspec validate                     # check every document in the project
```

## Upgrading

Upgrading is two steps on two schedules: upgrade the binary on each machine, then bring each project up to it. Within 1.x neither asks for much.

**An upgrade across several releases asks what each release in between asks.** The [changelog](CHANGELOG.md)'s "Upgrading" notes are cumulative: read every one from the release after yours up to the one you are moving to. `htmlspec --version` says which release you are on.

### The binary

Use the command of the channel you installed from, then check the release you now have with `htmlspec --version`, or `-v`.

**From a release, on Linux or WSL,** run [the install block](#linux-and-windows-through-wsl) again. It reads the latest release, so a second run installs the newer one over the first.

**From a release, on macOS,** download the newer tarball, check it against its `SHA256SUMS` as [the first time](#macos), and put its `htmlspec` where the old one was.

**With Homebrew,** on macOS or Linux. On Linux, if `brew` is not on your `PATH`, run `eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"` first:

<!-- upgrade: upgrade-homebrew -->
```sh
brew upgrade htmlspec
htmlspec --version
```

**With cargo,** install it again. Cargo replaces an installed version that is older than the newest:

<!-- upgrade: upgrade-cargo -->
```sh
cargo install --locked htmlspec-cli
htmlspec --version
```

**With Nix,** upgrade the profile entry, which `nix profile list` names, here `htmlspec`. In a project whose flake takes htmlspec as an input, here also named `htmlspec`, update that input and commit `flake.lock`:

<!-- upgrade: upgrade-nix -->
```sh
nix profile upgrade htmlspec          # a profile install
nix flake update htmlspec             # in a project that has htmlspec as a flake input
htmlspec --version
```

- **A tag does not move.** An entry installed from `github:Firefly-Projects/htmlspec/vX.Y.Z`, and an input whose URL names a tag, stay at that tag through `nix profile upgrade` and `nix flake update`. Name the new tag first: in `flake.nix` for an input, and for a profile entry with `nix profile remove htmlspec` and `nix profile install github:Firefly-Projects/htmlspec/vX.Y.Z`.
- **An older Nix.** `nix profile` names its entries from Nix 2.20, and before that `nix profile upgrade` takes the entry's index from `nix profile list`. Nix 2.34 refuses an index. `nix flake update htmlspec` updates one input from Nix 2.19, and before that `nix flake lock --update-input htmlspec` does.
- **`nix run` installs nothing to upgrade.** `nix run github:Firefly-Projects/htmlspec` runs the head of the public repository's default branch, which carries the latest export. Nix reuses its last look at that branch for `tarball-ttl` seconds, an hour by default, and `--refresh` makes it look again at once. `nix run github:Firefly-Projects/htmlspec/vX.Y.Z` runs exactly that release.

### A project

In the project's root, once the binary is newer:

<!-- upgrade: upgrade-project -->
```sh
htmlspec init --tools claude                  # the tools the project uses
htmlspec format --upgrade-assets --check      # optional: which documents are behind
htmlspec format --upgrade-assets              # optional: move them onto this release's assets
htmlspec validate
```

Then commit what changed.

- **`init` refreshes the agent instructions** to the new release's, including its marked region of `AGENTS.md` where it writes one, and leaves the specifications, the changes, the archive, the runbooks, and `spec.nix` as they are. Name the tools the project already uses, from `claude`, `codex`, `cursor`, `pi`, and `agents`, so `init` does not ask. `init` also writes into the asset store any file of the release's that the store lacks. Every 1.x release so far writes the same assets, so for a project made by any of them that changes nothing, and `--upgrade-assets` has nothing to move.
- **Each installed file records the release that wrote it,** so this shows which releases a project's instructions come from:

  ```sh
  grep -rhoE '(generatedBy:|Generated by) htmlspec [0-9]+\.[0-9]+\.[0-9]+' .claude .codex .cursor .pi AGENTS.md 2>/dev/null | sed -E 's/.*htmlspec /htmlspec /' | sort -u
  ```

- **`--upgrade-assets` is optional.** A document that references an older stylesheet or script still validates and still reads. The option moves living documents onto the files this release writes into new ones. It never rewrites an archived change or a run record, and it downloads nothing.
- **A project that pins htmlspec** moves its pin too: the `htmlspec` field of `spec.nix`, or a flake input. Pin the public repository, `github:Firefly-Projects/htmlspec/vX.Y.Z`, where an older pin names another address.

### What 1.x promises

Every 1.x release keeps three promises, each stated by an accepted requirement:

- **The command surface only grows.** toolchain-packaging's "The command surface is stable within a major version" states it. Every operation keeps its name in both spellings. Every flag keeps its name and meaning, every exit code its value, and every `--json` field its name and type. A minor release can add any of them, and only 2.0 removes or renames one. The suite fails when the program loses any of them. The wording of human-readable output can still change, so a script that needs an answer reads `--json`.
- **Documents stay valid within the contract's major version.** document-schema's "The contract's version is a semantic version" states it, and it is worded about the contract's version, the one a document declares in its `htmlspec-version` meta, not about the program's release: every 1.x release so far reads and writes contract `1.0.0`. Within a major version of the contract, a document one version accepts gains no error under any later version, unless the earlier version accepted it by a defect, which the release's changelog names. Warnings are not covered. Contract `1.0.0` is the `0.7.0` vocabulary, unchanged, so a document that declares `0.1.0` to `0.7.0` still validates. The suite checks that each conformance fixture gains no error under `1.0.0` that it lacked under the version it declares. It cannot check that a rule added later leaves a valid document valid, so review holds that line: a change to what a fixture expects is a change to the contract.
- **Upgrade every machine before adopting a newer runbook format.** A binary refuses a runbook declaration in a format newer than it reads, and names the formats it does read: 1.0 refuses `preflight/v2`, and 1.1 refuses `preflight/v3`. runbook-portability's "Machine requirements are explicit and mechanically checkable" states it.

The suite holds the surface to one file per released minor, [`1.0.json`](crates/htmlspec-cli/tests/surface/1.0.json), [`1.1.json`](crates/htmlspec-cli/tests/surface/1.1.json), and [`1.2.json`](crates/htmlspec-cli/tests/surface/1.2.json), each read from the program's own parser: the program keeps everything an earlier file of its major records, and the current minor's file records exactly what it does. Each payload has a published JSON Schema under [`crates/htmlspec-cli/schema/payload/`](crates/htmlspec-cli/schema/payload/). A published schema admits a field it does not declare, so output from a later 1.x release still validates against it. The suite reads the same schemas strictly, so the program writes only what its schema declares.

**Observed, not promised.** A run's record is read across 1.1 and 1.2 both ways. A record 1.2 writes states its runbook's release in its head, and 1.1 reads that record and ignores the line. A record 1.1 wrote states no release, and 1.2 reads it, and it meets a `preflight/v3` run check only where the check names no `atLeast`. This was observed when 1.2.0 was reviewed, and no requirement promises it.

### What each release lets a project adopt

Nothing here is required. The [changelog](CHANGELOG.md) says what each release changed and what upgrading to it asks.

- **1.1:** `htmlspec new runbook`, `format --upgrade-assets`, a `remedy` in a `preflight/v2` declaration, and `-v`.
- **1.2:** a runbook can require a passed run of another with a `preflight/v3` `run` check, and `htmlspec list --runbooks --graph` draws who requires whom.

1.1.1 and 1.2.1 give a project nothing new to adopt. 1.1.1 gave people an install path through WSL and every route without Nix, and 1.2.1 this section.

## Reading the documents

Open any document in Firefox or Chrome. Every page reads its stylesheet and scripts from the project's one asset store, so it renders offline with no build step.

Safari is stricter. When it opens a file from disk, it refuses a stylesheet in a parent folder, and the store always sits above the document. So every document the program writes also carries a small fallback stylesheet in its head. In Safari the page still reads as a centred serif column with monospace code. Everywhere else the real stylesheet wins, and the fallback changes nothing.

The fallback buys a readable page, not the full design. The store's scripts draw the diagrams and charts, and no stylesheet can replace them. If Safari refuses those scripts too, each figure shows its caption and its diagram source instead. To see the whole design in Safari, serve the project. From the directory that holds `htmlspec/`:

```sh
python3 -m http.server
# then open http://localhost:8000/htmlspec/specs/
```

Documents written before 0.31.0 have no fallback. `htmlspec format --fallback` adds it to them.

## Reference

### Commands

`htmlspec init` takes no path. Its subject is the repository you are standing in.

```sh
htmlspec init                       # set up this project, interactively
htmlspec init --tools claude,codex  # or name the tools and skip the prompt
htmlspec init --tools none          # or install no agent instructions at all
```

It creates `htmlspec/` with `specs/`, `changes/`, `changes/archive/`, and `runbooks/`. It establishes **one** shared asset store at `htmlspec/assets/`, writes a `spec.nix` declaration, and installs the workflow instructions for the tools you choose. It detects which tools your project uses and preselects them.

`init` is safe to run again. It refreshes the instructions and leaves your specifications, changes, archive, and runbooks alone. That is how a project picks up a newer version's instructions.

```sh
htmlspec new change add-dark-mode                # a change: proposal, design, and tasks
htmlspec new runbook deploy                      # a runbook: specification, tasks, VERSION, changelog
htmlspec new document specification s.html       # or one document of a given kind
htmlspec validate                                # every document in the project
htmlspec validate proposal.html --strict         # one document, warnings fatal
htmlspec format                                  # rewrite documents into canonical form
htmlspec format --check                          # report without writing; nonzero if any would change
htmlspec format --fallback                       # also give each document the fallback stylesheet
htmlspec format --upgrade-assets                 # also move documents onto the store's current assets
htmlspec assets                                  # check the project stores each asset once
htmlspec list --runbooks                         # the runbooks, with versions, titles, and what each requires
htmlspec list --runbooks --graph                 # which runbook requires which, as a Mermaid flowchart
htmlspec preflight release                       # can this runbook start here?
htmlspec show add-dark-mode --diff               # each requirement it modifies, beside the accepted one
htmlspec archive add-dark-mode                   # check a change before archiving it
htmlspec validate --json                         # structured diagnostics on stdout
htmlspec -v                                      # the version, as --version and -V print it
```

Every project-wide command finds its project by walking up from where you run it, so each one works from any directory inside the project. A new change or runbook goes where the project keeps it, and the command reports the resolved path.

`format --upgrade-assets` rewrites each document's references to the stylesheet and scripts this release writes into new documents. It never downloads anything: the current assets are compiled into the program, and any the store lacks are written into it from that copy. It adds nothing a document does not already load, and it leaves archived changes and run records as written, judging each document by the project it belongs to wherever you run it from, a link's target included. `--fallback` keeps history the same way. Where a stored file no longer matches its address, it leaves the reference as written and says how to restore the file. Without the flag, formatting leaves every reference to stored bytes as written, so a document keeps rendering with the bytes it was reviewed against until someone asks to move it.

Every operation also has a request-verb spelling, borrowed from htmx's request keywords. `get` never writes. `post` creates something that was not there. `put` creates or replaces a whole thing, and running it again leaves the same result. `patch` changes something that exists. `delete` removes something. Both spellings run the same operation, with the same arguments, options, output, and exit status.

The verb names the most an operation writes in any invocation, so `put format --check` writes nothing, as `format --check` does. The program's messages and the instructions it installs use the OpenSpec spelling, because adopters arrive from OpenSpec. The verbs send nothing anywhere.

| Operation | OpenSpec spelling | Request-verb spelling |
| --- | --- | --- |
| Validate documents | `validate` | `get validity` |
| List what the project holds | `list` | `get list` |
| Report a change's completion | `status` | `get status` |
| Report whether a change reached the accepted specifications | `sync` | `get sync` |
| Show modified requirements beside the accepted ones | `show` | `get show` |
| Check the asset topology | `assets` | `get assets` |
| Check a runbook's prerequisites | `preflight` | `get preflight` |
| Print a stage's workflow instructions | `instructions` | `get instructions` |
| List registered stores | `store list` | `get registrations` |
| Survey an OpenSpec project | `migrate survey` | `get survey` |
| Create a change | `new change` | `post change` |
| Create a document | `new document` | `post document` |
| Create a runbook | `new runbook` | `post runbook` |
| Register a store | `store add` | `post registration` |
| Start a run of a runbook | `run` | `post run` |
| Promote a staged migration | `migrate promote` | `post promotion` |
| Set up or refresh the project | `init` | `put project` |
| Format documents | `format` | `put format` |
| Archive a change | `archive` | `patch archive` |
| Remove a store's registration | `store remove` | `delete registration` |

`htmlspec --help` lists the verbs after the operations, and `htmlspec get --help` names the operation each of its objects runs.

Documents reference the project's one asset store relatively. Copy or rename the project anywhere and every page still opens with no network. Moving a *single change* out of its project breaks its references, deliberately: the alternative is a copy of the 6 MB runtime per change. `htmlspec validate` reports the broken reference instead of accepting a page that renders against nothing.

Exit codes are a contract:

| Code | Meaning |
| --- | --- |
| `0` | Success. |
| `1` | A document violates the contract, or a completed preflight found requirements unmet. Both are an unsatisfied contract, not a broken tool. |
| `2` | The operation could not run at all, for example an unreadable file or an invalid invocation. |
| `3` | You declined a prompt or interrupted the run. Nothing went wrong, and nothing was done. |

With `--json`, standard output carries nothing but the payload, every progress message goes to standard error, and no prompt appears. Each operation's payload has a JSON Schema in `crates/htmlspec-cli/schema/payload/`. A later 1.x release can add a field to a payload, so a script ignores a field it does not know.

`list --runbooks --json` carries each runbook's run checks. Per runbook, `declarations` is `readable`, `malformed`, or `unreadable`, and `prerequisitesMet` is true only when the declarations are readable and every prerequisite is met. Each entry of `prerequisites` gives the `requirement` declaring it, the `runbook` it names, any `atLeast`, its `state` (`met`, `unmet`, `unavailable`, `missing`, `cycle`, or `self`), up to three `satisfiedBy` records, the `satisfying` count, and the `openRuns` still under way. Only a `met` or `unmet` prerequisite names records, because only those are decided from records. `prerequisiteCycles` lists one cycle for each group of runbooks that lead back to one another, the shortest through its least runbook. `--graph` changes nothing under `--json`.

### Carrying a runbook to another machine

A runbook states what it needs of a machine. `htmlspec preflight <runbook>` reports what is missing before a run starts, not at the step that needed it. Starting a run performs the same checks, and nothing skips them. A runbook can also require that another runbook has run first, and the same checks hold that. A preflight you ran earlier is not permission: every start observes the machine, and reads the project's runs, afresh.

A machine requirement is an ordinary requirement unit in the runbook's `specification.html`, with its check declared in a note:

```html
<section class="requirement" id="req-start-outside-a-project" data-change="unchanged">
  <h3>The run starts outside a project <span class="hs-id">req-start-outside-a-project</span></h3>
  <p>A run MUST start in a directory that is not inside an existing project.</p>
  <p data-field="note">preflight/v1 {"kind":"state","check":"outside-project"}</p>
  <!-- a scenario, as any requirement carries -->
</section>
```

The note's projected text begins `preflight/v1 ` followed by one JSON object. `preflight/v1` has four forms, `preflight/v3` adds a fifth, `run`, and nothing else is read. A runbook cannot supply a command, an argument, or a regular expression. A carried document able to run arbitrary code during an inspection is not an inspection.

| `kind` | Fields besides `kind` | What it checks |
| --- | --- | --- |
| `tool` | `program`, `version` | A bare executable on the invocation's `PATH`, reporting exactly that `major.minor.patch` |
| `toolchain` | `manifest`, `shell`, `tools` | A local locked `flake.nix`, its named development shell, and the executables it supplies |
| `skill` | `scope`, `path` | A readable, nonempty `SKILL.md` in the `home` or `workdir` scope, at that relative path |
| `state` | `check` | `outside-project`, or `clean-git-worktree` |
| `run` (`preflight/v3` only) | `runbook`, optional `atLeast` | A finished, passed run of that runbook, recorded in this project |

Paths are relative, and they cannot be absolute or climb out with `..`. Executable names are bare: no separator, no leading hyphen. A requirement holds one declaration, and every declared check must pass. A note that begins `preflight/` and names a version this program does not read is an error, not prose. Skipping it would report a runbook ready when it asked for checks nothing performed.

A runbook can also say where to get what a check needs. `preflight/v2` reads every form `preflight/v1` reads, plus one optional field, `remedy`:

```html
<section class="requirement" id="req-the-login-skill-is-installed" data-change="unchanged">
  <h3>The login skill is installed <span class="hs-id">req-the-login-skill-is-installed</span></h3>
  <p>A run MUST have the login skill installed.</p>
  <p data-field="note">preflight/v2 {"kind":"skill","scope":"home","path":".claude/skills/login/SKILL.md","remedy":"copy skills/login from the team's skills repository into ~/.claude/skills"}</p>
  <!-- a scenario, as any requirement carries -->
</section>
```

A remedy is one line of 1 to 200 characters, and every character in it is shown as what it is. It holds no control character, no whitespace but the plain space (so no tab, no non-breaking space, and no line or paragraph separator), and no invisible character: none of Unicode's format characters or default-ignorable characters, the bidirectional ones and the variation selectors included, and not the braille pattern blank, which draws as a space. The zero-width joiner and non-joiner are the exceptions, because scripts and emoji need them, and so is one emoji presentation selector directly after the character it is for. Spaces around it are trimmed, and a remedy with no visible character is refused as empty. When a check does not pass, the report shows the program's own remedy and then `the runbook says:` with the runbook's text, so you can tell the two apart. With `--json`, every check whose declaration carries a remedy has `runbookRemedy`, passed or not. A remedy is text. Preflight never runs it, never opens an address in it, and never counts it as satisfying anything: the skill still has to be installed. htmlspec 1.0 refuses `preflight/v2` as a format it does not read, so a runbook that declares a remedy needs 1.1 or later. A runbook with no remedy keeps writing `preflight/v1`, which every version reads.

A runbook can also require that another runbook has run first. `preflight/v3` reads every form `preflight/v2` reads, with its `remedy`, plus one more kind, `run`:

```html
<section class="requirement" id="req-a-backup-ran-first" data-change="unchanged">
  <h3>A backup ran first <span class="hs-id">req-a-backup-ran-first</span></h3>
  <p>A run MUST start only after a finished, passed run of the backup runbook.</p>
  <p data-field="note">preflight/v3 {"kind":"run","runbook":"backup","atLeast":"1.4.0"}</p>
  <!-- a scenario, as any requirement carries -->
</section>
```

A record of `backup`, one of the `.html` files under its `runs/` folder, satisfies the check when it parses without error recovery, is a tasks document, has at least one task done and every task done or dropped, and records every requirement it carries as `passed`. Any such record counts. A record that is a symbolic link, or resolves outside the runbook's `runs/` folder, does not, because the run it records may have happened elsewhere. A record states no time, so the program never prefers one run as newer than another. A run still under way is named in the report even when a finished one meets the check, so a backup in progress beside an older finished one is in view. When the report is not ready for another reason, it lists such met checks, and their runs under way, after the ones that are not satisfied.

`atLeast` is optional. With it, the record's release must be at or above that one within the same major: `"atLeast":"1.4.0"` accepts 1.4.0 and 1.9.2, and refuses 1.3.0 and 2.0.0. `htmlspec run --as` writes each new record's runbook release in its head, `<meta content="1.4.0" name="htmlspec-runbook-release">`, and that is what `atLeast` reads, from the record's `<head>` and nowhere else. A record written by an earlier version states no release, so it satisfies only a check without `atLeast`. Under `atLeast`, a record that states no release, states one that is not three dot-separated integers, or states more than one does not count, and the report says which.

The check reads only the runbooks and records of this project. The name is matched byte for byte against the project's runbook directories, whatever the filesystem, so `Backup` names no runbook beside `backup`. A run recorded in another checkout or on another machine does not count. A runbook named nowhere in the project leaves the check unmet, and the remedy says to add a runbook of that name or correct the declaration to a name `htmlspec list --runbooks` prints. A runbook that requires itself, or a cycle that leads back to it, is an invalid contract: `preflight` and `run --as` exit 2 and name the cycle in order. A specification reached on the way that cannot be read leaves the check unavailable, and the report gives the remedy for what could not be read.

An unmet run check refuses `htmlspec run --as` with exit 1 before anything is written, exactly as a missing tool does, and there is no override. The report says why:

```
  !!  req-a-backup-ran-first [unmet] — a finished, passed run of `backup` at release 1.4.0 or later within major 1, recorded in this project
      expected: a finished, passed run of `backup` at release 1.4.0 or later within major 1, recorded in this project
      observed: `backup` has no runs on record
      remedy: run `backup` to completion, with every requirement it carries recorded as passed, then start this run again: `htmlspec run backup --as NAME`. A run that recorded a failure is a correct record of a failed run, and the way forward is a new run, never an edited outcome
```

When records exist and none satisfies the check, the observation names each one and the first reason it does not count, checked in this order: it cannot be read, it needs error recovery to parse, it is not a tasks document, it has no task done, it is still open with so many steps done, it recorded an outcome other than `passed`, it records no outcome for a requirement, or, under `atLeast`, its release is missing, malformed, stated twice, or misses `atLeast`. A run that recorded a failure is a correct record of a failed run. The way forward is a new run, never an edited outcome. Records are documents anyone with the repository can edit, so the check keeps runs in order and is not a security boundary: a hand-edited record satisfies it as readily as a real one.

When the check is met, the new run's record names the runs that satisfied it, up to three with their releases and a count of the rest, so a reader can later see which backup a migration relied on. With `--json`, each run check carries a `run` object: the runbook, its `state` (`met`, `unmet`, `missing`, or `unavailable`), and the records that do and do not satisfy it.

A carried runbook checks its prerequisites in the project it was carried to. Under `--store`, the check reads that project's runbooks and records, and machine checks still read the working directory. A prerequisite runbook that was not carried, or has no passed run there, leaves the check unmet. Do not carry old records across to meet it: the check cannot tell a copied record from one made here, and a run that happened elsewhere is not a run in this place. htmlspec 1.1 and earlier refuse `preflight/v3` as a format they do not read, so a runbook that declares a run check needs 1.2 or later. A runbook with no run check keeps writing `preflight/v1` or `preflight/v2`.

What the checks are, and what they are not:

- **A tool check runs the resolved executable with `--version` and reads one standalone three-component number out of its output.** No shell, closed stdin, ten seconds, 64 KiB of output. Output holding two candidate versions is ambiguous, and preflight reports it instead of guessing. A tool with another version interface needs an adapter. The runbook cannot supply the argument.
- **A toolchain check asks the package manager and provisions nothing.** For Nix it evaluates the selected development shell's inputs offline, with lock updates and import-from-derivation disabled. Then it compares where each named tool resolves against the store paths those inputs already occupy on this machine. It never runs `nix develop`, `build`, `shell`, or a fetch. An environment whose inputs are not realized is *unavailable*, not failed, and nothing downloads to make it available. No version is copied out of the manifest into a second dependency catalog, which is why the form names the manifest.
- **A skill check establishes that installed content is readable, not that it is good.** A symlink escaping its scope is refused. A copy bundled beside the runbook does not satisfy a `home`-scope declaration, because the question is whether the skill is installed.
- **A run check reads records, not the machine.** It opens the named runbook's records in this project and judges each one as a document. It also reads the specifications its run checks reach, which is where a cycle is found and where an unreadable one leaves the check unavailable. It never runs that runbook, never asks whether its steps really happened, and never looks outside the project.
- **A state check is about the invocation's working directory, not the store's.** `--store` selects where documents are read from. `clean-git-worktree` uses a read-only status with optional locks disabled, and counts modified, staged, conflicted, and untracked files alike. A directory that is not a repository is unavailable, not clean.

Preflight reports and does not repair. Choosing how to install something on an unknown machine is judgment. So the report names a remedy for each refusal and leaves the choice to you.

A runbook's run records live in the project it came from. So a carried runbook needs a version of its own: `VERSION` beside its documents, and a `CHANGELOG.md` heading for it. Archival compares each runbook's canonical documents against the last release it published, and refuses one version for two different contents. A version identifies one released procedure. A run's record is not part of that comparison, because a runbook exists to be run, and neither is changelog prose.

A copy needs very little:

```text
portable/
  htmlspec/
    assets/                 one shared asset store, copied
    runbooks/release/
      specification.html
      tasks.html
      VERSION
      CHANGELOG.md
      runs/                 created by the first run started here
```

Register it with `htmlspec store add portable <path>`. Then check and run it from the directory the work is in: `htmlspec --store portable preflight release`. The source project's archive, declaration, accepted specifications, and old run records stay behind, and a new local run does not need them. Transport is not a command. Copying is yours, and `git`, an archive, or `cp` are all equally correct.

Older runbooks have no `VERSION`. Starting one is refused with the migration, instead of recorded anonymously. Reviewing the runs an older runbook already has still works: `htmlspec run <runbook> --check` reports conformance and never inspects the machine.

### The project declaration

`init` writes `spec.nix` at the project root:

```nix
{
  name = "my-project";
  version = "0.0.0";
  htmlspec = null;   # replace with a pinned flake reference
}
```

`htmlspec archive` reads `version` and refuses unless it is **ordered after** the last archived version, so the accepted contract carries a history. Advancing it is your work, or your agent's. The program only compares. Versions start at `0.0.0` and are ordered componentwise, so `0.10.0` follows `0.9.0`.

### Layout of a project

```text
spec.nix                               Project name, version, and pinned toolchain
htmlspec/
├── assets/                            Pinned CSS and visualization runtime, one copy
├── specs/
│   └── theme-selection.html           An accepted specification
├── changes/
│   ├── add-dark-mode/
│   │   ├── proposal.html              The value, need, and scope
│   │   ├── design.html                Architecture and technical reasoning
│   │   ├── tasks.html                 The work items and their states
│   │   └── specification.html         The settled contract, produced at archival
│   └── archive/
│       └── 2026-08-04-add-dark-mode/  Every document intact, and a VERSION file
└── runbooks/
    └── release/
        ├── specification.html         What the procedure is for, and its hazards
        ├── tasks.html                 The steps, in order
        ├── VERSION
        ├── CHANGELOG.md
        └── runs/                      One record per run
```

The asset store is the project's, not each change's. One copy of the runtime serves the whole project, however many changes it holds.

### Validation

`htmlspec validate` checks structure and safety before a change is accepted:

- every requirement and scenario has a stable, unique identifier
- every internal link and fragment resolves
- task and verification references point to real requirements
- the HTML parses without error recovery, so its reading is deterministic
- example code is inert, and only the pinned runtime executes
- every embedded chart and diagram is checked against its own format
- nothing reaches the network: remote scripts, fonts, and tracking are rejected
- the core content reads with scripting disabled

The goal is not merely attractive output. It is an artifact that agents modify reliably and people actually read.

### HTML is the artifact, not the application

HTMLSpec does not turn a proposal into a miniature web application. The baseline is semantic HTML and CSS. JavaScript enhances diagrams, charts, and navigation, and a specification stays readable when JavaScript fails or is turned off.

The runtime is local, offline, and pinned to known versions. The project shares one copy instead of copying it into each document. Only approved libraries run, and an agent parsing a document needs none of them.

### Design principles

- **Browser-first review.** The browser is the primary human interface. Source view and Git diffs still matter, but they are not the only way to understand a change.
- **Agent-authored, human-directed.** People own intent, judgment, and approval. Agents own the repetitive work of keeping the artifacts consistent.
- **Semantic before decorative.** Requirements, scenarios, decisions, evidence, risks, and tasks use stable HTML structures. Styling helps comprehension and never hides meaning.
- **Visuals when they earn their space.** Diagrams explain topology and sequence. Charts explain evidence. Tables hold exact comparisons. Ordinary prose stays ordinary prose.
- **Fluid, not ceremonial.** Exploration, proposal, design, implementation, and verification can loop. The format supports the work instead of forcing rigid phase gates.
- **Local, portable, and durable.** Specifications work offline, live beside the code, and travel through Git. No hosted service is required.
- **Deterministic and reviewable.** Stable identifiers, predictable ordering, canonical formatting, and validation keep rich output from producing noisy diffs.

### Relationship to OpenSpec

[OpenSpec](https://github.com/Fission-AI/OpenSpec/) provides the starting model: lightweight spec-driven development, a fluid change lifecycle, plain files in the repository, and support for many coding agents.

HTMLSpec explores one deliberate disagreement. OpenSpec treats plain Markdown as a feature. HTMLSpec treats the browser as the review surface and the agent as the document editor. `htmlspec migrate survey` and the `/htmlspec:migrate` workflow bring an existing OpenSpec project across.

The project also builds on the argument in [The unreasonable effectiveness of HTML](https://claude.com/blog/using-claude-code-the-unreasonable-effectiveness-of-html). Once agents create and revise our technical artifacts, richer and more readable output keeps people engaged in the work.

HTMLSpec is an independent experiment and is not affiliated with Fission AI or Anthropic.

### Building

Two routes build htmlspec and run its checks, and neither is second-class. Continuous integration runs both.

**With [Nix](https://nixos.org/):**

```sh
nix build              # the htmlspec binary, at result/bin/htmlspec
nix run . -- --help    # run it without installing
nix develop            # a shell with the pinned toolchain
nix flake check        # the full check suite, in the build sandbox
```

**Without Nix,** a machine needs:

- [rustup](https://rustup.rs/), then `rustup toolchain install` in the repository, which reads `rust-toolchain.toml` and installs the pinned Rust with `rustfmt`, `clippy`, and `rust-src`;
- a C linker for cargo to link with: `build-essential` on Ubuntu, the Xcode command-line tools on macOS;
- Python 3, standard library only. Python 3.9.6 and 3.13.14 were checked here, and continuous integration runs Ubuntu's own;
- bash and git.

```sh
rustup toolchain install       # in the repository: the toolchain rust-toolchain.toml pins
cargo build --release          # the htmlspec binary, at target/release/htmlspec
cargo run --bin htmlspec -- --help
scripts/check                  # every stage; scripts/check --list prints them
```

With those, `scripts/check` runs every stage continuous integration runs. If you set `CARGO_TARGET_DIR`, the prompt stage finds the binary there, as cargo builds it.

A few things only the Nix sandbox and continuous integration check, because they need the flake: that every file a stage reads is tracked and survives the flake's source filter, the provenance of the vendored browser libraries, which are fetched again from their pinned addresses, and a reproducible `nix build`. Publishing a release is a maintainer's job, and its runbook requires the flake's development shell.

The packaged binary needs no source checkout and no Nix at runtime.

### Checks

One command runs everything: formatting, lints, every test, the conformance fixtures, the asset lockfile hashes, the enhancement-script byte budget, and this project's own documents, checked through the built binary.

```sh
scripts/check
```

`nix flake check` and continuous integration both run that same script instead of keeping their own step lists, so all three check the same thing. A second job runs it without Nix, the way [Building](#building) describes, so that route stays true. To change what is checked, change the script.

### Landing a change

Work reaches `main` through a pull request, never by a push to it. Install the hook that enforces that locally, once per clone:

```sh
scripts/install-hooks
```

Git does not install hooks from a clone on its own, and it is right not to. A repository that ran code out of its own working tree on clone would be a remote execution vector. The installer points `core.hooksPath` at `.githooks/`, which holds one hook: `pre-push`, refusing a push to `main`. In the private working repository the project is published from, it also refuses every push to the public repository, because a push from there would carry that repository's history. The public repository receives only exports.

Then the loop is ordinary:

```sh
git switch -c a-branch-naming-the-change
# work, commit
scripts/check
scripts/check-pull-request --title "The title you will give the pull request"
git push -u origin HEAD
gh pr create
```

Three workflows check what arrives, and a branch ruleset guards `main`:

| Workflow | Asks | Runs |
| --- | --- | --- |
| `check.yml` | is this revision healthy? | `scripts/check`, on every pull request and every push to `main` |
| `check-without-nix.yml` | does the suite still run without Nix? | `scripts/check` with rustup and Ubuntu's Python, weekly and on pull requests that touch what it builds from |
| `pr-guard.yml` | is this a well-formed proposal? | `scripts/check-pull-request`, on the pull request's title and diff |

`pr-guard` enforces two things. The title becomes the squash commit's subject, so it must read the way this project's history reads: imperative prose under 72 characters, no Conventional Commits prefix, no trailing period. And the diff must touch `CHANGELOG.md`, unless the pull request carries the `skip-changelog` label to state that there is nothing worth recording. Both rules live in `scripts/check-pull-request`, which you can run before you push. The workflow only supplies it with the event's facts.

The ruleset on `main` requires a pull request and has no bypass, so GitHub refuses a direct push, a force-push, or a deletion of `main` outright. The hook stops the same mistake earlier, on your machine, before anything leaves it.

For a repository whose `main` has no ruleset, the hook's check can be skipped on purpose:

```sh
HTMLSPEC_ALLOW_MAIN_PUSH=1 git push origin main
```

The variable exists so the escape is visible. `--no-verify` skips every hook silently and leaves no record of intent. This names the decision, and the ruleset still decides whether the push is accepted.

### Source layout

| Path | Contents |
| --- | --- |
| `crates/htmlspec-schema` | the JSON Schema contract, the projection specification, and the types generated from them |
| `crates/htmlspec-parse` | HTML to document model, including fatal parse-error handling |
| `crates/htmlspec-format` | the canonical serializer |
| `crates/htmlspec-validate` | the rule engine and diagnostics |
| `crates/htmlspec-assets` | the asset store in its `assets/`: the templates, the lockfile, the stylesheets, the enhancement script, and the vendored libraries a document renders through |
| `crates/htmlspec-cli` | the `htmlspec` binary |
| `conformance` | the fixture harness, driving the crates' public APIs only |
| `fixtures` | conformance, projection, parse-error, and hand-authored example documents |
| `packaging` | the Homebrew formula's template, which `scripts/render-formula` fills for each release |
| `htmlspec` | this repository's own project: its specifications, changes, archive, and release runbook |

The public contract is three pieces, not one: `crates/htmlspec-schema/schema/document.schema.json` for the shape of the model, `crates/htmlspec-schema/doc/projection.md` for the HTML-to-model mapping, and the fixtures that pin it. An implementation written from those reproduces the model with no access to this source.

## Project status

HTMLSpec 1.0.0 is the first release that makes promises, and every 1.x release since keeps them. This repository runs on it: its own specifications, changes, and runbooks are the HTML documents in [`htmlspec/`](htmlspec/). It migrated itself out of OpenSpec at 0.29.0.

Release 1.0.0 implements:

- **the document format:** the contract, the parser, the validator, the canonical formatter, the visual system, and the offline runtime
- **the command-line toolchain**, with an OpenSpec spelling and a request-verb spelling for every operation
- **the change lifecycle**, and the agent workflows that drive it
- **runbooks:** versions, preflight, runs and their records, and portability
- **migration from an OpenSpec project**

Release 1.1.0 adds:

- **the notices of the Rust crates** the binary is built from, in every channel that ships it
- **`htmlspec new runbook`** and **`htmlspec format --upgrade-assets`**, and a runbook listing from `run` and `list --runbooks`
- **install hints:** `preflight/v2` lets a runbook say where to get what a check needs
- **`htmlspec -v`**, and an export that pushes as the account `gh` is signed in as

Release 1.1.1 adds **an install path through WSL, and every route without Nix**: the Linux and WSL block, Homebrew on Linux, and `cargo install` all stand on their own, a release job runs the README's install commands against each published release, and native Windows is stated as unsupported.

Release 1.2.0 adds:

- **runbook prerequisites:** a `preflight/v3` run check makes a runbook require a finished, passed run of another runbook before its own run starts, with no override
- **the order, shown:** `list --runbooks` and a bare `run` show what each runbook requires and whether it is met now, and `list --runbooks --graph` draws it as a Mermaid flowchart

Release 1.2.1 adds **a guide to upgrading**: how to upgrade the binary on each channel and bring a project up to it, under [Upgrading](#upgrading).

**What 1.x promises** is set out under [Upgrading](#what-1x-promises): the command surface only grows, documents stay valid within the contract's major version, and the suite holds both.

One piece of the original proposal is not built yet: **portable single-file exports**. An export would inline the runtime into one document that travels alone, for email, archival, or an external review.

HTMLSpec succeeds when a developer asks an agent for a substantial change, opens one page, and understands the plan. No wall of Markdown stands in the way. It succeeds when better presentation makes people read specifications more closely and catch bad decisions earlier. Then people stay in control while agents do more of the implementation.

If the premise is right, the interface for spec-driven development is not another text format.

It is a browser tab.
