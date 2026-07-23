# Create Jira Subtasks

## Required inputs

Required before generating output:

1. Parent Story — title, description, acceptance criteria, story points if available
2. Splitting input — notes, scope decisions, technical breakdown, or explicit subtask list

If either is missing, ask for it. Ask for the parent story first, then the splitting input.
Ask one question at a time; wait for the answer before asking the next.
Offer a recommended answer when a reasonable default exists.

If a required input can be gathered by exploring the workspace (Jira issue in context, related repository, prior conversation), explore first instead of asking.

Do not guess, infer, or substitute missing inputs. Do not generate subtasks from incomplete information.

Optional inputs (use when provided, do not ask for them):

- max_subtasks
- story points (scope signal only)
- component context

## Clarifying ambiguous input

Do not resolve unclear or conflicting input by guessing — ask.

- Vague or overloaded term (e.g. "account", "sync", "the service") → ask which concrete meaning is intended. Offer a recommended interpretation when reasonable.
- Parent story and splitting input conflict, or splitting input references scope/terminology/components absent from the parent story → ask which one is authoritative. Do not silently prefer one side.
- Workspace exploration contradicts the parent story or splitting input (different naming convention, already-existing implementation, stale branch) → surface it and ask how to proceed.

Ask one clarifying question at a time; wait for the answer before continuing.

## Output contract

Generate one or more Jira Subtasks, each as `summary`, `narrative`,
`acceptance_criteria`, and optional `out_of_scope` — the fields
`create_jira_subtask` takes. The server renders these into Jira wiki markup;
do not format markup yourself.

Do not output: Components, Story Points, ticket IDs, priority, explanations
before/after the ticket, implementation plans not supported by the input.

## Source handling

Parent story and input are authoritative.

Use the parent story for: business goal, current problem, accepted target behavior, scope boundaries, terminology, constraints, dependencies between subtasks.

Use input to determine the actual subtasks.

If they conflict, follow input only when it explicitly overrides the parent story; otherwise preserve the parent story's scope.

Story points are a scope signal only (Fibonacci scale; up to 8 typical in a three-week sprint):

- Lower values → fewer, smaller subtasks.
- Higher values → more decomposition, but only when the input contains separable work.
- Story points must not override the scope described by the parent story and input.

Do not copy the parent story text verbatim or recreate it as a subtask.
Do not create subtasks outside the parent story scope.
Do not create a separate "Background Context" section.

### Workspace context

Use the workspace for additional context when it supports the parent story or input: project structure, existing interfaces/contracts, naming conventions, module boundaries.

Before relying on workspace content, check that relevant repositories are up to date (branch status, pending remote changes). Warn the user if a repository appears stale.

## Audience requirements

Each subtask must work for two readers:

- Product Owner: understands how the subtask contributes to the parent story.
- Developer: understands what needs to change, why it's separate, and how completion is verified.

Understandable without rereading the full parent story, but must not repeat it. Title and opening narrative must be quick to grasp; the full description stays concise.

## Narrative quality requirements

Write each subtask as a compact narrative, not a documentation dump. Carry the reader from: relevant parent story context → specific gap/unit of work → consequence if missing → intended target state → verifiable completion.

Do not merely list tasks. Make the reason for the subtask obvious before technical details appear.

Surface hesitation points before they become ambiguity: unclear scope boundaries, non-obvious constraints, dependency on sibling subtasks, risks from duplicated logic, manual work, drift, missing automation, or inconsistent behavior.

Do not dramatize, use theatrical language, or write metaphorically. Apply narrative structure through clarity, sequence, and emphasis alone.

Do not retell the full parent story or copy its text verbatim. Compress only the context needed for this specific subtask.

## Language and writing style

Write in American English.

Every sentence must earn its place. Short, clear descriptions over long explanations. No boilerplate, no restating the title in the description, no marketing language, no design-document tone, no mechanical plan listing. Prefer concrete wording.

Lead with business or delivery value before technical details. Answer: why does this work matter, what's broken/missing/duplicated/risky/inefficient today, what will be true after this subtask is done, how will completion be verified.

Name tools, repositories, files, protocols, tag patterns, or systems when provided. Preserve relevant technical terms from the parent story and input.

