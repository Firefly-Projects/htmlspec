# Base Engineering Instructions

## Purpose and Scope

These are cross-project defaults for software work. Project- and directory-level
instructions may specialize them with repository facts, commands, architecture,
and conventions.

Use the following meanings:

- **MUST**: required unless a higher-priority instruction explicitly overrides it.
- **SHOULD**: the default; diverge only for a concrete, stated reason.
- **MAY**: optional when useful.

Do not silently weaken a safety rule to reduce effort. If a rule is inappropriate
for the task, explain why and use the safest practical alternative.

---

# TIER 1: TYPE-DRIVEN ENGINEERING DOCTRINE

## 1. Treat Types as the Plan

Types are tools for constructing the program, not annotations added after it.

- Write or revise important domain types and function signatures before writing
  their implementations.
- Use the compiler's errors, inferred types, holes, and coverage checks as an
  implementation worklist.
- A material change in meaning SHOULD produce a visible change in the relevant
  types or contracts.
- If radically different or invalid implementations satisfy a signature, ask
  whether the signature grants too much power.

## 2. Move Invariants Out of Human Memory

Any correctness condition that developers or agents must remember is a candidate
for enforcement by types, constructors, module boundaries, tests, or tooling.

- MUST encode important domain distinctions instead of relying on comments or
  naming conventions.
- MUST NOT depend on callers remembering undocumented preconditions.
- SHOULD make violations fail as early as possible, preferably at compile time,
  otherwise at the system boundary.

## 3. Give Domain Meanings Distinct Representations

- Prefer strong domain IDs over raw strings or integers.
- Prefer distinct types for values with different meanings, units, provenance,
  authority, or lifecycle states even when their runtime representations match.
- Prefer sum types or state enums over flags, nulls, sentinel values, and invalid
  field combinations.
- Avoid semantic dumping grounds such as unstructured strings, generic JSON,
  catch-all maps, or unrelated error constructors.
- Use domain terminology consistently. Introduce a new term only when it names a
  real distinction, and define it where the project records its language.

## 4. Parse at Boundaries and Preserve Evidence

- Validate untrusted or weakly typed inputs at system boundaries.
- Prefer parsers or smart constructors that return a trusted domain value over
  predicates that return only a boolean.
- Once a value is validated, preserve that fact in its type; do not repeatedly
  revalidate it throughout the core.
- Keep raw constructors private when public construction could violate an
  invariant.
- Return structured errors that preserve actionable failure information.

## 5. Prefer Total Core Logic

Core and domain functions MUST be total unless partiality is explicitly represented
in their result type or isolated behind a justified boundary.

- Avoid unchecked indexing, incomplete pattern matches, null assertions,
  impossible-case panics, unsafe casts, and partial helpers.
- Use non-empty collections, bounded indices, result/option types, exhaustive
  matching, and validated constructors where appropriate.
- Treat a partial function as evidence that an input constraint or failure mode
  may be missing from the model.
- Never encode failure with an in-band sentinel value.

## 6. Grant the Least Power Necessary

Every concrete type, interface constraint, dependency, effect, and capability
grants implementation freedom.

- Generalize code when parametricity rules out unwanted behavior.
- Use the weakest sufficient constraint and the narrowest sufficient effect.
- Do not give core logic access to I/O, clocks, randomness, global state, or broad
  service interfaces unless it needs that capability.
- Pass narrow capabilities explicitly rather than making broad environments
  globally available.
- Prefer guarantees obtained from parametricity and shared type variables before
  adding bespoke proof machinery.

## 7. Put Relationships in Signatures

When correctness depends on relationships between inputs, outputs, states, or
resources, express those relationships directly when practical.

Examples include:

- input and output share an identity, owner, unit, shape, or length;
- a transition accepts only the source state and returns the destination state;
- an index is valid for the collection it indexes;
- a resource cannot escape its scope or be used after release;
- a result carries evidence of the property a caller needs next.

## 8. Use the Right Amount of Type Precision

The objective is not maximum type sophistication. It is the strongest practical
model that eliminates real failure modes without making ordinary work needlessly
opaque.

- Start from the operations and guarantees the system needs.
- Add only enough phantom parameters, witnesses, refinements, or dependent indices
  to make those operations safe.
- Prefer local invariants over globally entangled proofs when both provide the
  required guarantee.
- Hide proof-oriented implementation details behind ergonomic public APIs when
  callers do not need them.
