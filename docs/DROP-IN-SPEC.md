# Drop-in spec: one generator, every surface

The drop-in is the existing 3tched UI generator, repurposed: the same spec
pipeline that renders the web UI also renders into consumer apps (Slack
first), and every action it exposes is governed by the new compliance
routing, policy and enforcement.

Marked **existing** (3tched today, used through its interfaces) or **new**
(built and judged in this repository).

## 1. What the generator does today (existing)

| Capability | Where it lives |
|---|---|
| **Catalog**: the components a UI may use, as sealed declarations (Canvas profile, whole-application contract, imported shadcn component schemas) | `json_render` plugin: `list_catalog`, `list_components`, `get_component_schema`, `get_catalog_entry` |
| **Spec**: a json-render element tree, validated against the catalog | `json_render`: `get_spec_schema`, `validate_spec`, `export_json_schema` |
| **Generation**: the model is given the catalog surface and writes a spec; the spec is validated before rendering | `json_render`: `build_prompt_surface`; chat application mode (`json_render_generation`) |
| **Deterministic rendering**: a UI built directly from a plugin's sealed schema, no model involved | `schema_renderer`: `render_schema`, `get_rendering`, `validate_schema`, `list_galleries` |
| **Gallery**: specs a human kept, saved as candidates and promoted into catalog entries | `json_render`: `save_candidate`, `list_candidates`, `promote_candidate`, `retire_catalog_entry` |
| **Actions and events**: elements bound to plugin operations; events handled and components updated | `json_render`: `list_actions`; `web_ui`: `render_page`, `handle_event`, `update_component` |
| **Renderers** | `json_render`: `list_renderers`, `list_packages`; web renderer: json-render's Svelte renderer in 3tched.com |

Rules that carry over unchanged: every visible element is declared in the
catalog; specs are validated before they are shown; the plugin is the schema.

## 2. What the drop-in adds (new)

### 2.1 Renderer targets

One spec, several targets. A target renders the catalog components it
supports and degrades the rest predictably (never drops an action or a
refusal silently).

| Target | Status | Notes |
|---|---|---|
| Web (Svelte) | existing | reference renderer |
| **Slack** (Block Kit) | new — first | messages, modals, interactive buttons |
| Teams (Adaptive Cards), email (HTML) | new — later | same mapping approach |

Component → Slack mapping (initial):

| Catalog component | Slack Block Kit |
|---|---|
| heading, text | `header`, `section` (mrkdwn) |
| key/value facts | `section` with `fields` |
| list, table (small) | `section` lines; large tables → attached file + summary |
| button / action | `actions` with `button` (value = action id) |
| select, input | modal with `static_select` / `plain_text_input` |
| status / progress | `context` line with state |
| **receipt** (new) | `context`: operation id, outcome, capability, event hash |
| **refusal** (new) | `section` with the exact missing capability and the rule that refused |
| **approval request** (new) | `actions` Approve / Reject, shows approvals so far (e.g. 1 of 2) |
| **redaction notice** (new) | `context`: what was redacted and which policy did it |

### 2.2 Governed actions

Every action, on every target, takes one path:

```
user action (web click / Slack button / Slack command)
  → consumer adapter (identifies the user; no authority of its own)
  → compliance router: request → OSCAL controls → candidate plugin operations
  → policy engine: allow / refuse (with reason) / needs approval (n of m)
  → enforcement: the operation runs through the identity's 3tchedFS call file;
    the bridge checks the exact capability and commits
  → result + receipt rendered back on the same target
```

- The adapter never calls plugins directly and never holds grants.
- A refusal is a first-class result, rendered with the missing capability and
  the rule.
- Approvals are collected on the target (Slack buttons) and counted by the
  policy engine; root actions need two different humans.
- Redaction is applied by policy before content reaches any target; the
  receipt records it.

### 2.3 New catalog components

`receipt`, `refusal`, `approval_request`, `redaction_notice`, `policy_decision`.
They are declared in the catalog like every other component (web and Slack
renderers both implement them).

### 2.4 Generation stays the same

The model (Meta's) still writes specs from the catalog surface; the spec is
validated before rendering on any target. A target-capability filter keeps the
model to components the destination supports.

## 3. Consumer adapters (new)

| Adapter | Inbound | Outbound | Identity of the user |
|---|---|---|---|
| Slack | slash command, app mention, button, modal submit (Socket Mode: no public inbound URL) | Block Kit messages in thread | Slack user mapped to a 3tched principal; unmapped users get read-only answers |

Adapters are thin: parse the event, look up the principal, hand the request to
the router, render the reply. All decisions happen in routing, policy and
enforcement, so every surface behaves identically.

## 4. Acceptance criteria (the working product)

1. A spec that renders on the web renders in Slack with the same actions.
2. From Slack, an allowed request runs through the identity and the thread
   shows the result and its receipt (operation id, outcome).
3. An out-of-bounds request is refused in Slack with the exact missing
   capability and the rule.
4. A root-level request shows an approval request and runs only after two
   different humans approve.
5. Content with PII posted through the drop-in is redacted before display,
   with a redaction notice and receipt.
6. The model (Meta) generates a spec from the catalog; invalid specs are
   rejected before rendering.

## 5. Out of scope

New UI frameworks (web stays Svelte + json-render), direct plugin calls from
adapters, adapters holding grants, a second policy engine per target.
