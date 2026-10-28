# Etched in Schema — brief for Muse Spark

Hackathon: Meta Model API, Indie Dev Track. Builder: Jeremy Hobson (3tched).
State as of 2026-10-28. Everything marked **proven** was demonstrated on the
host that day; everything else is planned.

## The project in one paragraph

A self-contained 3tched identity — the "twin", the living embodiment of a
3tched subsidiary — turns a business goal into deployed, verified
infrastructure. It acts only through its own typed filesystem (3tchedFS),
every action is authorised against its capabilities and leaves a
tamper-evident receipt, and a Web3 index anchored on-chain lets anyone verify
who did what, under what authority, without trusting the agent or operator.
That link between on-chain artifacts and accountable, authorised actors is the
gap blocking Web3 adoption.

## Proven today

- **Identity container**: `3tchedCtlChatbot` (Incus, pool `3tched`, profile
  `3tched-lungs`), identity session `b0c777e3-0955-cd8b-3d01-ca12998cae28`,
  service principal, persist volume, kept running by runit `chatbot-identity`.
- **First-person operation call**: inside the container, write the plain JSON
  arguments to `/run/mount/3tchedfs/operations/<plugin>/<method>/call` (open
  read-write, write, fsync, read back on the same handle). Example
  `identity_sled/get_own_identity` with `{}` returned the identity, operation
  `51c8f91c-c2f1-4688-a985-fb41684adfff`, chain events 10389/10390, outcome
  completed. The bridge identifies the caller from the kernel peer over the
  lungs Unix socket, checks the exact capability, commits via MutationEngine.
- **Infrastructure operations available to the identity**: `incus/*`
  (create/start/stop/snapshot/devices), `ovsdb_bridge/*` (bridges, ports,
  transact), plus ~70 plugins. Each call is checked against the identity's
  grants.
- **No network path to 3tched**: the twin talks to 3tched only through its FUSE
  mount. Its only network will be the internet (Meta API, blockchain).
- **Chat path**: UI (`http://10.0.0.2:8080`) → door `10.0.0.2:8090` →
  `chat_router.Chat` → zeroclaw router on `127.0.0.1:8084` (protocol
  `zeroclaw.v1`) → model. Every turn receipted.
- **Avatar**: `IdentityAvatar.svelte` speaks a fixed intro script (exact
  wording, no model) and answers spoken or typed questions via the chat path.

## Where things are

| Thing | Path |
|---|---|
| Backend (Rust workspace) | `/srv/git/odbus-verified` — read its `AGENTS.md` and `HANDOFF.md` first |
| UI (SvelteKit + json-render) | `/srv/git/3tched.com` (has uncommitted WIP — do not publish it as-is) |
| UI build actually live | scratch worktree of 3tched.com HEAD + `tched_router`→`chat_router` + avatar |
| This repo | `/srv/git/etched-in-schema` — Web3 index schema, LLC declaration schema |
| Recovered execution tracker | branch `recover/execution-tracker`, worktree `/srv/git/odbus-tracker-recover` (compiles; not deployed) |
| Router config | `/var/lib/tched-router/config.toml` (root; backups `*.bak-20261028-*`) |
| Qdrant | `127.0.0.1:6333` — `repos_specs_docs_voyage_4_lite`, `repos_repomix_voyage_context_4`, `snowball_footprints`, `blockchain_footprints` |

## Rules that must not be broken

- The plugin IS the schema. Backend schemas live only in Rust plugins under
  `crates/op-plugins/src/state_plugins/`, sealed into blobs. No standalone
  schema files in op-dbus (that is why drafts live in this repo).
- UI is Svelte/SvelteKit + Vite + json-render's Svelte renderer only. Every
  visible element is a catalog component.
- Never edit Cozo directly, never bypass a permission denial, never add or
  remove Btrfs members outside explicit identity provisioning.
- Deployment is `./activate-golden.sh` / `op-release-launch release`
  (btrfs send/receive slots). Never hand-copy binaries onto the host as a deploy.
- One Cargo process at a time; release builds need `CXXFLAGS="-include cstdint"`.
- Tests only when the owner asks.

## Plan (10 days)

1. OSCAL catalog of 3tched capabilities with a tag taxonomy; component
   definitions for 3–5 plugins (Compliance Trestle is installed: `trestle`).
2. Tag routing: profile → components → plugin operations.
3. Boundary rules compiled into capability grants and budgets (the bridge
   already enforces grants on every call).
4. "Create a CMS for this business": business declaration (start with
   `schema/llc-declaration.schema.json`) → profile → provision via the twin,
   receipts recorded as OSCAL assessment results and anchored on-chain.
5. UI: avatar intro, then "enough about me — here's what I can do for your
   business" (scripted once the business is chosen), vitals and nerve-trace
   views fed by real receipts, OSCAL views.
Golden path (must work three times in a row before any stretch goal):
scripted intro → "create a CMS for this business" from one small declaration
and one OSCAL profile → the twin provisions one or two real things through its
own mount, with a visible nerve trace and receipts → one receipt anchored
on-chain → a judge verifies it themselves.

Out of scope (owner, 2026-10-28): moving the identity between hosts
("astral projection") — no business case. Client acquisition is optional; add
it only if it fits the flow.

## Known open issues

- Meta model not yet wired into the router (no Meta API key on the host;
  Junie reaches Muse Spark through its JetBrains login). Hermes has a `Meta`
  custom provider stub (`HERMES_CUSTOM_META_API_KEY`).
- Router turn costs ~15k tokens (skills bundle removed to halve it). Qwen does
  not reliably emit text tool calls; MCP compact tools are connected
  (`https://10.0.0.2:8090/mcp`) but retrieval answers came back empty.
- Live state reads through the mount (`plugins/*/value.json`) return EIO.
- Next release will fail activation: `notebook-sources-sync` cannot pass its
  30 s check after restart. Fix before releasing.
- Re-key of the core chatbot to the WireGuard server key (session
  `58287215-0136-f7c3-0a02-d51a593b75ef`) has no supported op yet.
- UI overlay on `dashboard-release` is runtime-only; lost on reboot or release.

## CMS: compliant by construction (owner direction, 2026-10-28)

The CMS is the showcase because it comes out usable *and* evidently
controlled. Pick a handful of controls whose effect is visible on stage; each
control → a CMS behaviour → evidence (receipt or anchor):

| Control area | Visible behaviour | Evidence |
|---|---|---|
| PII handling (NIST 800-53 PT) | automatic redaction on publish; original kept only in the owner's private scope | redaction operation receipt naming the triggering policy |
| Audit (AU) | every create/edit/publish attributable | receipt chain: who, capability, when |
| Access control (AC) | roles compiled into capability grants | a live denial showing the exact missing capability |
| Retention (SI-12) | scheduled expiry/archive | scheduled operation receipt |
| Integrity | published versions hash-anchored | on-chain anchor a judge can check |

Pitch wording: "implements and evidences these controls" — never "compliant"
or "certified" (an assessor declares compliance).

## Remote identities: established design (owner, 2026-10-28)

WireGuard is only the establishment step: the peer key proves the identity
once, the decoy issues the signed peer proof (OIA1) and the session gets its
sealed credential (SID1). After that it is zero trust — every call carries the
credential and the door's interceptor re-verifies it, over any transport
(internet, Cloudflare). 3tched-fs `Credential` mode is the remote form of the
lungs. The mechanism exists; it needs testing and refinement, not a rebuild.
Do not add a mesh (Netmaker/Cloudflare) as an identity layer.
