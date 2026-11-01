//! End-user workflow: promoted schema tags for one routing request.
//!
//! The end user names a workflow for their own declaration (business, CMS,
//! task); the adapter expands it to tags against a caller-supplied
//! taxonomy snapshot. No I/O, no network, no default business baked in.
//!
//! This is the routing layer, distinct from
//! [`crate::catalogs::projection::Purpose`]: that closed enum selects a
//! per-schema field-class view downstream (Vectorize/Display/Overlay);
//! [`WorkflowPurpose`] below selects the tag set that [`crate::decide`]
//! routes on upstream. Values are closed (taxonomy-known), combinations
//! are open. Tags never grant — `decide` still partitions candidates
//! against the identity's compiled [`crate::model::Grants`].
//! Unknown or control-less workflows fail here so `decide` refuses upstream.

use serde::{Deserialize, Serialize};

use crate::model::{ActionKind, Request, Surface};

/// Caller-supplied taxonomy snapshot (`oscal/TAG-TAXONOMY.md`; `oscal/`
/// JSON is authoritative). Passed in so this crate stays pure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Taxonomy {
    /// Every tag the adapter may emit (control + qualifier vocabularies).
    pub known_tags: Vec<String>,
    /// Control tags: at least one must survive expansion.
    pub known_controls: Vec<String>,
}

/// One end-user workflow: a name plus promoted schema tags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowPurpose {
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowError {
    Empty,
    UnknownTag(String),
    NoControlTag,
}

impl std::fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "workflow promotes no tags"),
            Self::UnknownTag(tag) => write!(f, "tag is not taxonomy-known: {tag}"),
            Self::NoControlTag => write!(f, "workflow carries no control tag"),
        }
    }
}

/// Expand a workflow to its tag set. Closed values, open combinations.
pub fn expand(
    purpose: &WorkflowPurpose,
    taxonomy: &Taxonomy,
) -> Result<Vec<String>, WorkflowError> {
    if purpose.tags.is_empty() {
        return Err(WorkflowError::Empty);
    }
    for tag in &purpose.tags {
        if !taxonomy.known_tags.iter().any(|k| k == tag) {
            return Err(WorkflowError::UnknownTag(tag.clone()));
        }
    }
    if !purpose
        .tags
        .iter()
        .any(|t| taxonomy.known_controls.iter().any(|c| c == t))
    {
        return Err(WorkflowError::NoControlTag);
    }
    Ok(purpose.tags.clone())
}

/// Build the adapter [`Request`] for one end-user workflow.
pub fn request_for(
    actor: &str,
    surface: Surface,
    action: ActionKind,
    purpose: &WorkflowPurpose,
    taxonomy: &Taxonomy,
    payload_ref: &str,
) -> Result<Request, WorkflowError> {
    Ok(Request {
        actor: actor.to_string(),
        surface,
        action,
        tags: expand(purpose, taxonomy)?,
        payload_ref: payload_ref.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taxonomy() -> Taxonomy {
        Taxonomy {
            known_tags: vec![
                "pii".to_string(),
                "action:publish".to_string(),
                "surface:web".to_string(),
            ],
            known_controls: vec!["pii".to_string()],
        }
    }

    #[test]
    fn open_combination_of_known_tags_passes() {
        let purpose = WorkflowPurpose {
            name: "portal".to_string(),
            tags: vec!["pii".to_string(), "action:publish".to_string()],
        };
        assert_eq!(expand(&purpose, &taxonomy()).unwrap(), purpose.tags);
    }

    #[test]
    fn unknown_tag_fails_closed() {
        let purpose = WorkflowPurpose {
            name: "portal".to_string(),
            tags: vec!["pii".to_string(), "nope".to_string()],
        };
        assert_eq!(
            expand(&purpose, &taxonomy()),
            Err(WorkflowError::UnknownTag("nope".to_string()))
        );
    }

    #[test]
    fn control_less_workflow_fails() {
        let purpose = WorkflowPurpose {
            name: "portal".to_string(),
            tags: vec!["action:publish".to_string()],
        };
        assert_eq!(
            expand(&purpose, &taxonomy()),
            Err(WorkflowError::NoControlTag)
        );
    }

    #[test]
    fn empty_workflow_fails() {
        let purpose = WorkflowPurpose {
            name: "portal".to_string(),
            tags: vec![],
        };
        assert_eq!(expand(&purpose, &taxonomy()), Err(WorkflowError::Empty));
    }

    #[test]
    fn workflow_purpose_routes_to_allow() {
        use crate::model::{ActionKind, Grants, Profile, Surface, Verdict};
        let taxonomy = Taxonomy {
            known_tags: vec!["audit".to_string(), "action:read".to_string()],
            known_controls: vec!["audit".to_string()],
        };
        let purpose = WorkflowPurpose {
            name: "receipts".to_string(),
            tags: vec!["audit".to_string(), "action:read".to_string()],
        };
        let request = request_for(
            "demo",
            Surface::Web,
            ActionKind::Read,
            &purpose,
            &taxonomy,
            "payload:demo-1",
        )
        .unwrap();
        let profile = Profile {
            id: "demo".to_string(),
            controls: vec!["etched-au-1".to_string()],
        };
        let grants = Grants {
            capabilities: vec![
                "cap.identity.sled.self.read@v1".to_string(),
                "cap.governance.routing-decision.read@v1".to_string(),
            ],
        };
        let decision = crate::decide(&request, &profile, &grants);
        assert_eq!(decision.verdict, Verdict::Allow);
    }

    #[test]
    fn unsealed_candidate_refuses_before_grants() {
        use crate::model::{ActionKind, Grants, Profile, Surface, Verdict};
        let taxonomy = Taxonomy {
            known_tags: vec!["integrity".to_string()],
            known_controls: vec!["integrity".to_string()],
        };
        let purpose = WorkflowPurpose {
            name: "anchor".to_string(),
            tags: vec!["integrity".to_string()],
        };
        let request = request_for(
            "demo",
            Surface::Web,
            ActionKind::Publish,
            &purpose,
            &taxonomy,
            "payload:demo-2",
        )
        .unwrap();
        let profile = Profile {
            id: "demo".to_string(),
            controls: vec!["etched-in-1".to_string()],
        };
        let grants = Grants {
            capabilities: vec!["identity_sled.read".to_string()],
        };
        let decision = crate::decide(&request, &profile, &grants);
        assert!(matches!(decision.verdict, Verdict::Refuse { .. }));
    }
}
