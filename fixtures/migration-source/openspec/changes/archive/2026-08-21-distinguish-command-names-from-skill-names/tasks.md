## 1. The check, before the fix

- [x] 1.1 Add a test in `crates/htmlspec-cli/src/install.rs` that installs for every tool in `TOOLS`, collects the `name:` value from every skill and command file that tool receives, and asserts no value appears twice within one tool. Scope it per tool: two tools declaring `htmlspec-apply-change` is correct, since each tool's namespace is its own
- [x] 1.2 Make the failure name the tool, the repeated value, and both file paths. "A name is duplicated" across 21 files is not actionable
- [x] 1.3 Run it and **watch it fail** on the 21 command files that Claude Code, Cursor, and Pi currently receive — 7 each. This is the naturally occurring failing case and the only chance to see the check work
- [x] 1.4 Record the failure output as evidence, with the per-tool count. Confirm Codex does not appear: it is `SkillsOnly`, so it has no second file to collide with
- [x] 1.5 Confirm `every_generated_name_is_loadable` passes throughout. Both duplicates are well-formed, so the character check is blind to this fault — that is the reason a second check exists rather than an extension of the first

## 2. Stop writing the name

- [x] 2.1 Change `write_commands` in `install.rs` to omit `name:` from the generated frontmatter, keeping `description`, `allowed-tools`, and `generatedBy`. `generatedBy` is required by an accepted requirement about recording the generating version and must not be dropped along with the name
- [x] 2.2 Replace the comment added by the previous change. It currently explains why the name is `workflow.directory` rather than the invocation; it must now explain why there is no name field at all, and record that the file path determines the invocation
- [x] 2.3 Confirm `workflow.directory` still has a caller in `write_skills`, so the field is not left dead
- [x] 2.4 Re-run the test from task 1 and confirm it passes
- [x] 2.5 Run the full suite. `every_generated_name_is_loadable` must still pass — it now reads only skill files' names for the three tools that changed, and it must not have become vacuous. Confirm it still finds 7 names per tool rather than 0

## 3. Nothing a contributor types has changed

- [x] 3.1 Confirm the command **file names** are untouched: `.claude/commands/htmlspec/apply.md`, `.cursor/commands/htmlspec-apply.md`, `.pi/prompts/htmlspec-apply.md`
- [x] 3.2 Confirm `report_next_command` still prints `/htmlspec:new`, `$htmlspec-new`, `/htmlspec-new`, and the AGENTS.md sentence, grouped per distinct spelling
- [x] 3.3 Confirm `Tool::invocation` still has exactly one non-test caller. This change does not alter that function, and `every_invocation_names_its_workflow` in `tools.rs` must still pass

## 4. Prove the omission does not break loading

- [x] 4.1 Regenerate this repository's instructions with `htmlspec init --tools claude,codex,pi` and confirm the 14 command files no longer carry `name:`. 14 here, not 21: this repository selects `claude,codex,pi` and Codex is skills-only
- [x] 4.2 Confirm the seven `/htmlspec:*` commands still load and are still invocable. This must be checked by loading rather than by reasoning — it is the risk that would make the change wrong rather than merely different. **Closed by a session load on 2026-08-21**, after the merge of #6 and the profile upgrade to `c864727`. The commands load and remain invocable, so omitting `name:` from a command file does not change what a contributor types
- [x] 4.3 Confirm a session load now lists seven html-spec entries rather than fourteen. That count is the symptom this change exists to remove, and it is the only direct check that it worked. **Closed by a session load on 2026-08-21.** The doubled listing is gone. This is the observation the `opsx` comparison could only argue for
- [x] 4.4 Record the result as evidence, including the `opsx` comparison: ten `.pi/prompts/opsx-*.md` omit `name:` and load, which is the reason to expect this to hold
- [x] 4.5 Note in the evidence that Cursor's command loader was **not** tested, since this repository does not select Cursor. The design records this as an accepted risk and it must not be reported as verified
- [x] 4.6 Confirm the AGENTS.md region is untouched by the regeneration

## 5. The contract

- [x] 5.1 Withdraw the `#### Scenario: One identity per workflow` scenario from `openspec/specs/project-bootstrap/spec.md`. This is a deliberate removal, the first in this capability, and it must touch nothing else
- [x] 5.2 Add a scenario requiring that no two files a tool receives declare the same name
- [x] 5.3 Amend the requirement's prose: it keeps the claim that a workflow has one generated identity, and drops the sentence asserting that identity is written into every file kind
- [x] 5.4 Confirm the requirement's other paragraphs and scenarios are intact, and that no other accepted requirement is touched. The counts stated when this task was written were wrong: the requirement had **ten** paragraphs and **twelve** scenarios, so it now carries eleven and thirteen. The diff is 9 insertions and 3 deletions, and the applied requirement is textually identical to the delta's
- [x] 5.5 Run `openspec validate --all` and confirm all seven capabilities pass
