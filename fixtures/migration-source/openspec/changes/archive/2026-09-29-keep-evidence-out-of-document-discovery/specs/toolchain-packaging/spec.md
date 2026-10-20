## ADDED Requirements

### Requirement: An operation over a directory collects documents and not a change's evidence

When `validate` or `format`, in either its writing or its checking mode, is given a directory or no path at all, it SHALL operate on every `.html` file beneath that directory, except files beneath a change's evidence as artifact-kinds' "A change's evidence is kept in the change and is not one of its documents" places it. Every other operation that collects a project's documents by walking a directory SHALL make the same exclusion from the same definition.

**A path the contributor names SHALL be checked as named.** A file means that file. A directory that is itself a change's evidence directory, or lies beneath one, means that directory, and `validate` and `format --check` SHALL check every `.html` file beneath it. The exclusion applies to what a walk comes across and never to what a contributor asked for. A contributor who wants to know whether a record still conforms can ask, and the answer is the ordinary one.

**Checking is not writing.** `format` in its writing mode SHALL NOT write a file beneath a change's evidence, including one a contributor names, because artifact-kinds states that no operation rewrites one. Given such a path, it SHALL name each record, write nothing at all, and exit with the status for an operation that could not run. It SHALL NOT write the other documents and skip the records. A partial write would report success for a request it half ignored.

Where the walked directory is not within a project, no directory is a change's evidence, and every `.html` file beneath it SHALL be operated on. A conformance fixture and a loose directory of documents keep the behavior they have.

The exclusion SHALL be defined once and reached by every such operation. It SHALL NOT be restated per command. A second definition that drifts from the first would have `validate` and `format` disagree about which files are documents. That is the fault this requirement exists to remove, arriving by another route.

#### Scenario: Validating a project that holds HTML evidence
- **GIVEN** a project whose documents all conform, and a change whose evidence holds an HTML file that does not
- **WHEN** a contributor runs `validate` with no path
- **THEN** it succeeds and reports no diagnostic naming the evidence file

#### Scenario: Formatting leaves evidence untouched
- **GIVEN** a change whose evidence holds an HTML file that is not in canonical form
- **WHEN** a contributor runs `format` over the project, and then `format --check`
- **THEN** the evidence file is byte-identical to what it was, and the check succeeds

#### Scenario: Naming an evidence file checks it
- **GIVEN** a change whose evidence holds an HTML file that does not conform
- **WHEN** a contributor runs `validate` naming that file
- **THEN** the file is validated and its diagnostics are reported

#### Scenario: Naming the evidence directory checks what it holds
- **GIVEN** a change whose evidence holds an HTML file that does not conform
- **WHEN** a contributor runs `validate` naming the change's evidence directory
- **THEN** the file is validated and its diagnostics are reported

#### Scenario: Formatting a named record is refused
- **GIVEN** a change whose evidence holds an HTML file that is not in canonical form
- **WHEN** a contributor runs `format` naming that file, or the evidence directory holding it
- **THEN** nothing is written, each record is named, and the exit is the one for an operation that could not run, while `format --check` naming the same path still reports the file

#### Scenario: Outside a project every file is a document
- **GIVEN** a directory that is not within a project, holding `evidence/a-record.html`
- **WHEN** a contributor runs `validate` naming that directory
- **THEN** the file is validated, because outside a project there is no change for it to be the evidence of
