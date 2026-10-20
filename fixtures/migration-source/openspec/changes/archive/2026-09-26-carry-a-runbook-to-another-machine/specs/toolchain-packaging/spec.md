# Spec Delta

## ADDED Requirements

### Requirement: The command surface checks a runbook's machine prerequisites

The command `htmlspec preflight <runbook>` SHALL report whether the selected runbook can start
in the invocation's working environment. It SHALL support the existing `--store` and `--json`
selectors. Store selection SHALL locate the runbook and its records, while machine requirements
SHALL be evaluated against the invocation's working directory and environment, not the store's
directory. Names SHALL follow the existing runbook-name rules.

The report SHALL identify the runbook, carried version, source location, working directory,
aggregate readiness, and each checked requirement with its expected condition, observation,
outcome, and remedy for a refusal. Results SHALL follow document order. Human output SHALL lead
with readiness. JSON output SHALL contain one structured report with no probe output interleaved.

A completed preflight with unmet requirements SHALL exit 1, a successful one SHALL exit 0, an
invalid invocation, invalid contract, or operational failure SHALL exit 2, and cancellation SHALL
exit 3. Unknown observations such as an unavailable manifest environment or a timed-out probe
SHALL be reported as unmet readiness rather than as success. An invalid declaration SHALL remain
distinguishable from a valid check whose condition is false.

`htmlspec run <runbook> --as <record>` SHALL apply the same checks and exit distinctions before
creating a run. `htmlspec run <runbook> --check` SHALL continue to report existing run conformance
without checking the present machine. Existing JSON fields on successful run creation SHALL be
preserved, with version and preflight information added.

#### Scenario: An agent inspects readiness
- **WHEN** the operator requests preflight with `--json`
- **THEN** standard output is one parseable readiness report, no prompt occurs, and probe output stays inside the report

#### Scenario: The selected store differs from the working directory
- **WHEN** a stored runbook is checked from a separate target checkout
- **THEN** the report identifies both locations and evaluates working-tree assumptions in the target checkout

#### Scenario: Several prerequisites fail
- **WHEN** several valid checks report unsatisfied conditions
- **THEN** one report names all of them in document order and the command exits 1

#### Scenario: The contract is malformed
- **WHEN** a selected runbook contains an invalid machine declaration
- **THEN** the command exits 2 and identifies the requirement and invalid field

#### Scenario: Reviewing existing records
- **WHEN** the operator runs `htmlspec run <runbook> --check` on a machine missing a prerequisite
- **THEN** it reports run conformance under its existing advisory contract