Do not invent: implementation details, components, story points, priority, dependencies, exclusions, acceptance behavior. Use `<TBD>` only if a value is genuinely undecided.

## Subtask decomposition requirements

Create multiple subtasks only when the work contains independently executable/verifiable units. If the input describes one coherent unit, generate one subtask.

Each subtask must: represent a concrete unit of work, fit under the parent story, have a specific action-oriented title, include enough context to stand alone, avoid duplicating the full parent story, include observable completion criteria.

Calibrate decomposition depth to story points:

- 1–2: one subtask unless the input clearly contains separable work.
- 3–5: multiple subtasks only when separate work units are present.
- 8: expect multiple subtasks when the input contains separable implementation, migration, integration, data, validation, or behavior changes.
- Do not create extra subtasks to match a story point value, or split a coherent unit because the value is high.

Do not split by artificial phases ("analysis", "implementation", "testing") unless explicitly requested as separate items.
Do not create subtasks for project management, coordination, review, or communication unless explicitly requested.
Do not create duplicate subtasks for the same behavior, or testing-only subtasks unless testing is explicitly separable.

Respect `max_subtasks` when provided: do not exceed it; if it forces compression, merge related work into coherent subtasks without mentioning the compression.

## Title requirements

Use concrete wording; do not restate the parent story title.

Good: "Add validation for import configuration", "Replace manual mapping with generated rules", "Persist retry state for failed exports"

Bad: "Validation", "Implement changes", "Testing", "Improve logic", "Parent story subtask"

## Description layout requirements

Do not add sections beyond `narrative`, `acceptance_criteria`, `out_of_scope` unless explicitly requested: Scope, Technical Notes, Dependencies, Test Notes, Open Questions, Hints, Background Context.

## Narrative field requirements

Start with "As a [role], I/we want [goal], so that [benefit]." only when a meaningful stakeholder perspective exists; otherwise skip it and start directly with the narrative.

Must explain: what part of the parent story this delivers, why it's separate, who benefits (when relevant), what problem/gap exists today, what target state should be true after completion.

First paragraph must be understandable to a Product Owner; avoid jargon unless self-explanatory. Technical specifics belong later or in the acceptance criteria.

Do not start with: "This ticket is part of...", "This is one step of...", "The goal of this ticket is...", "This subtask is about...". Tell the story directly.

Surface constraints or risks when relevant. Do not repeat the full parent story background or restate its title.

## Acceptance criteria requirements

Use concrete values where known: repository names, tag patterns, file paths, package names, system names, expected status codes, artifact names.

Avoid vague outcomes ("it works correctly", "the pipeline is improved", "the user can proceed"). Name observable outcomes: a file exists, a job runs/is skipped, an artifact is published, an HTTP response returns 200, a version string matches a pattern, a configuration references a shared template, an expected value is stored, an invalid input is rejected, an existing behavior remains unchanged.

## Out of Scope requirements

Do not invent exclusions or add entries just to fill space.

## Metadata handling

Components and Story Points are Jira metadata — use as context only, never output them. Do not mention Story Points in the Description unless the input explicitly requires a sizing discussion.

## Anti-patterns

Do not generate a parent story or summarize it as a subtask.
Do not create subtasks that only rename acceptance criteria.
Do not split one tightly coupled change into artificial subtasks.
Do not create vague subtasks ("Update logic", "Add tests", "Clean up code", "Improve handling") unless the input provides concrete scope.
Do not prescribe exact file names, function signatures, YAML snippets, or implementation structure unless provided as requirements.
Do not copy parent story text verbatim.
Do not write acceptance criteria as checkbox lists or vague scenarios.
Do not output explanations before or after the subtasks.

## Cross-impact revision

When elaborating a later subtask reveals information affecting an earlier one in the same session, revise the earlier subtask directly — do not append a notes block or leave it with a comment to fix later. Final output must contain every subtask in its finished, correct state.

Revise an earlier subtask when a later one's elaboration reveals: missing acceptance criteria, scope overlap/duplication, an unaccounted dependency, a constraint/assumption that changes the target state, or naming/interface/contract decisions the earlier subtask must align with.

Keep revisions minimal — change only what the new information requires.
