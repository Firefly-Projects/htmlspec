# Mutation testing

**Date:** 2026-10-09
**Task:** 5.2

Each mutation was applied to the working tree by a script in the session scratchpad. It backed up
each file the mutation edits, checked that every anchor matched exactly once before writing
anything, and applied the edits. Then it ran
`cargo test -p htmlspec-cli --bin htmlspec --test verbs --test cli --no-fail-fast` and
`cargo test -p htmlspec-assets --lib --no-fail-fast`, and for mutation 9 `scripts/check-installed`
outside `nix develop`. Then it restored each file from its backup, compared the two byte for byte,
and touched the file so cargo would rebuild it. All 10 restores matched, and afterwards every
backup of `main.rs` and `operation.rs` equalled the current file, and `commands.rs` and `sync.md`
equalled HEAD. Every mutation compiled. No run wrote a `proptest-regressions/` directory.

**All nine mutations the task names are caught, and so is 8b**, which was added because mutation 8
can only be caught at run time where a test reaches the mutated message. No mutation survived, so
no test had to be added after the run. The test that catches 8b,
`no_program_message_names_a_request_verb_spelling`, was written at task 3.3 for exactly this case.

| # | Mutation | Caught by |
|---|---|---|
| 1 | One verb spelling runs another operation: `get status` and `get sync` swap objects | `both_spellings_of_every_operation_parse_to_one_invocation`, `status_and_get_status_agree`, `sync_and_get_sync_agree`, `every_get_operation_leaves_the_project_as_it_was`, `an_objects_help_shows_its_operations_options` |
| 2 | A verb spelling drops an option: `get validity --strict` parses and is not passed on | `both_spellings_of_every_operation_parse_to_one_invocation` |
| 3 | `--check` ignored under `put format` | `both_spellings_of_every_operation_parse_to_one_invocation`, `format_and_put_format_agree`, `a_withholding_option_writes_nothing_under_the_verb` |
| 4 | `--check` ignored under `post promotion` | `both_spellings_of_every_operation_parse_to_one_invocation`, `migrate_promote_and_post_promotion_agree`, `a_withholding_option_writes_nothing_under_the_verb` |
| 5 | An operation that writes is spelled with `get`: `format` moved to `get format` in the table, in clap's declaration, and in the routing | `each_request_verb_spelling_is_the_one_the_design_fixed`, `help_documents_every_subcommand_and_option`, `the_top_level_help_lists_the_operations_and_then_the_verbs`, `each_verbs_help_lists_its_objects_with_the_operation_each_runs`, `an_objects_help_shows_its_operations_options`, `format_and_put_format_agree`, `a_withholding_option_writes_nothing_under_the_verb` |
| 6 | Two operations share a verb spelling: `store list` spelled `get list` in the table | `the_request_verb_spellings_are_distinct`, `each_request_verb_spelling_is_the_one_the_design_fixed`, `the_command_line_offers_exactly_the_operations_under_both_spellings`, `both_spellings_of_every_operation_parse_to_one_invocation`, `each_verbs_help_lists_its_objects_with_the_operation_each_runs`, `the_top_level_help_lists_the_operations_and_then_the_verbs` |
| 7 | A verb's help omits an object: `get survey` hidden | `each_verbs_help_lists_its_objects_with_the_operation_each_runs`, `help_documents_every_subcommand_and_option` |
| 8 | A program message names a verb spelling: `init`'s closing advice says `htmlspec post change <name>` | `no_program_message_names_a_request_verb_spelling`, `init_and_put_project_agree`, and two existing `cli.rs` tests that read that advice, `bare_init_establishes_a_project_and_names_the_next_command` and `no_tools_establishes_the_project_and_names_no_tool_command` |
| 8b | The same in a message no command-line test reaches: `store add`'s refusal says `` `htmlspec put project` `` | `no_program_message_names_a_request_verb_spelling` |
| 9 | A workflow body names a verb spelling: `sync.md` says `` `htmlspec get sync` `` | `no_workflow_names_a_request_verb_spelling`, `every_get_operation_leaves_the_project_as_it_was` (its `instructions sync --json` case prints the body), and `scripts/check-installed`, exit 1 |

## Notes

**Mutation 2 is caught by one test, and only a unit test can catch it.** `--strict` promotes warnings,
and every warning a real rule reports is in `review/*`, which strict mode does not promote by
contract. So no document makes `validate --strict` print or exit otherwise than `validate`, and
the parity test `validate_and_get_validity_agree`, which runs `--strict` under both spellings,
passes with the option dropped. `both_spellings_of_every_operation_parse_to_one_invocation` parses
both spellings with every option and compares the invocations, which is where a dropped option
shows. `every_option_is_given_in_the_parse_tests` keeps that test giving each option clap declares,
so a later option cannot be left out of it.

**Mutation 5 is the consistent form.** Changing only the table, `Verb::Put` to `Verb::Get`, is caught
at once by the design-table test and the command-tree test. The mutation here moves `format` in
all three places the program states it, so that only tests that know what `format` does, or what
the design fixed, can catch it. Seven do. The `get` test does not, because it runs the `get`
operations its own table lists, and `format` is not one; the help tests are what hold that table to
what the program offers.

**Mutation 8 against 8b.** Mutation 8 edits a message that `init` prints on every success, so the
command-line tests that run `init` see it, and so do two existing `cli.rs` tests that check the
advice line. Mutation 8b edits `store add`'s refusal of a location that holds no project, which no
command-line test reaches. Only the source scan catches it, and that difference is why the scan
exists.
