//! Spec generator: trim rows to a renderable grants-console spec.
//!
//! The console is a generated spec, not a hand-built page: every element is
//! a catalog component, validated before rendering, actions call granted
//! methods only. The propose form is illustrative (`dashboardFormPreview`
//! submits nothing) — proposals travel the approval path, never the UI.

use serde_json::{json, Value};

/// One grant row: the unit the console table renders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantRow {
    pub principal: &'static str,
    pub capability: &'static str,
    pub effect: &'static str,
    pub decision: &'static str,
}

/// Components this generator may emit. Mirrors the sealed catalog names
/// (`dashboard-elements.ts` + `chat-catalog.ts`); the sealed blob stays
/// authoritative — unknown names fail validation before render.
pub const ALLOWED_COMPONENTS: &[&str] = &[
    "container",
    "pageHeader",
    "dashboardStatus",
    "dashboardTable",
    "dashboardFormPreview",
];

/// Build the flat-spec console for `rows`.
pub fn console_spec(title: &str, subtitle: &str, rows: &[GrantRow]) -> Value {
    let table_rows: Vec<Value> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            json!({
                "id": format!("grant-{i}"),
                "cells": [r.capability, r.effect, r.principal, r.decision],
            })
        })
        .collect();
    json!({
        "root": "root",
        "elements": {
            "root": {
                "type": "container",
                "props": { "className": null },
                "children": ["header", "reference", "grants-table", "propose"]
            },
            "header": {
                "type": "pageHeader",
                "props": { "title": title, "subtitle": subtitle }
            },
            "reference": {
                "type": "dashboardStatus",
                "props": {
                    "label": "Draft console",
                    "detail": "Illustrative data. Proposals travel the approval path; this form submits nothing.",
                    "tone": "info"
                }
            },
            "grants-table": {
                "type": "dashboardTable",
                "props": {
                    "caption": "Capability trim",
                    "columns": [
                        { "id": "capability", "label": "Capability" },
                        { "id": "effect", "label": "Effect" },
                        { "id": "principal", "label": "Principal" },
                        { "id": "decision", "label": "Decision" }
                    ],
                    "rows": table_rows
                }
            },
            "propose": {
                "type": "dashboardFormPreview",
                "props": {
                    "title": "Propose grant change",
                    "fields": [
                        { "id": "principal", "label": "Principal", "type": "text", "value": "", "help": "Reference control; no request is sent." },
                        { "id": "capability", "label": "Capability", "type": "text", "value": "", "help": "Reference control; no request is sent." },
                        { "id": "action", "label": "Action", "type": "select", "options": ["grant", "revoke", "delegate"], "help": "Reference control; no request is sent." },
                        { "id": "reason", "label": "Reason", "type": "textarea", "value": "", "help": "Reference control; no request is sent." }
                    ]
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<GrantRow> {
        vec![
            GrantRow {
                principal: "control-plane-chatbot",
                capability: "cap.identity.sled.self.read@v1",
                effect: "obs",
                decision: "keep",
            },
            GrantRow {
                principal: "control-plane-chatbot",
                capability: "cron.invoke",
                effect: "mut",
                decision: "delegate",
            },
        ]
    }

    #[test]
    fn spec_uses_only_catalog_components() {
        let spec = console_spec("T", "S", &rows());
        let elements = spec["elements"].as_object().unwrap();
        for (id, el) in elements {
            let t = el["type"].as_str().unwrap();
            assert!(ALLOWED_COMPONENTS.contains(&t), "{id} uses {t}");
            assert!(el.get("props").is_some(), "{id} missing props");
        }
    }

    #[test]
    fn table_cells_match_columns() {
        let spec = console_spec("T", "S", &rows());
        let table = &spec["elements"]["grants-table"]["props"];
        let n_cols = table["columns"].as_array().unwrap().len();
        for row in table["rows"].as_array().unwrap() {
            assert_eq!(row["cells"].as_array().unwrap().len(), n_cols);
        }
        assert_eq!(table["rows"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn empty_rows_still_valid_shape() {
        let spec = console_spec("T", "S", &[]);
        assert_eq!(spec["root"], "root");
        assert!(spec["elements"]["grants-table"]["props"]["rows"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
