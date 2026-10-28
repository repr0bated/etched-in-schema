# Etched in Schema — Web3 index

An append-only table of contents for everything a 3tched identity (the
hackathon "twin") creates in the Web3 space. Each entry links an on-chain or
content-addressed artifact back to the identity that made it and the 3tched
operation receipt that authorised and recorded it.

- Schema: `schema/index-entry.schema.json` (`etched.index-entry/1`).
- Entries are hash-chained (`prev_hash`) per identity.
- The Merkle root over entries is anchored on-chain (default: Base Sepolia,
  EAS attestation), so any entry can be verified without trusting the agent
  or the operator.

Kept outside the op-dbus repo on purpose: op-dbus allows no parallel schema
sources. If this works out it becomes a sealed 3tched plugin; entries carry
over unchanged because each already references its 3tched receipt.
