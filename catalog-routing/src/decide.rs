//! The decision: allow / refuse / needs-approval.
//!
//! Order (`docs/DROP-IN-SPEC.md` §2.2, `routing/DATA-MODEL.md`): match
//! controls → root needs two humans → capability check → allow. A control
//! miss refuses before grants are consulted; grants never come from tags.

use crate::catalogs::compliance;
use crate::model::{Decision, Grants, Profile, Request, Verdict};

/// Pure decision over a request, profile controls, and compiled grants.
pub fn decide(request: &Request, profile: &Profile, grants: &Grants) -> Decision {
    let matches = compliance::match_controls(&request.tags, &profile.controls);
    if matches.is_empty() {
        return Decision {
            operations: vec![],
            verdict: Verdict::Refuse {
                missing_capability: None,
                rule: "no-control-match".to_string(),
            },
        };
    }
    let operations: Vec<_> = matches
        .iter()
        .flat_map(|m| m.operations.clone())
        .collect();

    if compliance::is_root(&request.tags) {
        return Decision {
            operations,
            verdict: Verdict::NeedsApproval {
                have: 0,
                need: 2,
                rule: "two-person-root".to_string(),
            },
        };
    }

    for op in &operations {
        let capability = op.required_capability();
        if !grants.covers(&capability) {
            let rule = matches
                .iter()
                .find(|m| {
                    m.operations
                        .iter()
                        .any(|o| o.required_capability() == capability)
                })
                .map(|m| m.control.clone())
                .unwrap_or_default();
            return Decision {
                operations: vec![],
                verdict: Verdict::Refuse {
                    missing_capability: Some(capability),
                    rule,
                },
            };
        }
    }

    Decision {
        operations,
        verdict: Verdict::Allow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ActionKind, Surface};

    fn profile() -> Profile {
        Profile {
            id: "small-business-portal".into(),
            controls: vec![
                "etched-pt-1".into(),
                "etched-ac-1".into(),
                "etched-au-1".into(),
                "etched-si-1".into(),
                "etched-in-1".into(),
            ],
        }
    }

    fn request(tags: &[&str]) -> Request {
        Request {
            actor: "principal-1".into(),
            surface: Surface::Web,
            action: ActionKind::Read,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            payload_ref: "payload-1".into(),
        }
    }

    fn full_grants() -> Grants {
        Grants {
            capabilities: vec![
                "schema_tag_registry.tag-registry.get-policy".into(),
                "schema_tag_registry.*".into(),
                "elevation.authorize_task".into(),
                "identity_sled.*".into(),
                "routing_decision.*".into(),
                "cron.schedule".into(),
                "workflows.*".into(),
                "web3.*".into(),
            ],
        }
    }

    #[test]
    fn allow_when_controls_match_and_grants_cover() {
        let d = decide(&request(&["audit"]), &profile(), &full_grants());
        assert_eq!(d.verdict, Verdict::Allow);
        assert_eq!(d.operations.len(), 2);
    }

    #[test]
    fn refuse_on_control_miss_before_grants() {
        let d = decide(&request(&["bogus"]), &profile(), &full_grants());
        assert_eq!(
            d.verdict,
            Verdict::Refuse {
                missing_capability: None,
                rule: "no-control-match".into(),
            }
        );
        assert!(d.operations.is_empty());
    }

    #[test]
    fn refuse_names_missing_capability_and_rule() {
        let grants = Grants {
            capabilities: vec!["workflows.*".into()],
        };
        let d = decide(&request(&["retention"]), &profile(), &grants);
        assert_eq!(
            d.verdict,
            Verdict::Refuse {
                missing_capability: Some("cron.schedule".into()),
                rule: "etched-si-1".into(),
            }
        );
    }

    #[test]
    fn root_needs_two_approvals() {
        let d = decide(
            &request(&["integrity", "approval:root"]),
            &profile(),
            &Grants {
                capabilities: vec![],
            },
        );
        assert_eq!(
            d.verdict,
            Verdict::NeedsApproval {
                have: 0,
                need: 2,
                rule: "two-person-root".into(),
            }
        );
    }
}
