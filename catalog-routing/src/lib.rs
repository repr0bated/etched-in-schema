//! Catalog routing: match a request against nested catalogs, then decide.
//!
//! Pipeline (`docs/DROP-IN-SPEC.md` §2.2): adapter → [`decide`] → enforcement.
//! This crate computes the decision only. Enforcement (mount call files,
//! receipts) stays with the bridge; grants come from the compiled profile.
//!
//! ```text
//! Request { actor, surface, action, tags }
//!   → catalogs::compliance::match_controls  (tags → controls → operations)
//!   → catalogs::ui::check                   (target support, degradation)
//!   → catalogs::models::select              (provider/model for the turn)
//!   → decide                               (allow | refuse | needs-approval)
//! ```

pub mod purpose;
pub mod redaction;
pub use purpose::{Taxonomy, WorkflowError, WorkflowPurpose, expand, request_for};
pub use redaction::{Hit, clean, mask_text, mask_value, screen_text, screen_value};
pub mod acceptance;
pub mod catalogs;
pub mod decide;
pub mod model;
pub mod org_bridge;

pub use decide::decide;
pub use model::{ActionKind, Decision, Grants, Profile, Request, Surface, Verdict};
