# Dev cycle: blob → mount → identity

**The method is the blob.** In 3tched a plugin *is* its schema, sealed into a
blob; a method exists if and only if its blob is in the sealed catalog. The
identity (the chatbot / the subsidiary) never calls code directly: it sees the
sealed methods as typed files in its own 3tchedFS mount, calls them there, and
every call is admitted, checked and receipted by the platform. Every new
method built for the hackathon follows this path — that is the dev cycle.

```text
 this repo (new logic)        platform registration        sealed catalog
 ┌───────────────────┐       ┌────────────────────┐       ┌──────────────────────────┐
 │ crate: types,     │──────▶│ plugin schema +    │──────▶│ <plugin>.<hash16>.blob   │
 │ rules, tests      │       │ dispatch (op-dbus) │ seal  │ (/dev/shm/opdbus/...)    │
 └───────────────────┘       └────────────────────┘       └────────────┬─────────────┘
                                                                        │ projected by 3tchedFS
                                                                        ▼
 receipt ◀── bridge: kernel/credential identity, ◀── call file ◀── identity's mount
 (operation id,      exact capability, commit          (write args,   operations/<plugin>/<method>/call
  chain event)                                          read result)
```

## The three relationships

| Relationship | What it means | Where to see it |
|---|---|---|
| **blob ↔ method** | A method exists only as a sealed blob; its schema is the single source of its arguments, results and required capability | `ls /dev/shm/opdbus/plugin-blobs/` (72 blobs today); `opblob inspect <file.blob>` |
| **blob ↔ mount** | 3tchedFS projects each sealed method as `operations/<plugin>/<method>/{call,metadata.json,input.schema.json,output.schema.json}`; a new blob appears in every identity's mount, with no mount change | `/home/3tched/Hackathon/3tchedfs/operations/` (as `3tched`) |
| **mount ↔ chatbot** | The identity acts only through its mount: call files, or `3tched-fs mcp --mount <mount>` which serves the sealed methods as MCP tools dispatched as that identity; `3tched-fs toolsets --publish` announces them on EMQX | call-file helper `3tched-callfile`; `3tched-fs mcp --help` |

## Steps for a new method (e.g. `approvals.request_approval`)

1. **Logic, in this repo.** Types, rules and tests in a crate here (e.g.
   `approvals/`). No I/O. This is the judged, new work.
2. **Register as a plugin, on the platform.** A thin schema + dispatch module in
   `/srv/git/odbus-verified/crates/op-plugins/src/state_plugins/<name>.rs`,
   registered in `default_registry.rs`, calling this repo's crate. Follow that
   repo's `AGENTS.md` (plugin is the schema; no parallel schema files). Keep it
   an adapter: decisions stay in this repo's crate.
3. **Seal a development catalog, without touching live.** Build, then
   `opblob seal-plugins <dev-catalog-dir>` and inspect the new blob
   (`opblob inspect`). The live catalog is unchanged.
4. **Release.** The only deploy path is the platform release
   (`./activate-golden.sh` → slot activation). After it, the blob is in the live
   catalog and the operation file appears in the subsidiary's mount
   automatically.
5. **Grant.** The identity gets the method's exact capability (an owner-approved
   grant change). Without it, the call is refused with the capability named —
   which is itself demonstrable.
6. **Call and verify.** As the identity: write arguments to the call file (or
   use the MCP tool), read the result, keep the `operation_id` and
   `event_hash` as evidence.

## Mounts today (2026-10-28)

| Mount | Identity | Status |
|---|---|---|
| `/home/3tched/Hackathon/3tchedfs` (host) | Hackathon subsidiary `<subsidiary-session>` | mounted, runit `subsidiary-hackathon-3tchedfs` |
| `/run/mount/3tchedfs`, `/run/mount/threetched` (inside the identity container) | same identity | mounted |
| `/run/mounts/3tchedFS`, `/run/mounts/threetched` (host) | owner `<owner-session>` | **not mounted** — fstab entries missing since the reboot; owner's decision to restore |

Known limits: live state reads (`plugins/*/value.json`) return EIO — use
operations; the `3tched-fs call --mount` CLI cannot drive guest-kernel mounts —
use call files or `3tched-fs mcp`.
