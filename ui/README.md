# ui

The web UI is built **from scratch** for the hackathon (no code shared with the
3tched.com app). Only these references are kept, in `reference/`:

| File | What it is |
|---|---|
| `reference/element-map.json` | Design-free map of the 46 screens of the retired operation dashboard: headings, fields (type/label/placeholder), tabs, table columns and visible text. No styling, class names or layout. |
| `reference/menu.json` | The dashboard's menu: id, label, section, route. |
| `reference/docs/json-render-core.README.md` | `@json-render/core` 0.20.0 — specs, catalogs, validation (+ LICENSE). |
| `reference/docs/json-render-svelte.README.md` | `@json-render/svelte` 0.20.0 — the Svelte renderer that turns generated specs into UI (+ LICENSE). |

Stack (versions in use on the platform): Svelte 5.57, SvelteKit 3, Vite 8,
`@json-render/core` + `@json-render/svelte` 0.20. Svelte, SvelteKit and Vite
documentation: the skills listed in `AGENTS.md` (`svelte`, `svelte-code-writer`,
`sveltekit`, `json-render`, `core`, `devtools-svelte`) and the official docs at
svelte.dev and vite.dev.

Rules: Svelte + json-render only; every visible element is declared in the
catalog; the element map is a content reference, not a design to copy.
