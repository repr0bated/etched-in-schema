//! Request, grants, and decision types: `routing/DATA-MODEL.md` "Types".

use serde::{Deserialize, Serialize};

/// Renderer target. Web is the reference renderer; team chat is the first
/// drop-in target (`docs/DROP-IN-SPEC.md` §2.1). Teams and email are planned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Surface {
    Web,
    TeamChat,
    Teams,
    Email,
}

/// What the request does. Qualifier tags use the same words
/// (`action:read`, `action:write`, `action:publish`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionKind {
    Read,
    Write,
    Publish,
}

/// Adapter output. `tags` carries ≥1 control tag (`oscal/TAG-TAXONOMY.md`)
/// plus qualifier tags (`action:*`, `approval:root`, `surface:*`, `model:*`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub actor: String,
    pub surface: Surface,
    pub action: ActionKind,
    pub tags: Vec<String>,
    pub payload_ref: String,
}

/// Compiled profile: which controls apply. Grants (authority) arrive
/// separately — tags never grant (`oscal/TAG-TAXONOMY.md` rules).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub controls: Vec<String>,
}

/// The identity's compiled grants: exact capability strings
/// (`"<plugin>.<method>"`, or `"<plugin>.*"` for a whole plugin).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grants {
    pub capabilities: Vec<String>,
}

impl Grants {
    /// Exact match, or a `<plugin>.*` grant covering the capability.
    pub fn covers(&self, capability: &str) -> bool {
        if self.capabilities.iter().any(|c| c == capability) {
            return true;
        }
        match capability.split_once('.') {
            Some((plugin, _)) => self
                .capabilities
                .iter()
                .any(|c| c == &format!("{plugin}.*")),
            None => false,
        }
    }
}

/// A candidate plugin operation: `operations/<plugin>/<method>/call`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    pub plugin: String,
    pub method: String,
}

impl Operation {
    /// The exact capability enforcement checks.
    pub fn required_capability(&self) -> String {
        format!("{}.{}", self.plugin, self.method)
    }
}

/// The verdict. Refusals name the missing capability and the rule;
/// root actions need two different humans (mirrors `approvals/`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Allow,
    Refuse {
        missing_capability: Option<String>,
        rule: String,
    },
    NeedsApproval { have: u32, need: u32, rule: String },
}

/// The decision: candidate operations plus the verdict on them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub operations: Vec<Operation>,
    pub verdict: Verdict,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grants_cover_exact_plugin_wildcard_and_miss() {
        let grants = Grants {
            capabilities: vec!["cron.schedule".into(), "workflows.*".into()],
        };
        assert!(grants.covers("cron.schedule"));
        assert!(grants.covers("workflows.cancel"));
        assert!(!grants.covers("identity_sled.get"));
        assert!(!grants.covers("nocaps"));
    }

    #[test]
    fn operation_capability_is_plugin_dot_method() {
        let op = Operation {
            plugin: "cron".into(),
            method: "schedule".into(),
        };
        assert_eq!(op.required_capability(), "cron.schedule");
    }
}
