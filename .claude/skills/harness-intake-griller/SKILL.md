---
name: harness-intake-griller
description: Use when a user has a rough product idea, feature request, bug-fix intent, or Harness improvement and wants to clarify intent before implementation. This skill grills intent one decision at a time until shared understanding is explicit, then runs this repository's Harness feature-intake workflow, creates or updates product docs and story packets, and prepares runnable work without starting implementation unless explicitly requested.
---

# Harness Intake Griller

Turn fuzzy intent into shared understanding first, then Harness-ready planning artifacts before any implementation starts.

Use this skill as the pre-run discussion gate for this repository. The first output is clarity, not code and not Harness paperwork. Once the user confirms the intent is understood, the output becomes intake classification, docs, stories, validation expectations, and a clear implementation handoff.

## Operating Boundary

Do not jump from user intent directly to implementation.

Do not treat Harness artifact creation as a substitute for understanding the user's idea. The discussion is successful only when the user can see their intent reflected back clearly enough to correct or approve it.

Do not start implementation or invoke a long-running agent execution unless the user explicitly asks to execute after the intake artifacts are ready. This skill owns the discussion and planning work before implementation.

## Required Preflight

Read the local agent instructions first. If `AGENTS.md` lists a Harness block, follow it.

Prefer these sources before asking questions when they exist:

- `AGENTS.md`
- `README.md`
- `docs/HARNESS.md`
- `docs/FEATURE_INTAKE.md`
- `docs/ARCHITECTURE.md`
- `docs/CONTEXT_RULES.md`
- `docs/TOOL_REGISTRY.md`
- `scripts/bin/harness-cli query matrix`
- relevant `docs/product/*`
- relevant `docs/stories/*`
- relevant `docs/decisions/*`

Before a step that could use an optional external tool, query the Harness tool registry:

```bash
scripts/bin/harness-cli query tools --capability <capability> --status present
```

If the capability is inactive or absent, skip cleanly and note the gap in the final trace or planning notes.

## Interview Loop

Ask exactly one question at a time when the answer is not discoverable from local files or prior conversation. Asking multiple questions at once is bewildering.

Walk down the design tree one branch at a time. Resolve dependencies between decisions before asking about downstream details. If a question can be answered by exploring the codebase or Harness docs, explore first instead of asking.

For each question:

- restate the current understanding in one sentence
- name the missing decision
- include a recommended answer
- explain why that recommendation is probably right

Prefer a few sharp questions over a long questionnaire, but do not stop merely because the work is safe to classify. Stop interviewing only when shared understanding is explicit enough to create accurate Harness artifacts, or when the user tells you to proceed with known uncertainty.

## Shared Understanding Gate

Before durable intake, story packets, product docs, or implementation handoff, reach a checkpoint with the user.

The checkpoint must make these fields clear enough that the user can correct them:

1. Problem: what pain or opportunity is being addressed.
2. Desired outcome: what should be true when the work succeeds.
3. Audience or operator: who benefits or uses the result.
4. Current behavior: what happens today.
5. Target behavior: what changes from today.
6. Non-goals: what should not be changed.
7. Constraints: product, technical, timing, UX, data, or workflow limits.
8. Decision chain: the important choices already made and what each choice unlocks.
9. Remaining uncertainty: what is still unknown and whether it blocks planning.

When the checkpoint is ready, present it concisely and ask for confirmation or correction. Do not proceed to artifacts until the user confirms, or until the user explicitly asks to continue despite unresolved uncertainty.

## Intake Gate

After the shared understanding gate, do not create story packets until these Harness fields are clear enough:

1. Outcome: what should be true for the user.
2. User-visible behavior or operational behavior: what changes from today.
3. Scope boundary: what may change and what is out of scope.
4. Source of truth: product docs, existing stories, issue, screenshot, logs, code path, or conversation note.
5. Risk lane: tiny, normal, or high-risk, using `docs/FEATURE_INTAKE.md`.
6. Validation proof: command, test, screenshot, API response, matrix row, rebuild, or review artifact.
7. Handoff rule: whether the result should stop at docs/stories or proceed to implementation after explicit approval.

If any field is weak, ask the next highest-leverage question.

## Workflow

1. **Clarify intent**
   - Convert the user request into a concise intent brief.
   - Grill one decision at a time until the idea is clear in the user's terms.
   - Capture goals, non-goals, assumptions, open questions, affected surfaces, and likely risks.

2. **Confirm shared understanding**
   - Present the checkpoint from the shared understanding gate.
   - Ask for confirmation or correction before creating Harness artifacts.
   - Keep interviewing if the user corrects a material part of the checkpoint.

3. **Record intake**
   - Classify the request using `docs/FEATURE_INTAKE.md`.
   - Record the durable intake row with `scripts/bin/harness-cli intake`.
   - Use `harness_improvement` when the work changes how humans and agents collaborate.

4. **Map docs and stories**
   - Identify existing product docs and story packets that already cover the request.
   - Update existing artifacts when they are the real source of truth.
   - Create new product docs, initiative notes, story packets, hierarchy, or dependency records only when the request needs them.

5. **Shape validation**
   - Define cheap checks for planning changes.
   - Define final proof for the eventual implementation.
   - Keep proof concrete enough for `scripts/bin/harness-cli story verify <id>` when possible.

6. **Prepare implementation handoff**
   - Mark or create story records with correct lane, status, and verify command.
   - Ensure dependencies and hierarchy are recorded when relevant.
   - State which story is ready for implementation and what should remain manual review.

7. **Stop before execution**
   - Summarize the intake result.
   - List created or updated artifacts.
   - Name the ready story and its next step only as a suggestion unless the user explicitly asked to start execution.

## Artifact Standards

For tiny work:

- record intake
- patch directly only if the user asked for implementation after the intent is clear
- keep docs current
- run quick checks

For normal work:

- require a confirmed shared understanding checkpoint first
- create or update one story packet from the repo template
- update or reference relevant product docs
- define validation expectations
- add or update the durable story row

For high-risk work:

- require a confirmed shared understanding checkpoint first
- use the repo's high-risk story template
- document design, validation, and pause points
- ask for human confirmation before implementation if direction remains ambiguous
- record durable decisions when behavior, architecture, data ownership, API shape, authorization, or validation requirements change meaningfully

## Output Shape

During the interview, do not use a long final template. Ask the next single question.

At the shared understanding checkpoint, respond with:

```text
Current understanding:
- Problem:
- Outcome:
- Current behavior:
- Target behavior:
- Non-goals:
- Constraints:
- Key decisions:
- Remaining uncertainty:

Question:
- Is this right, or what should change?
```

When the intake is complete after confirmation, respond with:

```text
Intent brief:
- Outcome:
- Non-goals:
- Assumptions:
- Open questions:

Harness intake:
- Intake:
- Lane:
- Reason:
- Affected docs:
- Stories:
- Validation:

Implementation handoff:
- Ready story:
- Dependencies:
- Next step:
- Do not start until:
```

If implementation has not been explicitly authorized, end by making the handoff clear without starting it.

## Friction Rule

If the discussion reveals a missing Harness rule, stale doc, unclear source of truth, or repeated manual step, either fix it within scope or record a backlog item:

```bash
scripts/bin/harness-cli backlog add --title "<short name>" --pain "<what was hard>" --risk normal
```

Before final response, record a trace that includes intake id, actions, files read, files changed, validation, and any friction discovered.
