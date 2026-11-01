# Judge one-step verification (procedure draft — executes once an anchor exists)

Status: **procedure only.** No anchor exists yet (blocked: deployer key +
Sepolia RPC). When the first receipt is anchored, this is the single
check a judge runs. Everything below resolves through public reads;
nothing requires credentials.

## The one step

Given a receipt ID shown in the demo (an `operation_id`):

```sh
./tools/verify-anchor.sh <operation_id>
```

The script performs every check below and prints PASS/FAIL per line.
A pass means: *this exact agent action, under this exact authority,
produced this exact artifact, and its proof is on-chain.* Anything
else is a fail — no partial credit, no trust required.

## What the script checks

1. **Receipt resolves.** Fetch the `Outcome` for `operation_id` from the
   public receipt endpoint. Expect: `plugin`, `method`,
   `required_capability`, `event_hash`, `outcome: completed`.
   Fail if the receipt is absent or the outcome is not completed.

2. **Index entry binds the receipt.** Fetch the Web3 index entry whose
   `receipt.operation_id` equals the receipt. Expect: `seq`,
   `prev_hash`, `artifact.ref` (tx hash, contract address, or CID),
   `identity.session_id` + `genesis`, and the same `event_hash`.
   Recompute `sha256(canonical entry JSON)` and compare to the entry's
   own hash. Fail on any mismatch.

3. **Hash chain is intact.** Walk `prev_hash` links from the entry back
   to the anchored stream head. Recompute each link. Fail on first
   break. (Short circuits: the anchor record commits to
   `{seq_from, seq_to, stream_head_hash, merkle_root}` — verifying the
   head plus the Merkle proof for this entry is sufficient; full walk
   is the paranoid mode, enabled with `--full`.)

4. **Merkle proof verifies.** Recompute the leaf from the entry hash,
   verify the proof against the anchored `merkle_root`. Fail on
   mismatch.

5. **Root is on-chain.** Read the anchor (EAS attestation by default,
   Base Sepolia; UID recorded in the anchor record). Expect: the same
   `merkle_root`, `seq_from`/`seq_to` covering this entry's `seq`,
   and an attester address in the subsidiary's published attester set.
   Fail if the attestation is missing, revoked, or from an unknown
   attester.

6. **Authority matches.** The receipt's `required_capability` must be a
   capability the acting principal held at `event time` (checked
   against the published grant snapshot referenced by the anchor
   record). Tags alone never suffice — this check fails closed on
   tag-only authority. Fail otherwise.

## What the judge sees

```
receipt ............ FOUND (memory.create_namespace, completed)
index entry ........ BOUND (seq 41, hash ok)
chain .............. INTACT (41 links to head 9f3a…)
merkle proof ....... VALID against root b71c…
on-chain anchor .... EAS 0x… (Base Sepolia, attester ok)
authority .......... HELD (mut.data.memory.create-namespace@v1)
RESULT: PASS — don't trust us, check it.
```

## Negative controls (part of the demo)

- **Refusal:** request an ungranted operation live; the router refuses
  naming the exact missing capability, and the refusal receipt itself
  verifies through steps 1–2 (refusals are evidence, not errors).
- **Tamper:** flip one byte in a presented entry; step 2 fails loudly.
  This is the "wow, it actually checks" moment.

## Implementation notes for the builder

- `verify-anchor.sh` is a thin wrapper: `curl` the receipt endpoint,
  `curl` the index, `cast`/`cast call` (or the Cloudflare ETH gateway
  once its RPC route is repaired) for the attestation, and a ~60-line
  canonical-JSON + sha256 + Merkle checker in the repo's existing
  language (Rust crate `anchor-verify/`, pure, no I/O except args).
- Canonical JSON MUST byte-match the daemon's `canonicalize_json`
  (`op-state-store`), or step 2 fails spuriously. Test vectors come
  from real receipts, never fixtures.
- Until the Cloudflare gateway RPC route returns clean chain data,
  step 5 reads via the signer's RPC endpoint directly.
