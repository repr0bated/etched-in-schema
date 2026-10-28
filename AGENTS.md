# AGENTS.md — etched-in-schema (read by omp)

Start with `START-HERE.md`, then read `CONTEXT.md` in full (it bundles every doc).

## Mandatory skill preload

At the start of every session, before any work, read these files completely
(base: `/srv/git/odbus-verified/.agents/skills/<name>/SKILL.md`). A sub-agent must
repeat the preload for its own session.

- **Always:** `verification-before-completion`, `agent-self-evaluation`, `quality-auditor`, `application-security`, `requesting-code-review`, `receiving-code-review`
- **UI (Svelte + json-render):** `svelte`, `svelte-code-writer`, `sveltekit`, `json-render`, `core`, `devtools-svelte`, `web-design-guidelines`, `accessibility`, `typescript-patterns`, `zod-validation`
- **Rust (router, policy, enforcement, approvals):** `rust`, `rust-patterns`, `rust-testing`, `rust-memory-optimization`, `memory-safety-patterns`
- **Compliance (OSCAL):** `trestle-workspace`, `trestle-oscal-models`, `trestle-authoring-workflow`, `trestle-control-implementation`, `trestle-validation`, `trestle-governance`, `trestle-assessment`, `trestle-compliance-pipeline`, `trestle-poam`, `trestle-task-system`, `compliance-check`
- **Judgments (Jev):** `typesafe-ai`
- **Platform (3tched, used through interfaces):** `grpc-expert`, `grpc-protocol-expert`, `incus`, `btrfs`

UI references (from-scratch UI): `ui/README.md`, `ui/reference/`.

This repository is PUBLIC: placeholders only, push often, never rewrite history, never use `--no-verify`. Enable the guard once per clone: `git config core.hooksPath .githooks`.
Runtime and rules: `docs/RUNTIME.md`; specs: `docs/DROP-IN-SPEC.md`, `docs/APPROVALS-SPEC.md`.


Read `SPARK-BRIEF.md` first: it is the current project state, paths, rules and plan.
Then `README.md`, `docs/3TCHED-RESOURCES.md`, `docs/SUBSIDIARY-SLED.md`, `docs/PRIOR-ART.md`, `docs/WINNING.md` and `docs/DROP-IN-SPEC.md` (the drop-in to build).
For any backend work, also read `/srv/git/odbus-verified/AGENTS.md` and its `HANDOFF.md`.

- Schemas here are drafts. They become sealed 3tched plugins (Rust) later; do not copy them into op-dbus as files.
- Keep claims honest: mark what is proven vs planned.
- Record decisions and progress at the end of `SPARK-BRIEF.md` (dated), so Claude, Codex, Grok and Spark share one state.
