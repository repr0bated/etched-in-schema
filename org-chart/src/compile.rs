//! The compiler: chart facts in, effective capabilities out.
//!
//! Rules: explicit roles only (no inheritance); unknown roles refuse the
//! position; missing attributes skip with reason; reporting cycles and
//! depth beyond [`MAX_DEPTH`] refuse the whole chart.

use crate::model::OrgChart;
use std::collections::{BTreeMap, BTreeSet};

/// Maximum reporting depth. Deeper charts refuse as malformed.
pub const MAX_DEPTH: usize = 32;

/// One derived grant, with provenance for the "why do I hold this" view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub member_key: String,
    pub capability: String,
    pub via_role: String,
    pub via_position: String,
}

/// A grant withheld, with its reason. Never silent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skip {
    pub member_key: String,
    pub capability: String,
    pub via_role: String,
    pub via_position: String,
    pub reason: String,
}

/// Compiler output: grants plus skips. Empty chart compiles to empty.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompileOutput {
    pub grants: Vec<Grant>,
    pub skips: Vec<Skip>,
}

/// Whole-chart refusal: the chart itself is malformed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartError {
    UnknownRole { position: String, role: String },
    Cycle { position: String },
    TooDeep { position: String },
    DuplicatePosition { position: String },
}

fn check_structure(chart: &OrgChart) -> Result<(), ChartError> {
    let mut seen = BTreeSet::new();
    for p in &chart.positions {
        if !seen.insert(p.id.clone()) {
            return Err(ChartError::DuplicatePosition {
                position: p.id.clone(),
            });
        }
    }
    let ids: BTreeSet<&str> = chart.positions.iter().map(|p| p.id.as_str()).collect();
    for p in &chart.positions {
        for role in &p.roles {
            if chart.role(role).is_none() {
                return Err(ChartError::UnknownRole {
                    position: p.id.clone(),
                    role: role.clone(),
                });
            }
        }
        // Walk the reporting chain: missing parents are roots; cycles and
        // over-depth chains refuse.
        let mut chain = BTreeSet::new();
        chain.insert(p.id.as_str());
        let mut current = p.reports_to.as_deref();
        let mut depth = 1;
        while let Some(id) = current {
            if !chain.insert(id) {
                return Err(ChartError::Cycle {
                    position: p.id.clone(),
                });
            }
            depth += 1;
            if depth > MAX_DEPTH {
                return Err(ChartError::TooDeep {
                    position: p.id.clone(),
                });
            }
            current = chart
                .positions
                .iter()
                .find(|q| q.id == id)
                .and_then(|q| q.reports_to.as_deref());
            if !ids.contains(id) {
                break;
            }
        }
    }
    Ok(())
}

/// Compile `(chart, ())` to effective grants. The profile argument is
/// reserved: role capabilities already name sealed caps, so no profile
/// filtering happens here — controls govern use at `decide()` time.
pub fn compile(chart: &OrgChart) -> Result<CompileOutput, ChartError> {
    check_structure(chart)?;
    let mut out = CompileOutput::default();
    let mut granted: BTreeMap<(String, String), (String, String)> = BTreeMap::new();
    for position in &chart.positions {
        for role_id in &position.roles {
            let role = chart.role(role_id).expect("checked above");
            for member in &position.members {
                let missing: Vec<&String> = role
                    .requires_attributes
                    .iter()
                    .filter(|a| !member.attributes.contains(a))
                    .collect();
                for cap in &role.capabilities {
                    if missing.is_empty() {
                        // First writer wins deterministically (position order);
                        // provenance records where it came from.
                        granted
                            .entry((member.key.clone(), cap.clone()))
                            .or_insert((role.id.clone(), position.id.clone()));
                    } else {
                        out.skips.push(Skip {
                            member_key: member.key.clone(),
                            capability: cap.clone(),
                            via_role: role.id.clone(),
                            via_position: position.id.clone(),
                            reason: format!(
                                "missing attributes: {}",
                                missing
                                    .iter()
                                    .map(|s| s.as_str())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            ),
                        });
                    }
                }
            }
        }
    }
    for ((member_key, capability), (via_role, via_position)) in granted {
        out.grants.push(Grant {
            member_key,
            capability,
            via_role,
            via_position,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Member, OrgChart, Position, Role};

    fn member(key: &str, attrs: &[&str]) -> Member {
        Member {
            key: key.into(),
            attributes: attrs.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn role(id: &str, caps: &[&str], attrs: &[&str]) -> Role {
        Role {
            id: id.into(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            requires_attributes: attrs.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn empty_chart_compiles_empty() {
        let out = compile(&OrgChart {
            id: "empty".into(),
            positions: vec![],
            roles: vec![],
        })
        .unwrap();
        assert!(out.grants.is_empty() && out.skips.is_empty());
    }

    #[test]
    fn unknown_role_refuses_position() {
        let chart = OrgChart {
            id: "bad".into(),
            positions: vec![Position {
                id: "p".into(),
                reports_to: None,
                roles: vec!["ghost".into()],
                members: vec![member("k", &[])],
            }],
            roles: vec![],
        };
        assert_eq!(
            compile(&chart),
            Err(ChartError::UnknownRole {
                position: "p".into(),
                role: "ghost".into(),
            })
        );
    }

    #[test]
    fn cycle_refuses_chart() {
        let mk = |id: &str, parent: &str| Position {
            id: id.into(),
            reports_to: Some(parent.into()),
            roles: vec![],
            members: vec![],
        };
        let chart = OrgChart {
            id: "cycle".into(),
            positions: vec![mk("a", "b"), mk("b", "a")],
            roles: vec![],
        };
        assert!(matches!(compile(&chart), Err(ChartError::Cycle { .. })));
    }

    #[test]
    fn missing_attributes_skip_with_reason() {
        let chart = OrgChart {
            id: "gate".into(),
            positions: vec![Position {
                id: "p".into(),
                reports_to: None,
                roles: vec!["r".into()],
                members: vec![member("k", &[])],
            }],
            roles: vec![role("r", &["cap.x"], &["compartment-a"])],
        };
        let out = compile(&chart).unwrap();
        assert!(out.grants.is_empty());
        assert_eq!(out.skips.len(), 1);
        assert!(out.skips[0].reason.contains("compartment-a"));
    }

    #[test]
    fn duplicate_position_refuses() {
        let mk = || Position {
            id: "p".into(),
            reports_to: None,
            roles: vec![],
            members: vec![],
        };
        let chart = OrgChart {
            id: "dup".into(),
            positions: vec![mk(), mk()],
            roles: vec![],
        };
        assert!(matches!(
            compile(&chart),
            Err(ChartError::DuplicatePosition { .. })
        ));
    }
}
