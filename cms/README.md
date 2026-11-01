# cms

Drop-in CMS: usable *and* evidently controlled. Pitch wording: "implements
and evidences these controls" — never "compliant" or "certified".

Each control → CMS behaviour → evidence. Status is honest: **crate-proven**
means covered by `catalog-routing` unit tests in this repo; **stage-gated**
means proven only by a live mount read + receipt on rehearsal day.

| Control area | Visible behaviour | Evidence | Status |
|---|---|---|---|
| PII handling (PT) | automatic redaction on publish; original stays in owner's private scope | redaction receipt naming the triggering policy | crate-proven detector + masker (`redaction`); live receipt stage-gated |
| Audit (AU) | every create/edit/publish attributable | receipt chain: who, capability, when | crate-proven decision records; live chain stage-gated |
| Access control (AC) | roles compiled into capability grants | live denial naming the exact missing capability | crate-proven (`decide` refuse paths + route pins); live denial stage-gated |
| Retention (SI-12) | scheduled expiry/archive | scheduled operation receipt | planned — off the Wed run unless proven |
| Integrity | published versions hash-anchored | on-chain anchor a judge can check | planned — off the Wed run unless proven |

Routing in: end-user `WorkflowPurpose` (promoted tags, closed taxonomy
values) → `decide` allow / refuse / needs-approval → Spark `muse-spark-1.3`
spec → `validate_spec` vs sealed catalog → renderer draws only validated
refs. Publish path adds the deterministic screen first, JEV semantic
screen second, fail-closed both; publish stays off the run unless its
masking receipt proves out.
