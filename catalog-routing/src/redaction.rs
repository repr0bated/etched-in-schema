//! Deterministic PII screen: layer one of the pre-publish gate.
//!
//! Pure string/JSON scanning, no I/O, no model calls. Runs before any
//! semantic (JEV) screen: field-tag classification first, these patterns
//! second, JEV only for ambiguous free text — and any hit at any layer
//! blocks publish (fail-closed).
//!
//! Deliberately dependency-free (no regex crate): small hand-rolled
//! matchers over explicit shapes. New shapes are added here with a test,
//! never inferred at runtime. Raw hits are reported as rule names plus
//! byte offsets, never echoed back — the check must not leak what
//! redaction is meant to protect.

use serde_json::Value;

/// One deterministic hit: rule name plus byte offsets into the scanned text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub rule: &'static str,
    pub start: usize,
    pub end: usize,
}

fn is_email_at(text: &str, at: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut left = at;
    while left > 0 && is_email_local(bytes[left - 1]) {
        left -= 1;
    }
    let mut right = at + 1;
    while right < bytes.len() && is_email_domain(bytes[right]) {
        right += 1;
    }
    let candidate = &text[left..right];
    if left == at || right == at + 1 {
        return None;
    }
    let mut parts = candidate.split('@');
    let domain = parts.nth(1).unwrap_or("");
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return None;
    }
    Some((left, right))
}

fn is_email_local(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'%' | b'+' | b'-')
}

fn is_email_domain(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-')
}

/// Scan one string. Returns hits ordered by offset.
pub fn screen_text(text: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            if let Some((s, e)) = is_email_at(text, i) {
                hits.push(Hit {
                    rule: "email",
                    start: s,
                    end: e,
                });
                i = e;
                continue;
            }
        }
        i += 1;
    }
    hits.extend(screen_digit_runs(text, &hits));
    hits.sort_by_key(|h| (h.start, h.end));
    hits
}

/// Phone / SSN / secret runs: digit-heavy spans of length ≥ 7 after
/// stripping separators, excluding spans already covered by email hits.
fn screen_digit_runs(text: &str, covered: &[Hit]) -> Vec<Hit> {
    let bytes = text.as_bytes();
    let mut hits = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            let mut digits = 0;
            while i < bytes.len()
                && (bytes[i].is_ascii_digit()
                    || matches!(bytes[i], b' ' | b'-' | b'.' | b'(' | b')'))
            {
                if bytes[i].is_ascii_digit() {
                    digits += 1;
                }
                i += 1;
            }
            if digits >= 7
                && digits <= 15
                && !covered.iter().any(|h| start >= h.start && start < h.end)
            {
                hits.push(Hit {
                    rule: if digits == 9 { "id-digits" } else { "phone-digits" },
                    start,
                    end: i,
                });
                continue;
            }
            continue;
        }
        i += 1;
    }
    hits
}

/// Walk a JSON value, screening every string. Object keys are screened too:
/// a field named `email` carrying free text is still a leak vector.
pub fn screen_value(value: &Value) -> Vec<Hit> {
    let mut hits = Vec::new();
    match value {
        Value::String(s) => hits.extend(screen_text(s)),
        Value::Array(items) => {
            for item in items {
                hits.extend(screen_value(item));
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                hits.extend(screen_text(k));
                hits.extend(screen_value(v));
            }
        }
        _ => {}
    }
    hits
}

/// Gate verdict: clean passes, anything else blocks. No silent truncation.
pub fn clean(hits: &[Hit]) -> bool {
    hits.is_empty()
}

/// Deterministic mask: replace every hit span with `[redacted:<rule>]`.
/// Spans must come from [`screen_text`] on the same input (ordered,
/// non-overlapping by construction). Returns the masked text plus the
/// count of masked spans, so the caller can receipt what was removed
/// without ever logging the raw values.
///
/// Preview only: masked output does NOT clear publish. Publish still
/// needs the sealed masking op plus its receipt naming the policy; this
/// crate proposes, the bridge disposes. Treat any caller that publishes
/// on `clean()` alone as a policy violation.
pub fn mask_text(text: &str, hits: &[Hit]) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut count = 0;
    for hit in hits {
        if hit.start < cursor || hit.end > text.len() || hit.start >= hit.end {
            continue;
        }
        out.push_str(&text[cursor..hit.start]);
        out.push_str("[redacted:");
        out.push_str(hit.rule);
        out.push(']');
        cursor = hit.end;
        count += 1;
    }
    out.push_str(&text[cursor..]);
    (out, count)
}

/// Mask every string in a JSON value. Object keys are masked too.
pub fn mask_value(value: &Value) -> (Value, usize) {
    match value {
        Value::String(s) => {
            let (masked, n) = mask_text(s, &screen_text(s));
            (Value::String(masked), n)
        }
        Value::Array(items) => {
            let mut total = 0;
            let masked = items
                .iter()
                .map(|item| {
                    let (m, n) = mask_value(item);
                    total += n;
                    m
                })
                .collect();
            (Value::Array(masked), total)
        }
        Value::Object(map) => {
            let mut total = 0;
            let masked = map
                .iter()
                .map(|(k, v)| {
                    let (mk, nk) = mask_text(k, &screen_text(k));
                    let (mv, nv) = mask_value(v);
                    total += nk + nv;
                    (mk, mv)
                })
                .collect();
            (Value::Object(masked), total)
        }
        _ => (value.clone(), 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn email_is_hit_with_offsets() {
        let hits = screen_text("mail jane@example.com today");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule, "email");
        assert_eq!(&"mail jane@example.com today"[hits[0].start..hits[0].end], "jane@example.com");
    }

    #[test]
    fn bare_at_and_bad_domain_pass() {
        assert!(screen_text("a @ b").is_empty());
        assert!(screen_text("user@localhost").is_empty());
    }

    #[test]
    fn phone_run_is_hit() {
        let hits = screen_text("call +1 (555) 123-4567 now");
        assert!(hits.iter().any(|h| h.rule == "phone-digits"));
    }

    #[test]
    fn short_numbers_pass() {
        assert!(screen_text("order 42, qty 5").is_empty());
    }

    #[test]
    fn json_walk_finds_nested_and_keys() {
        let v = json!({"contact": {"email": "a@b.co"}, "note": "clean"});
        let hits = screen_value(&v);
        assert!(hits.iter().any(|h| h.rule == "email"));
    }

    #[test]
    fn clean_gate_is_fail_closed() {
        assert!(clean(&[]));
        assert!(!clean(&[Hit {
            rule: "email",
            start: 0,
            end: 1
        }]));
    }

    #[test]
    fn masker_replaces_hits_and_counts() {
        let text = "mail jane@example.com or 5551234567";
        let (masked, n) = mask_text(text, &screen_text(text));
        assert_eq!(n, 2);
        assert!(!masked.contains("jane@example.com"));
        assert!(!masked.contains("5551234567"));
        assert!(masked.contains("[redacted:email]"));
        assert!(masked.contains("[redacted:phone-digits]"));
    }

    #[test]
    fn masker_is_total_over_json() {
        let v = json!({"contact": "a@b.co", "note": "clean", "n": 7});
        let (masked, n) = mask_value(&v);
        assert_eq!(n, 1);
        assert_eq!(masked["note"], json!("clean"));
        assert_eq!(masked["n"], json!(7));
        assert!(!masked["contact"].as_str().unwrap().contains("a@b.co"));
    }
}
