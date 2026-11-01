//! End-user-driven purpose: promoted schema tags for one workflow.
//!
//! The end user names a purpose for their own declaration (business, CMS,
//! workflow); the adapter expands it to tags against a caller-supplied
//! taxonomy snapshot. No I/O, no network, no default business baked in.
//!
//! Split that keeps this expandable without widening authority:
//! * values are closed — every tag must be taxonomy-known;
//! * combinations are open — any subset composes a new purpose;
//! * tags never grant — [`crate::decide`] still partitions candidates
//!   against the identity's compiled [`crate::model::Grants`].
//! Unknown or control-less purposes fail here so `decide` refuses upstream.

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

/// One end-user purpose: a name plus promoted schema tags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Purpose {
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurposeError {
    Empty,
    UnknownTag(String),
    NoControlTag,
}

impl std::fmt::Display for PurposeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "purpose promotes no tags"),
            Self::UnknownTag(tag) => write!(f, "tag is not taxonomy-known: {tag}"),
            Self::NoControlTag => write!(f, "purpose carries no control tag"),
        }
    }
}

/// Expand a purpose to its tag set. Closed values, open combinations.
pub fn expand(purpose: &Purpose, taxonomy: &Taxonomy) -> Result<Vec<String>, PurposeError> {
    if purpose.tags.is_empty() {
        return Err(PurposeError::Empty);
    }
    for tag in &purpose.tags {
        if !taxonomy.known_tags.iter().any(|k| k == tag) {
            return Err(PurposeError::UnknownTag(tag.clone()));
        }
    }
    if !purpose
        .tags
        .iter()
        .any(|t| taxonomy.known_controls.iter().any(|c| c == t))
    {
        return Err(PurposeError::NoControlTag);
    }
    Ok(purpose.tags.clone())
}

/// Build the adapter [`Request`] for one end-user purpose.
pub fn request_for(
    actor: &str,
    surface: Surface,
    action: ActionKind,
    purpose: &Purpose,
    taxonomy: &Taxonomy,
    payload_ref: &str,
) -> Result<Request, PurposeError> {
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
        let purpose = Purpose {
            name: "portal".to_string(),
            tags: vec!["pii".to_string(), "action:publish".to_string()],
        };
        assert_eq!(expand(&purpose, &taxonomy()).unwrap(), purpose.tags);
    }

    #[test]
    fn unknown_tag_fails_closed() {
        let purpose = Purpose {
            name: "portal".to_string(),
            tags: vec!["pii".to_string(), "nope".to_string()],
        };
        assert_eq!(
            expand(&purpose, &taxonomy()),
            Err(PurposeError::UnknownTag("nope".to_string()))
        );
    }

    #[test]
    fn control_less_purpose_fails() {
        let purpose = Purpose {
            name: "portal".to_string(),
            tags: vec!["action:publish".to_string()],
        };
        assert_eq!(expand(&purpose, &taxonomy()), Err(PurposeError::NoControlTag));
    }

    #[test]
    fn empty_purpose_fails() {
        let purpose = Purpose {
            name: "portal".to_string(),
            tags: vec![],
        };
        assert_eq!(expand(&purpose, &taxonomy()), Err(PurposeError::Empty));
    }
}
