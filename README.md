# Etched in Schema

**An identity that acts — and can prove what it did.**

Etched in Schema is a self-contained 3tched identity — the living embodiment
of a business subsidiary — that turns a business goal into deployed,
working infrastructure. Every action it takes is authorised against its
capabilities, bounded by compliance rules, recorded as a tamper-evident
receipt, and anchored on a blockchain so anyone can verify *who did what,
under what authority, and what actually happened* without trusting the agent
or its operator.

Built for the Meta Model API hackathon (Indie Dev Track) by Jeremy Hobson,
founder of 3tched.

## Why

AI agents can reason about goals, but they rarely have a durable identity,
verifiable permissions, accountable execution, or the ability to build and
operate real systems. On-chain artifacts have the mirror-image problem: a
contract address or a content hash proves something exists, not that it came
from an accountable actor following permitted steps. That missing link
between artifacts and accountable, authorised actors is what keeps businesses
from trusting both autonomous agents and Web3.

Etched in Schema closes that gap from both sides: every on-chain artifact
points to the identity and authorised operation behind it, and every
operation points to the artifacts it produced, with the history anchored
on-chain.

## Goals

1. **Embodiment** — the identity presents itself (avatar, first person),
   explains what it is, and answers questions grounded in its own record.
2. **Compliance by construction** — a business declaration becomes an OSCAL
   profile; OSCAL tags route each requirement to the plugins that implement
   it; boundary rules compile into capability grants and OpenFlow network
   policy that are enforced, not just documented.
3. **"Create a CMS for this business"** — the identity provisions a usable CMS
   whose controls (PII redaction, audit, access control, retention,
   integrity) are visible on stage and evidenced by receipts.
4. **Verifiable on-chain** — a Web3 index of everything the identity creates,
   each entry linked to its receipt, with the index root anchored on-chain.

**Golden path** (must work three times in a row before any stretch goal):
scripted intro → "create a CMS for this business" from one small declaration
and one OSCAL profile → the identity provisions it through its own mount,
with a visible trace and receipts → one receipt anchored on-chain → a judge
verifies it themselves.

Out of scope: moving identities between hosts. Optional: a live
client-acquisition attempt, only if it fits the flow.

## What is new vs. what already exists

Everything in **this repository** is developed during the hackathon. It is
built *on* 3tched, which already exists and is used only through its
published interfaces (see [docs/3TCHED-RESOURCES.md](docs/3TCHED-RESOURCES.md)).

| New in this repo | Existing 3tched resource it uses |
|---|---|
| OSCAL catalog of 3tched capabilities, tag taxonomy, profiles, component definitions | `oscal_subid_registry` (subid discovery), `schema_tag_registry` (tag metadata, owner-scoped policy) |
| Tag router: profile → components → plugin operations | sealed plugin catalog, identity operation files (3tchedFS) |
| Boundary compiler: OSCAL rules → capability grants, budgets, OpenFlow policy | bridge capability enforcement, `openflow` / `ovsdb_bridge` plugins, router budgets |
| CMS with evidenced controls (redaction, audit, roles, retention, integrity) | identity container, receipts, Qdrant/Cozo |
| Business declarations (starting with a Florida LLC) | — |
| Web3 index + on-chain anchoring | chain events / operation receipts |
| Avatar, trace and OSCAL views (Svelte + json-render) | 3tched.com UI, chat path |

## Architecture at a glance

```
          internet only                      local filesystem only
  Meta model API  ◄──┐                 ┌──► 3tchedFS mount (lungs) ──► 3tched bridge
  blockchain      ◄──┤   IDENTITY      │      kernel-peer identity, capability check,
                     └── CONTAINER  ───┘      MutationEngine commit, receipt
                          (the twin)
                               │
            OSCAL profile ─► tag router ─► plugin operations (incus, openflow, …)
                               │
                  receipts ─► Web3 index ─► Merkle root anchored on-chain
```

- The identity has **no network path to 3tched**; it reaches 3tched only
  through its own FUSE mount, where every operation is a typed file.
- Its only network is the internet: the Meta model API and the blockchain.
- Subsidiaries map to Linux users; the root is the 3tched control plane.
  See [docs/SUBSIDIARY-SLED.md](docs/SUBSIDIARY-SLED.md).
- How this compares to SPIFFE, BeyondCorp and DIDs, and what it adds:
  [docs/PRIOR-ART.md](docs/PRIOR-ART.md).

## Repository layout

| Path | Contents |
|---|---|
| `schema/` | JSON Schemas: Web3 index entry, business declarations |
| `oscal/` | OSCAL catalog, profiles, component definitions (Compliance Trestle workspace) |
| `routing/` | tag router: profile → components → operations |
| `enforcement/` | boundary compiler: OSCAL rules → grants, budgets, OpenFlow policy |
| `cms/` | the CMS and its control behaviours |
| `web3/` | index log, Merkle anchoring |
| `ui/` | Svelte components for the 3tched.com app |
| `docs/` | 3tched resources, subsidiary/sled model, prior art and positioning, how to win |
| `SPARK-BRIEF.md` | current state, rules, open issues, progress log |

## Working rules

- Read `SPARK-BRIEF.md` before starting; log progress at its end (dated).
- Mark claims as **proven** (demonstrated, with evidence) or **planned**.
- Schemas here are drafts; promoted ones become sealed 3tched plugins (Rust).
- UI: Svelte/SvelteKit + json-render's Svelte renderer only.
- In pitches say "implements and evidences these controls", never
  "compliant" or "certified".
