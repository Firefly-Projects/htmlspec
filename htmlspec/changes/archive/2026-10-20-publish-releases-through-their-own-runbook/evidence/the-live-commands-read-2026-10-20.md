# The commands of steps 3, 5, and 7, read and not run

Task 3.2, recorded on 2026-10-20. Each command is printed as the publish runbook states it, with
this release's values. `MERGE` stays a name, because it is the public merge commit and no export
has been merged yet. Nothing below was run, and every call to the public services stays for the
first publication.

## Step 3: the bootstrap, first run only

```
gh api -X PUT repos/Firefly-Projects/htmlspec/rulesets/25362895 -f enforcement=disabled
publish/export --ref SOURCE --bootstrap --reviewed TREE
gh api -X PUT repos/Firefly-Projects/htmlspec/rulesets/25362895 -f enforcement=active
gh api repos/Firefly-Projects/htmlspec/rulesets/25362895 --jq .enforcement    # must print: active
```

GitHub's REST documentation, "Update a repository ruleset"
(`PUT /repos/{owner}/{repo}/rulesets/{ruleset_id}`), lists every body parameter, `name`,
`target`, `enforcement`, `bypass_actors`, `conditions`, and `rules`, as optional, so a body
holding `enforcement` alone changes that field. The values of `enforcement` are `disabled`,
`active`, and `evaluate`. "Get a repository ruleset" returns `enforcement` as a required field
with the same values, so the `--jq .enforcement` check always has a value to print. `gh api`
sends `-f enforcement=…` as a JSON string field, and `-X PUT` overrides the `POST` its `-f` would
otherwise imply. The ruleset's id, 25362895, and its present enforcement, `active`, are the ones
task 1.2 read.

## Step 5: the tag, on the public merge commit

```
gh api repos/Firefly-Projects/htmlspec/git/refs -f ref=refs/tags/v0.34.0 -f sha=MERGE
gh run list --repo Firefly-Projects/htmlspec --workflow release.yml
```

"Create a reference" (`POST /repos/{owner}/{repo}/git/refs`) requires `ref`, "the name of the
fully qualified reference … If it doesn't start with 'refs' and have at least two slashes, it will
be rejected", and `sha`, "the SHA1 value for this reference". `refs/tags/v0.34.0` has both. `gh
api` sends a `POST` because parameters were added: its help says "The default HTTP request method
is `GET` normally and `POST` if any parameters were added." The ref is a lightweight tag on
`MERGE`, which the release workflow's `gh release create --verify-tag` accepts as an existing tag.
The documentation does not say whether creating a ref through this endpoint starts a `push`
workflow, so the second command, and the risk the design names, stay.

## Step 7: the co-owner, first release only

```
cargo owner --add github:Firefly-Projects:firefly-developers htmlspec-schema
cargo owner --add github:Firefly-Projects:firefly-developers htmlspec-parse
cargo owner --add github:Firefly-Projects:firefly-developers htmlspec-assets
cargo owner --add github:Firefly-Projects:firefly-developers htmlspec-format
cargo owner --add github:Firefly-Projects:firefly-developers htmlspec-validate
cargo owner --add github:Firefly-Projects:firefly-developers htmlspec-cli
cargo owner --list htmlspec-cli
```

Each runs with `CARGO_HTTP_USER_AGENT` set to the output of
`python3 scripts/release_facts.py user-agent`. The cargo book's `cargo owner` page gives
`cargo owner [options] --add login [crate]`, and the pinned cargo 1.90.0's help gives
`-a, --add <LOGIN>  Name of a user or team to invite as an owner`. The cargo book's "Publishing on
crates.io" says "The syntax for teams is currently `github:org:team`", that of team owners "While
they have permission to publish or yank versions of the crate, they *do not* have the ability to add
or remove owners", and that "In order to invite a team as an owner one must be a member of that
team".
So the step runs signed in as `smurphy8`, the crates' owner, who must be a member of
`firefly-developers`, and the team can publish the next release but cannot change who owns the
crates. The six names are `scripts/release_facts.py`'s `CRATES`, in their publishing order.
