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
        # `LICENSE` and `THIRD-PARTY-LICENSES` carry no extension either. The release check
        # compares each published crate's `LICENSE` with the root's, so without them every
        # crate reads as unlicensed here while passing locally.
        srcFilter = path: type:
          let
            keep =
              builtins.match ".*\\.(json|html|css|js|md|yaml|txt|py|schema\\.json)$" path != null;
            isScript = builtins.match ".*/scripts(/.*)?" path != null;
            isVersion = builtins.match ".*/VERSION$" path != null;
            isLicense = builtins.match ".*/(LICENSE|THIRD-PARTY-LICENSES)$" path != null;
          in
          keep || isScript || isVersion || isLicense || craneLib.filterCargoSources path type;

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

        htmlspec = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          doCheck = false;
          meta.mainProgram = "htmlspec";
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
        checkSuite = craneLib.mkCargoDerivation (commonArgs // {
          inherit cargoArtifacts;
          pnameSuffix = "-check-suite";
          # Every script, not just the entry point. `scripts/check` runs the others as
          # programs, so the sandbox resolves each of their shebangs too — and an
          # unpatched one took CI red for five merges with `/usr/bin/env: bad
          # interpreter`. Patching only the script this phase names is the mistake that
          # hid, because the failure surfaces one level in. A script added to the suite
          # must be added here in the same change.
          buildPhaseCargoCommand = ''
            patchShebangs scripts/check scripts/check-prompt scripts/check-release scripts/check-installed \
              scripts/check-publish-crates scripts/publish-crates
            ./scripts/check
          '';
          # `python3` for the prompt, release, and publisher checks, all standard library
          # only. The prompt one drives a real pty, which is why it is not a `cargo test`; the
          # release one reads facts about this repository, which a shipped crate's tests
          # must not depend on. The publisher's serves a stub index on the loopback interface,
          # which the build sandbox provides, and reaches nothing beyond it.
          #
          # `gitMinimal` because that check runs `git init` per case: the prompt's
          # project resolution stops at a repository boundary, so a scratch directory
          # that is not a repository does not exercise the behavior under test. The
          # script refuses outright when `git` is absent rather than testing something
          # else. Minimal rather than full `git` — `git init` is the only command it
          # runs, and full `git` pulls a Perl closure this suite has no use for.
          nativeBuildInputs = [ rustToolchain pkgs.bash pkgs.python3 pkgs.gitMinimal ];
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
        };

        devShells.default = craneLib.devShell {
          packages = [
            rustToolchain
            pkgs.cargo-nextest
            pkgs.jq
          ];
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