- During discovery, provisional types MAY be used to expose unknowns. Before the
  change is complete, strengthen them to reflect the domain facts discovered.

## 9. Model Effects, State, and Resource Lifecycles

- Push side effects to boundaries and keep the decision-making core pure where
  practical.
- Represent important state transitions as explicit operations, not scattered
  mutations and boolean checks.
- Encode resource acquisition, ownership, cleanup, and legal use order with scoped
  APIs, brackets/context managers, typestates, linear ownership, or equivalent
  mechanisms supported by the language.
- Make retries, idempotency, cancellation, timeouts, and recovery behavior explicit
  when they affect correctness.

## 10. Do Not Evade Constraints

The following are design changes, not harmless implementation shortcuts:

- weakening a planned type;
- replacing a domain type with a primitive or generic container;
- stuffing new meaning into an existing string, integer, map, or constructor;
- adding a sentinel, default, unchecked cast, or impossible-case panic;
- suppressing a compiler warning, linter rule, or static analysis finding;
- disabling, deleting, or weakening a test to make it pass;
- changing behavior only in the test environment;
- bypassing a public API or invariant-preserving constructor.

MUST NOT make such a change silently. If one is genuinely required, state the
domain or engineering reason, make the exception narrow, and add compensating
verification where appropriate.

## 11. Let Tests Cover Residual Risk

Types and tests have different jobs.

- Types SHOULD prove structural validity, legal states, and relationships that
  hold for every execution.
- Tests SHOULD verify business semantics, algorithms, boundary behavior,
  integrations, failure handling, and properties not economically expressed in
  the type system.
- Do not use tests as the sole defense for an invariant that can be made
  unrepresentable at reasonable cost.
- Do not duplicate compile-time guarantees with low-value runtime tests.

---

# TIER 2: ENGINEERING PIPELINE (GATED)

Follow this order. Loop back whenever new information invalidates an earlier
assumption. Scale the ceremony to the risk, but do not skip the reasoning.

## 0. Establish Scope and Evidence

Before changing code:

- Read the active project instructions and the nearest relevant documentation.
- Inspect the current implementation, tests, build configuration, and recent local
  patterns before proposing a new one.
- Identify the requested outcome, affected boundaries, and explicit non-goals.
- Preserve unrelated user changes and avoid unrelated refactors.
- Ask a question only when the missing answer would materially change the design,
  risk, or user-visible result and cannot be recovered from the repository.

## 1. Domain Alignment (DDD-lite)

- Extract domain terms, workflows, actors, states, and invariants from code, docs,
  schemas, and tests.
- Identify module or bounded-context boundaries and integration points.
- Distinguish domain rules from transport, storage, and UI representations.
- If introducing a new term, define it or reject it; keep naming consistent.
- Record assumptions that affect the design.

For a small local change, this may be a brief internal checklist. For a feature or
cross-boundary change, make the model visible in the plan or project documentation.

## 2. Formal Methods Gate (Mandatory to Evaluate)

Evaluate this gate for every task. Activate it when correctness depends on event
order, state evolution, coordination, or uncertain interacting behavior.

### State, protocol, and coordination work

If the work includes concurrency, retries, queues, distributed coordination,
state machines, protocols, resource lifecycles, or multi-step orchestration:

1. Define the explicit state variables and states.
2. Define permitted transitions and their triggers.
3. Define invariants that must hold before and after every transition.
4. Define illegal states and forbidden transitions.
5. Define failure, timeout, retry, idempotency, cancellation, and recovery behavior.
6. Identify assumptions about ordering, duplication, delivery, and persistence.

If the repository already supports a modeling tool, add or update the smallest
runnable model/spec that can expose the relevant failure modes. Otherwise, write a
concise transition specification and encode it through types and tests. Do not add a
heavy formal-methods dependency without a clear payoff.

### Strategy and uncertainty work

If outcomes depend on heterogeneous actors, competing strategies, stochastic
behavior, or uncertain environmental responses:

1. State the hypothesis and decision the experiment informs.
2. Define actors, strategies, parameters, and outcome measures.
3. Fix seeds or otherwise make runs reproducible.
4. Include a baseline and meaningful comparison arms.
5. Define failure criteria and how results will be interpreted before running it.

Propose the smallest reproducible simulation or experiment that can distinguish the
important alternatives.

## 3. Types-First Design

Before implementation bodies:

- Define or revise the relevant domain types, state types, error types, and public
  signatures.
