//! Compliance catalog: control tags → OSCAL controls → candidate operations.
//!
//! Table is `oscal/TAG-TAXONOMY.md` as code; `oscal/catalogs/etched-catalog`
//! is authoritative. Qualifier tags (`action:*`, `approval:*`, `surface:*`,
//! `data:*`, `model:*`) never match controls — they steer the decision.

use crate::model::Operation;

/// One taxonomy row: tag, control, candidate `plugin/method` operations.
/// A `*` method means any method of the plugin.
const TAXONOMY: &[(&str, &str, &[&str])] = &[
    ("pii", "etched-pt-1", &["schema_tag_registry/tag-registry.get-policy"]),
    (
        "access-control",
        "etched-ac-1",
        &["elevation/authorize_task", "schema_tag_registry/*"],
    ),
    ("audit", "etched-au-1", &["identity_sled/*", "routing_decision/*"]),
    ("retention", "etched-si-1", &["cron/schedule", "workflows/*"]),
    ("integrity", "etched-in-1", &["identity_sled/*", "web3/*"]),
];

/// A control the request matched, with its candidate operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlMatch {
    pub control: String,
    pub operations: Vec<Operation>,
}


fn split_operation(spec: &str) -> Operation {
    match spec.split_once('/') {
        Some((plugin, method)) => Operation {
            plugin: plugin.to_string(),
            method: method.to_string(),
        },
        None => Operation {
            plugin: spec.to_string(),
            method: "*".to_string(),
        },
    }
}

/// Resolve request tags to controls, keeping taxonomy order and dropping
/// controls outside the profile. Unknown control tags match nothing —
/// a tag miss is a refusal downstream, never a default-allow.
pub fn match_controls(tags: &[String], profile_controls: &[String]) -> Vec<ControlMatch> {
    let mut out = Vec::new();
    for (tag, control, ops) in TAXONOMY {
        if !tags.iter().any(|t| t == tag) {
            continue;
        }
        if !profile_controls.iter().any(|c| c == control) {
            continue;
        }
        if out.iter().any(|m: &ControlMatch| m.control == *control) {
            continue;
        }
        out.push(ControlMatch {
            control: control.to_string(),
            operations: ops.iter().map(|s| split_operation(s)).collect(),
        });
    }
    out
}

/// True when the request carries the two-person-rule marker.
pub fn is_root(tags: &[String]) -> bool {
    tags.iter().any(|t| t == "approval:root")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> Vec<String> {
        vec![
            "etched-pt-1".into(),
            "etched-ac-1".into(),
            "etched-au-1".into(),
            "etched-si-1".into(),
            "etched-in-1".into(),
        ]
    }

    #[test]
    fn each_taxonomy_tag_resolves() {
        let cases = [
            ("pii", "etched-pt-1", 1),
            ("audit", "etched-au-1", 2),
            ("access-control", "etched-ac-1", 2),
            ("retention", "etched-si-1", 2),
            ("integrity", "etched-in-1", 2),
        ];
        for (tag, control, op_count) in cases {
            let m = match_controls(&[tag.into()], &profile());
            assert_eq!(m.len(), 1, "tag {tag}");
            assert_eq!(m[0].control, control);
            assert_eq!(m[0].operations.len(), op_count, "tag {tag}");
        }
    }

    #[test]
    fn qualifiers_never_match_and_miss_is_empty() {
        let tags: Vec<String> = vec![
            "action:publish".into(),
            "surface:web".into(),
            "bogus".into(),
        ];
        assert!(match_controls(&tags, &profile()).is_empty());
    }

    #[test]
    fn profile_filters_controls() {
        let m = match_controls(&["pii".into(), "audit".into()], &["etched-au-1".into()]);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].control, "etched-au-1");
    }

    #[test]
    fn duplicate_tags_match_once() {
        let m = match_controls(&["audit".into(), "audit".into()], &profile());
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn root_marker_detected() {
        assert!(is_root(&["approval:root".into()]));
        assert!(!is_root(&["action:write".into()]));
    }
}
