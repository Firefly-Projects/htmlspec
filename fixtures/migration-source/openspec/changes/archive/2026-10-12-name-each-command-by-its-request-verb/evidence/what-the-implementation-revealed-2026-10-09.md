# What the implementation revealed

**Date:** 2026-10-09
**Task:** 5.4

## 1. A second heading for the verbs is help text, not a clap heading

Design decision 5 asks for the operations as before and then the five verbs "under their own
heading". clap 4 gives a command one heading for all of its subcommands
(`subcommand_help_heading`), so declaring the verbs beside the operations would have listed them in
the same "Commands:" block. They are declared hidden instead, and `operation::with_request_verbs`
writes the "Request verbs:" section after the options, from the table. The "Commands:" list reads
exactly as it did.

Hidden has one cost: clap suggests a similar subcommand for a mistyped one, and it does not suggest
a hidden one, so `htmlspec gte` is not answered with "did you mean `get`". Every verb, every object,
and `htmlspec get --help` still parse and print as declared. **This is a choice for the contributor
to settle**, with the alternative of listing the verbs among the operations.

## 2. The verbs' help is written from the table, and parsing reads it

The object's help line, "Runs `htmlspec validate`. Validate documents against the contract", is
written at run time from `Operation` and from the operation's own doc comment, so an operation is
described once. That needs the program to parse against the adjusted command rather than through
`Cli::parse`, which builds its own. `main` now does what `Cli::parse` does, `get_matches_mut` and
then `from_arg_matches`, against `command()`, and every unit test that renders help uses the same
function.

clap's `mut_subcommand` panics on an absent name and moves the subcommand to the end of the list.
The help writer uses `find_subcommand` and `mut_subcommands` instead, which are total and keep the
order, so a verb or object the table does not name is left as clap declared it, and the tests
report it.

## 3. `Invocation` is the one handler, and `Operation` stays a table

Design decision 4 has both spellings "reach the handler through `Operation`". `Operation` carries no
arguments, so the handler is reached through `Invocation`, one variant per operation, each carrying
that operation's `Args` struct. `From<Command>` writes each operation's two spellings in one arm,
so routing a verb's object to another operation's variant does not type-check unless its arguments
are the same type, and no two operations share an `Args` type. `Invocation::operation()` maps back
to the table, and it is compiled for tests only.

**A test cannot list the variants of an enum without a macro.** `every_operation_is_listed` places
each operation of `Operation::ALL` by a match with no wildcard, which catches a variant added to the
type and left out of the match, but not one left out of `ALL`. Two things close that:

- `the_command_line_offers_exactly_the_operations_under_both_spellings` requires clap's paths to be
  exactly the table's two spellings of each operation in `ALL`, so an operation declared to clap and
  missing from `ALL` fails;
- with `Invocation::operation()` compiled for tests only, `ALL` is the one place outside the tests
  that constructs an `Operation`, so a variant left out of it is never constructed, and
  `clippy -D warnings` refuses it as dead code.

## 4. `--strict` cannot be told apart by running the program

The parity tests run `validate --strict` under both spellings, and they would pass if the verb
dropped `--strict`. Strict mode promotes warnings, and every warning a real rule reports is in
`review/*`, which strict mode does not promote by contract, so no document the fixture can build
makes `--strict` change the outcome. A dropped option is caught by
`both_spellings_of_every_operation_parse_to_one_invocation` instead, which parses both spellings
with every option and compares the invocations, and `every_option_is_given_in_the_parse_tests`
keeps that test giving every option clap declares. Mutation 2 in `mutation-testing-2026-10-09.md`
confirms it.

## 5. A usage error names the spelling typed

The requirement asks both spellings to "produce the same output and exit status" for an operation
they run. Where clap refuses the arguments before any operation runs, the exit status is the same,
2, and the usage line names the spelling typed:

```
$ htmlspec validate --bogus          $ htmlspec get validity --bogus
error: unexpected argument '--bogus' found   (identical)
Usage: htmlspec validate [OPTIONS] [PATH]    Usage: htmlspec get validity [OPTIONS] [PATH]
```

That is what decision 4 chose when it rejected rewriting the arguments before clap parses them,
"Help and error messages would then speak of a command the contributor did not type". The parity
tests cover the invocations that reach an operation, refusals the program writes itself among them.

## 6. The configuration directory resolves as a project

A refusal case in `store_add_and_post_registration_agree` registered `../config`, the test's
`XDG_CONFIG_HOME`, as a location holding no project, and `store add` accepted it. The registry lives
at `<config>/htmlspec/stores.json`, and `ProjectRoot::resolve` treats any directory holding an
`htmlspec/` directory as a project. So on a real machine `~/.config` resolves as an HTMLSpec project:
`htmlspec store add x ~/.config` registers it, and a project-wide command run from inside
`~/.config` acts on it. This change does not cause or alter it, and the case now registers an absent
path. **Recorded as a follow-up**, outside this change's scope: a project's root might require its
declaration, `spec.nix`, beside `htmlspec/`.

## 7. Measured, not read

- `archive`, in a project holding a runbook, writes `runbooks.json` beside the archived change's
  `VERSION`, and at the first publication `runbook-releases.json` at the planning root. Design
  decision 2 now names both; the verb stays `patch`.
- `init --tools none` on an established project writes nothing, and `init` and `format` run twice
  change nothing the second time, which is the `put` claim.
- The installed instruction lines that invoke a command are now 348, where the proposal counted 300.
- The OpenSpec-spelling measurement of task 1.2, repeated with the built change, is byte for byte
  the one taken at HEAD, over all 42 invocations.
