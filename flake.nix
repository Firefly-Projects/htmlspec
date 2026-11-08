{
  description = "HTMLSpec toolchain: validate, format, and initialize HTMLSpec documents";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , rust-overlay
    , crane
    , flake-utils
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # The toolchain is pinned by rust-toolchain.toml so that `nix develop`,
        # `nix build`, `nix flake check`, and a bare `cargo` invocation inside the
        # shell all use the same compiler.
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # The package's source, which the binary and the dependency derivation are built from. The
        # suite's derivation below no longer reads through this filter: it takes the whole tracked
        # tree. The filter stays as narrow as it was, so a change to a workflow or a document does not
        # rebuild the binary, and the reasons below say why each kind of file was kept when the
        # suite ran in this same source.
        #
        # Keep JSON schema assets, HTML fixtures, and vendored browser libraries in
        # the build source, because build scripts and tests read them from disk.
        # `scripts/` is kept because the check suite *is* `scripts/check`, and a
        # source tree without it would have no suite to run.
        #
        # `VERSION` is kept for the same reason one step further out: the release check
        # reads the version each archived change recorded, and those files carry no
        # extension, so the pattern above does not reach them. A file the suite reads
        # must be in the source or the suite passes locally and fails here.
        #
        # `yaml`, `txt`, and `py` are kept because the migration survey enumerates a source
        # project's files, and the OpenSpec project this repository was migrated from, kept in
        # `fixtures/migration-source/openspec/`, is what its tests survey. Each was dropped,
        # and each drop is a different failure:
        #
        #   yaml  `config.yaml` and the per-change `.openspec.yaml` files. Two of the eight
        #         entity kinds vanish, so `every_file_in_this_repository_classifies` fails
        #         here while passing locally. This one reached continuous integration.
        #   txt   `mutation-testing.txt`, one evidence file.
        #   py    `prove-listing.py` and `prove-sequence.py`, two more.
        #
        # The last two are the dangerous shape rather than the loud one. Evidence also arrives
        # as `.md`, so the `evidence` kind stays present and no kind-level check notices three
        # files missing — the survey simply reports a smaller project with every test green.
        # `every_extension_the_project_holds_reaches_the_survey` exists for that case and is
        # what found them.
        #
        # A dot-prefixed name needs no special case: the pattern matches on the extension, and
        # `.openspec.yaml` ends in one.
        #
        # `LICENSE`, `THIRD-PARTY-LICENSES`, and `CRATE-LICENSES` carry no extension either. The
        # release check compares each published crate's `LICENSE` with the root's, so without them
        # every crate reads as unlicensed here while passing locally, and the package installs all
        # three beside the binary.
        #
        # The notices stage regenerates `CRATE-LICENSES` from two more inputs no pattern above
        # reaches: `license-clarifications.tsv`, the reviewed table, and the standard license texts
        # vendored without an extension under `crates/htmlspec-cli/licenses/`. Without them the
        # stage reports each reviewed crate as having no file and no row, here and nowhere else.
        # It also holds the Homebrew formula's template to what `CRATE-LICENSES` credits, so the
        # template, `packaging/homebrew/htmlspec.rb.in`, is kept as well.
        srcFilter = path: type:
          let
            keep =
              builtins.match ".*\\.(json|html|css|js|md|yaml|txt|py|schema\\.json)$" path != null;
            isScript = builtins.match ".*/scripts(/.*)?" path != null;
            isVersion = builtins.match ".*/VERSION$" path != null;
            isLicense = builtins.match ".*/(LICENSE|THIRD-PARTY-LICENSES|CRATE-LICENSES)$" path != null;
            isNoticeInput =
              builtins.match ".*/crates/htmlspec-cli/(license-clarifications\\.tsv|licenses(/.*)?)" path != null
              || builtins.match ".*/packaging(/homebrew(/htmlspec\\.rb\\.in)?)?" path != null;
          in
          keep || isScript || isVersion || isLicense || isNoticeInput
          || craneLib.filterCargoSources path type;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = srcFilter;
          name = "source";
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = [ ];
          buildInputs = [ ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # The revision `htmlspec --version` reports. The build source is a store path with no
        # `.git`, so the build script cannot ask Git, and the flake states the revision it was
        # evaluated at instead: twelve characters, as the build script shortens Git's, and
        # `-dirty` for a tree with uncommitted changes. A flake with neither, such as one read
        # from a plain directory, sets nothing, and the binary reports `unknown`.
        #
        # Set on the package alone, not in `commonArgs`: the dependencies' derivation would
        # otherwise change with every commit and never be reused.
        revision =
          if self ? rev then builtins.substring 0 12 self.rev
          else if self ? dirtyRev then "${builtins.substring 0 12 self.dirtyRev}-dirty"
          else null;

        # The question the suite's derivation and the development shell's check each ask before
        # anything else: whether an interpreter resolves where none may. A name it finds fails the
        # build that asks, printing where it resolved. Stated once, so this file names the
        # interpreter on one line, which the ban's allow-list, in the tools crate, names by its
        # exact text; nothing else in this file may name it. An `if`, rather than each lookup
        # negated with `!` and joined by `&&`, because a build phase runs under `set -e`, which
        # ignores a failure that `!` inverts or that a `&&` list does not end on, so that form would
        # print what it found and carry on.
        noInterpreterIn = place: ''
          if command -v python3 || command -v python; then
            echo "${place} supplies an interpreter it must not; take it out of its inputs" >&2
            exit 1
          fi
        '';

        # The development shell, and its check below, which builds with exactly the shell's inputs.
        # No interpreter, as the suite's derivation has none: the repository's tools are built from
        # its Cargo workspaces, and nothing it holds runs one.
        #
        # `curl` because the crate publisher asks the index with it, and its tests run it, as the
        # suite's derivation supplies it.
        developmentShell = craneLib.devShell {
          packages = [
            rustToolchain
            pkgs.cargo-nextest
            pkgs.jq
            pkgs.curl
          ];
        };

        # The binary's notices go where a Nix user's documentation goes, as every other channel
        # ships them beside the binary: htmlspec's own license, the JavaScript libraries' it embeds,
        # and the Rust crates' it is built from.
        htmlspec = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          doCheck = false;
          meta.mainProgram = "htmlspec";
          postInstall = ''
            install -Dm644 LICENSE "$out/share/doc/htmlspec/LICENSE"
            install -Dm644 crates/htmlspec-assets/THIRD-PARTY-LICENSES "$out/share/doc/htmlspec/THIRD-PARTY-LICENSES"
            install -Dm644 crates/htmlspec-cli/CRATE-LICENSES "$out/share/doc/htmlspec/CRATE-LICENSES"
          '';
        } // pkgs.lib.optionalAttrs (revision != null) {
          HTMLSPEC_REVISION = revision;
        });

        # One command runs the full suite, and it is `scripts/check` — the same script
        # a contributor runs and the same one CI runs. This derivation invokes it
        # rather than restating its steps, because a restated list is a list that
        # drifts, and the day it drifts is the day CI passes something a
        # contributor's machine rejects.
        #
        # The script's own comments say what each step covers. Nothing about the
        # suite's contents is decided here.
        #
        # The suite runs in the whole tracked tree, the flake's own source, and not in the package's
        # filtered one, so every file a stage reads in a checkout is here too. Through the filter,
        # each kind of file it did not name passed locally and failed here, one extension at a time.
        # `HTMLSPEC_SOURCE_TREE` names the same tree, as Nix copied it, to the stage that holds the
        # ban, which reads every tracked file: the build directory the stages run in is a copy of it
        # that fills with build output and patched scripts, and is no record of what is tracked.
        checkSuite = craneLib.mkCargoDerivation (commonArgs // {
          inherit cargoArtifacts;
          src = ./.;
          HTMLSPEC_SOURCE_TREE = "${./.}";
          pnameSuffix = "-check-suite";
          # Every script under `scripts/`, not a list of them. `scripts/check` runs the others as
          # programs, so the sandbox resolves each of their shebangs too, and a hand-kept list
          # missed one and took CI red for five merges with `/usr/bin/env: bad interpreter`.
          #
          # The sandbox supplies no interpreter for a script that slipped past the suite's ban to
          # run in, and the question above checks it before the suite runs.
          buildPhaseCargoCommand = ''
            ${noInterpreterIn "the suite's sandbox"}
            patchShebangs scripts
            ./scripts/check
          '';
          # `gitMinimal` because the prompt's test runs `git init` per case: the prompt's
          # project resolution stops at a repository boundary, so a scratch directory
          # that is not a repository does not exercise the behavior under test. The
          # test fails outright when `git` is absent rather than testing something
          # else. The ban's own tests also make scratch repositories, with
          # `git init` and `git add`, and read them with `git ls-files`. Minimal rather
          # than full `git`, which pulls a Perl closure this suite has no use for.
          #
          # `curl` because the crate publisher asks the index with it, and its tests run the real
          # one against a listener on the loopback interface, which the build sandbox provides, and
          # reach nothing beyond it.
          nativeBuildInputs = [ rustToolchain pkgs.bash pkgs.gitMinimal pkgs.curl ];
        });

        # The provenance half of the asset lockfile check.
        #
        # `cargo test -p htmlspec-assets` verifies the committed bytes against the
        # recorded hashes, which a contributor can run offline. That proves the
        # bytes have not changed; it cannot prove they came from where the lockfile
        # says. This derivation closes that gap: Nix fetches each archive from its
        # pinned URL — failing if the archive hash does not match — then extracts
        # the recorded member and compares it against the recorded file hash.
        #
        # So the committed third-party build output is auditable rather than
        # trusted, which is what makes committing it defensible.
        assetLock = builtins.fromJSON (builtins.readFile ./crates/htmlspec-assets/assets/vendor.lock.json);

        # An asset's content address, by the rule documented in the lockfile and
        # implemented in `crates/htmlspec-assets/src/address.rs`. Stated here a third
        # time because Nix cannot call into the crate — which is why a Rust test
        # asserts all three agree rather than trusting that they do.
        addressLength = 16;
        storedPath = file:
          let
            components = pkgs.lib.splitString "/" file.path;
            name = pkgs.lib.last components;
          in
          "sha256-${builtins.substring 0 addressLength file.sha256}/${name}";

        vendoredArchive = library: pkgs.fetchurl {
          url = library.archive.url;
          sha256 = library.archive.sha256;
        };

        assetProvenance = pkgs.runCommand "htmlspec-asset-provenance"
          {
            nativeBuildInputs = [ pkgs.gnutar pkgs.gzip pkgs.coreutils ];
          }
          (''
            set -euo pipefail
            echo "verifying ${toString (builtins.length assetLock.libraries)} vendored libraries"
          '' + pkgs.lib.concatMapStrings
            (library: pkgs.lib.concatMapStrings
              (file: ''
                mkdir -p extract
                tar xzf ${vendoredArchive library} -C extract "${file.archive_member}"
                recorded="${file.sha256}"
                found="$(sha256sum "extract/${file.archive_member}" | cut -d' ' -f1)"
                if [ "$found" != "$recorded" ]; then
                  echo "FAIL ${file.path}: ${library.name} ${library.version} publishes"
                  echo "  ${file.archive_member} with hash $found,"
                  echo "  and the lockfile records $recorded."
                  echo "  The committed bytes do not come from the pinned source."
                  exit 1
                fi
                # The committed copy lives at its content address, derived from the
                # same lockfile digest by the same rule the Rust build script uses.
                # `every_generated_address_matches_the_derivation` asserts the two
                # agree, so this cannot drift into checking a path nothing ships.
                committed="$(sha256sum ${./crates/htmlspec-assets/assets}/${storedPath file} | cut -d' ' -f1)"
                if [ "$committed" != "$recorded" ]; then
                  echo "FAIL ${file.path}: the committed copy at ${storedPath file} has"
                  echo "  hash $committed, and the pinned source publishes $recorded."
                  exit 1
                fi
                echo "ok ${file.path} (${library.name} ${library.version})"
                rm -rf extract
              '')
              library.files)
            assetLock.libraries
          + ''
            touch $out
          '');
      in
      {
        packages = {
          default = htmlspec;
          htmlspec = htmlspec;
        };

        apps.default = flake-utils.lib.mkApp { drv = htmlspec; };

        checks = {
          default = checkSuite;
          suite = checkSuite;
          # Requires network access to re-fetch the pinned archives, so it is a
          # separate check rather than part of the offline suite.
          assetProvenance = assetProvenance;
          # The development shell's derivation itself, its build phase replaced by the question, so
          # it builds with exactly the shell's packages on its search path and nothing of the
          # machine's: a package added to the shell that brings an interpreter, under any package's
          # name, fails here. `nix develop` puts the caller's own search path after the shell's,
          # which this does not read, because a contributor's machine is theirs.
          shellHasNoInterpreter = developmentShell.overrideAttrs (_: {
            buildPhase = ''
              ${noInterpreterIn "the development shell"}
              touch "$out"
            '';
          });
        };

        devShells.default = developmentShell;

        formatter = pkgs.nixpkgs-fmt;
      });
}