- Make illegal states unrepresentable where practical.
- Use strong domain IDs and validated constructors.
- Encode important preservation and transition relationships in signatures.
- Choose the narrowest necessary effects and capabilities.
- Sketch implementations with typed holes or temporary stubs when the language
  supports them; inspect each goal and local context.
- Case-split on the data that determines behavior, eliminate impossible cases, and
  refine recursively.

Development holes or stubs MUST NOT remain in completed code unless the repository
explicitly treats them as valid artifacts.

If implementation pressure suggests weakening a type, first decide whether it has
revealed a genuine domain fact or merely a path-of-least-resistance shortcut.

## 4. Tests-First

After the type-level contract is sketched and before the implementation is complete:

- Write or update tests that encode key examples, invariants, and failure behavior.
- Include negative tests for invalid external inputs and important edge cases.
- Prefer property-based tests for algebraic laws and broad invariants.
- Prefer state-machine or model-based tests for transition systems when practical.
- Add regression tests that fail for the observed defect and pass for the intended
  reason.
- Keep tests deterministic; control clocks, randomness, scheduling, and external
  services where possible.
- Do not test what the type system already proves; test runtime and business
  behavior.

If an existing test conflicts with the intended behavior, determine whether the
test, the implementation, or the requirement is wrong. Do not simply weaken the
test.

## 5. Implementation

- Implement the smallest coherent change that satisfies the contract and tests.
- Keep core logic pure and deterministic where practical.
- Isolate I/O and integrations behind narrow boundaries.
- Follow existing project conventions and reuse established abstractions.
- Avoid speculative frameworks, premature generalization, and unrelated cleanup.
- Do not optimize for the smallest diff when that would preserve a misleading or
  unsafe model.
- If diverging from an established convention, state the reason.

## 6. Verification

Verification is part of implementation, not an optional final gesture.

1. Run the narrowest relevant checks during iteration.
2. Run the repository's standard build, typecheck, tests, lint, and formatting
   checks before completion.
3. Run relevant integration, property, model, simulation, migration, or performance
   checks when the change requires them.
4. Inspect the final diff for unintended changes, constraint evasion, incomplete
   handling, and stale comments or docs.
5. Confirm that generated artifacts are current when applicable.

If a check cannot be run, report the exact check not run, the reason, and the exact
command or procedure needed. Never imply that an unrun check passed.

Report evidence, not intent.

---

# TIER 3: COMPLETION AND COMMUNICATION

## Definition of Done

A change is complete only when all applicable conditions hold:

- The implementation matches the requested behavior.
- Domain terms and boundaries remain coherent.
- Important invariants are enforced at the strongest practical layer.
- No development holes, unexplained unsafe operations, or silent constraint
  evasions remain.
- Relevant tests were added or updated and pass.
- Standard repository checks pass.
- Documentation, schemas, migrations, and examples are updated when behavior or
  public contracts changed.
- The final diff contains no unintended changes.
- Remaining risks, assumptions, and unverified items are explicitly reported.

## Handoff Format

Lead with the outcome. Keep the handoff concise and include:

1. What changed and why.
2. The important type, domain, or architectural decisions.
3. Verification performed and its results.
4. Any remaining risks, assumptions, or follow-up work.

Do not provide a progress narrative when evidence and results are available.

---

# MAINTAINING AGENT INSTRUCTIONS

When creating or revising instruction files:

- Keep universal working agreements in the base file.
- Keep repository facts, layout, exact commands, architecture, and conventions in
  the repository-level file.
- Put specialized rules in the nearest directory they govern.
- Prefer short, imperative, observable rules over aspirations such as "write clean
  code."
- For every prohibition, give the safe alternative or the condition for an
  exception.
- Avoid duplicating the same rule across instruction layers.
- Link to canonical detailed documentation instead of copying large references.
- Keep the active instruction chain within the harness's context limit; split by
  scope when it grows.
- Add or revise a rule after repeated observed failures, not merely imagined edge
  cases.
- Periodically remove stale, redundant, contradictory, or unenforceable guidance.
- Verify discovery from both the repository root and relevant nested directories by
  asking the harness to list and summarize its active instruction sources.

A project-level instruction file SHOULD tell the agent, at minimum:

- where the important code and documentation live;
- how to set up and run the project;
- exact build, typecheck, test, lint, and format commands;
- architecture and dependency boundaries;
- domain terminology and critical invariants;
- files that are generated or must not be edited directly;
- migration, compatibility, security, and release constraints;
- project-specific code review and definition-of-done requirements.
