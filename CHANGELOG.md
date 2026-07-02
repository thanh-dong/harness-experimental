# Changelog

## 2026-07-02 - PR #5

- Event-log durable layer: git-native team state (US-028) (@thanh-dong)
- Merge commit: `ec50517847bd9da61dc352f97c1d467b31c4051b`
- Harness CLI release: `harness-cli-v0.1.13`
- Changed files:
  - `.claude/skills/reverse-tornado-okr/SKILL.md`
  - `.claude/skills/reverse-tornado-okr/agents/openai.yaml`
  - `.claude/skills/reverse-tornado-okr/contracts/handoff-contract.v1.json`
  - `.claude/skills/reverse-tornado-okr/references/artifact-guide.md`
  - `.claude/skills/reverse-tornado-okr/references/integrity-store.md`
  - `.claude/skills/reverse-tornado-okr/references/learning-memory.md`
  - `.claude/skills/reverse-tornado-okr/references/operating-loop.md`
  - `.claude/skills/reverse-tornado-okr/references/storage-idempotency.md`
  - `.claude/skills/reverse-tornado-okr/scripts/okra-store.sh`
  - `.claude/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py`
  - `.codex/skills/reverse-tornado-okr/SKILL.md`
  - `.codex/skills/reverse-tornado-okr/agents/openai.yaml`
  - `.codex/skills/reverse-tornado-okr/contracts/handoff-contract.v1.json`
  - `.codex/skills/reverse-tornado-okr/references/artifact-guide.md`
  - `.codex/skills/reverse-tornado-okr/references/integrity-store.md`
  - `.codex/skills/reverse-tornado-okr/references/learning-memory.md`
  - `.codex/skills/reverse-tornado-okr/references/operating-loop.md`
  - `.codex/skills/reverse-tornado-okr/references/storage-idempotency.md`
  - `.codex/skills/reverse-tornado-okr/scripts/okra-store.sh`
  - `.codex/skills/reverse-tornado-okr/scripts/okra-verify-artifact.py`
  - `.gitattributes`
  - `.gitignore`
  - `.harness/events/b6b55092-5630.jsonl`
  - `.harness/events/migration.jsonl`
  - `.okra/content/sha256/069a76733931721685775150c582dd2ff389df313ee8f802e383ddc7cbb6a085`
  - `.okra/content/sha256/17909a8d21bb63ef7a3fe32921f78745e81aef29e14159d7c4c3843b4dd2924e`
  - `.okra/content/sha256/6cd587dc1d47850324569e83c5aa48ee8c178d407ccc1aa6450951ce82dbc894`
  - `.okra/content/sha256/7a204902a5a40795d02db5f4e83e8cbbb5e8a5006ec42fc10b6e1681d230c14e`
  - `.okra/content/sha256/f44777980d4ba8f067ceb1ac1b3840f4bee9784448a304cad8e4afee4e8b42f3`
  - `.okra/runs/us-028-event-log/checkins.jsonl`
  - `.okra/runs/us-028-event-log/flags.jsonl`
  - `.okra/runs/us-028-event-log/frame/current`
  - `.okra/runs/us-028-event-log/frame/frame.v1.json`
  - `.okra/runs/us-028-event-log/ledger.jsonl`
  - `.okra/runs/us-028-event-log/status.md`
  - `.okra/runs/us-028-event-log/tree/current`
  - `.okra/runs/us-028-event-log/tree/tree.v1.json`
  - `.okra/runs/us-028-event-log/workers/DKR-1/progress.jsonl`
  - `.okra/runs/us-028-event-log/workers/DKR-2/progress.jsonl`
  - `.okra/runs/us-028-event-log/workers/DKR-3/progress.jsonl`
  - `.okra/runs/us-028-event-log/workers/DKR-4/progress.jsonl`
  - `AGENTS.md`
  - `CLAUDE.md`
  - `Cargo.lock`
  - `crates/harness-cli/Cargo.toml`
  - `crates/harness-cli/src/application.rs`
  - `crates/harness-cli/src/domain.rs`
  - `crates/harness-cli/src/events.rs`
  - `crates/harness-cli/src/infrastructure.rs`
  - `crates/harness-cli/src/interface.rs`
  - `crates/harness-cli/src/main.rs`
  - `docs/EVENT_LOG.md`
  - `docs/HARNESS.md`
  - `docs/HARNESS_BACKLOG.md`
  - `docs/SETUP.md`
  - `docs/TEST_MATRIX.md`
  - `docs/TOOL_REGISTRY.md`
  - `docs/decisions/0008-us-028-goal-loop-frame.md`
  - `docs/decisions/0009-event-log-source-of-truth.md`
  - `docs/decisions/README.md`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028-event-log-durable-layer.md`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028a-shadow-mode/US-028a-shadow-mode.md`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028a-shadow-mode/implementation-notes.html`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028b-cutover/design.md`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028b-cutover/execplan.md`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028b-cutover/implementation-notes.html`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028b-cutover/overview.md`
  - `docs/stories/epics/E05-event-log-durable-layer/US-028b-cutover/validation.md`
  - `scripts/calibrate-harness.sh`
  - `scripts/install-harness.sh`
  - `scripts/schema/007-ulid-ids.sql`

## 2026-06-16 - PR #4

- feat(harness): mine implementation-note signals in propose (US-027) (@thanh-dong)
- Merge commit: `b0dfa877e214302602d79f2d87de29f5ff2dda25`
- Harness CLI release: `harness-cli-v0.1.12`
- Changed files:
  - `crates/harness-cli/src/application.rs`
  - `crates/harness-cli/src/domain.rs`
  - `crates/harness-cli/src/infrastructure.rs`
  - `crates/harness-cli/src/interface.rs`
  - `docs/CONTEXT_RULES.md`
  - `docs/FEATURE_INTAKE.md`
  - `docs/IMPROVEMENT_PROTOCOL.md`
  - `docs/decisions/0007-story-signal-mining.md`
  - `docs/stories/US-027-implementation-note-signals/design.md`
  - `docs/stories/US-027-implementation-note-signals/execplan.md`
  - `docs/stories/US-027-implementation-note-signals/implementation-notes.html`
  - `docs/stories/US-027-implementation-note-signals/overview.md`
  - `docs/stories/US-027-implementation-note-signals/validation.md`
  - `docs/templates/implementation-notes.html`
  - `scripts/schema/006-story-signal.sql`

## 2026-06-16 - PR #4

- feat(harness): mine implementation-note signals in propose (US-027) (@thanh-dong)
- Merge commit: `b0dfa877e214302602d79f2d87de29f5ff2dda25`
- Harness CLI release: `harness-cli-v0.1.11`
- Changed files:
  - `crates/harness-cli/src/application.rs`
  - `crates/harness-cli/src/domain.rs`
  - `crates/harness-cli/src/infrastructure.rs`
  - `crates/harness-cli/src/interface.rs`
  - `docs/CONTEXT_RULES.md`
  - `docs/FEATURE_INTAKE.md`
  - `docs/IMPROVEMENT_PROTOCOL.md`
  - `docs/decisions/0007-story-signal-mining.md`
  - `docs/stories/US-027-implementation-note-signals/design.md`
  - `docs/stories/US-027-implementation-note-signals/execplan.md`
  - `docs/stories/US-027-implementation-note-signals/implementation-notes.html`
  - `docs/stories/US-027-implementation-note-signals/overview.md`
  - `docs/stories/US-027-implementation-note-signals/validation.md`
  - `docs/templates/implementation-notes.html`
  - `scripts/schema/006-story-signal.sql`

## 2026-06-13 - PR #3

- docs(harness): impact-analysis plugin on the inbound tool registry (US-026) (@thanh-dong)
- Merge commit: `fe27cbe697cb17bd7fc9806956ffac3a8e458350`
- Harness CLI release: not required
- Changed files:
  - `docs/FEATURE_INTAKE.md`
  - `docs/IMPACT_ANALYSIS.md`
  - `docs/stories/epics/E04-impact-analysis/US-026-blast-radius-plugin.md`
  - `docs/templates/story.md`

## 2026-06-13 - PR #2

- feat(cli): kind-aware inbound tool registry with presence scanning (@thanh-dong)
- Merge commit: `ba27b7c1204612f718c59b8e00624c0db7d73d8a`
- Harness CLI release: `harness-cli-v0.1.10`
- Changed files:
  - `AGENTS.md`
  - `README.md`
  - `crates/harness-cli/src/application.rs`
  - `crates/harness-cli/src/domain.rs`
  - `crates/harness-cli/src/infrastructure.rs`
  - `crates/harness-cli/src/interface.rs`
  - `docs/TOOL_REGISTRY.md`
  - `docs/stories/US-027-inbound-tool-registry.md`
  - `scripts/install-harness.sh`
  - `scripts/schema/005-tool-extensions.sql`

## 2026-06-09 - PR #13

- docs(phase5): Phase 5 — Evolution Infrastructure scope (@hoangnb24)
- Merge commit: `bfef94a77acfa33af81f6da96bc06f053d7f5164`
- Harness CLI release: `harness-cli-v0.1.9`
- Changed files:
  - `PHASE5.md`
  - `crates/harness-cli/src/application.rs`
  - `crates/harness-cli/src/domain.rs`
  - `crates/harness-cli/src/infrastructure.rs`
  - `crates/harness-cli/src/interface.rs`
  - `docs/FEATURE_INTAKE.md`
  - `docs/GLOSSARY.md`
  - `docs/HARNESS.md`
  - `docs/HARNESS_AUDIT.md`
  - `docs/HARNESS_COMPONENTS.md`
  - `docs/HARNESS_MATURITY.md`
  - `docs/IMPROVEMENT_PROTOCOL.md`
  - `docs/TOOL_REGISTRY.md`
  - `docs/decisions/0007-improvement-proposal-rules.md`
  - `docs/stories/US-019-machine-readable-tool-registry.md`
  - `docs/stories/US-020-batch-story-verification.md`
  - `docs/stories/US-021-intervention-recording-schema.md`
  - `docs/stories/US-022-context-rule-measurement.md`
  - `docs/stories/US-023-drift-detection-entropy-score.md`
  - `docs/stories/US-024-improvement-proposal-pipeline.md`
  - `docs/stories/epics/E03-phase-5-evolution-infrastructure/phase-5-progress.md`
  - `scripts/install-harness.sh`
  - `scripts/schema/003-tool-registry.sql`
  - `scripts/schema/004-intervention.sql`

## 2026-06-09 - Post-Merge Automation

- Added post-merge changelog automation for merged pull requests.
- Added conditional Harness CLI patch release automation when merged PRs change Rust CLI source, schema, Cargo metadata, or release packaging files.
- Reused the existing Harness CLI release workflow for release builds so tag, manual, and post-merge releases share the same verification and asset publishing path.
