//! Test utilities for parsing schema and model files.

pub mod docker_helpers;
pub mod model_parser;
pub mod schema_parser;

pub use docker_helpers::{establish_connection, reference_docker};

/// Represents a feature gate condition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FeatureGate {
    /// No feature gate - always enabled.
    Always,
    /// Feature must be enabled.
    Feature(String),
    /// Feature must NOT be enabled.
    NotFeature(String),
    /// All conditions must be met.
    All(Vec<FeatureGate>),
    /// Any condition must be met.
    Any(Vec<FeatureGate>),
    /// None of the conditions must be met (not(any(...))).
    NotAny(Vec<FeatureGate>),
}

impl FeatureGate {
    /// Check if this feature gate is enabled given the active features.
    pub fn is_enabled(&self, active_features: &[&str]) -> bool {
        match self {
            FeatureGate::Always => true,
            FeatureGate::Feature(f) => active_features.contains(&f.as_str()),
            FeatureGate::NotFeature(f) => !active_features.contains(&f.as_str()),
            FeatureGate::All(gates) => gates.iter().all(|g| g.is_enabled(active_features)),
            FeatureGate::Any(gates) => gates.iter().any(|g| g.is_enabled(active_features)),
            FeatureGate::NotAny(gates) => !gates.iter().any(|g| g.is_enabled(active_features)),
        }
    }

    /// Parse a cfg attribute meta into a `FeatureGate`.
    pub fn from_cfg_meta(meta: &syn::Meta) -> Option<Self> {
        match meta {
            syn::Meta::List(list) if list.path.is_ident("cfg") => {
                // Parse the tokens inside cfg(...)
                Self::parse_cfg_predicate(&list.tokens)
            }
            _ => None,
        }
    }

    fn parse_cfg_predicate(tokens: &proc_macro2::TokenStream) -> Option<Self> {
        let tokens_str = tokens.to_string();

        // Handle `feature = "name"`
        if tokens_str.starts_with("feature")
            && let Some(start) = tokens_str.find('"')
            && let Some(end) = tokens_str.rfind('"')
            && start < end
        {
            let feature_name = &tokens_str[start + 1..end];
            return Some(FeatureGate::Feature(feature_name.to_string()));
        }

        // Handle `not(any(feature = "X", feature = "Y"))`
        if tokens_str.starts_with("not (any (") || tokens_str.starts_with("not(any(") {
            let features = extract_feature_names(&tokens_str);
            if !features.is_empty() {
                let gates: Vec<FeatureGate> =
                    features.into_iter().map(FeatureGate::Feature).collect();
                return Some(FeatureGate::NotAny(gates));
            }
        }

        // Handle `not(feature = "name")`
        if tokens_str.starts_with("not")
            && let Some(inner_start) = tokens_str.find('(')
            && let Some(inner_end) = tokens_str.rfind(')')
        {
            let inner = &tokens_str[inner_start + 1..inner_end];
            if inner.trim().starts_with("feature")
                && let Some(start) = inner.find('"')
                && let Some(end) = inner.rfind('"')
                && start < end
            {
                let feature_name = &inner[start + 1..end];
                return Some(FeatureGate::NotFeature(feature_name.to_string()));
            }
        }

        // Handle `any(feature = "X", feature = "Y")`
        if tokens_str.starts_with("any (") || tokens_str.starts_with("any(") {
            let features = extract_feature_names(&tokens_str);
            if !features.is_empty() {
                let gates: Vec<FeatureGate> =
                    features.into_iter().map(FeatureGate::Feature).collect();
                return Some(FeatureGate::Any(gates));
            }
        }

        // Handle `all(...)` - similar to any
        if tokens_str.starts_with("all(") || tokens_str.starts_with("all (") {
            let features = extract_feature_names(&tokens_str);
            if !features.is_empty() {
                let gates: Vec<FeatureGate> =
                    features.into_iter().map(FeatureGate::Feature).collect();
                return Some(FeatureGate::All(gates));
            }
        }

        None
    }
}

/// Extract all feature names from a cfg predicate string.
fn extract_feature_names(s: &str) -> Vec<String> {
    let mut features = Vec::new();
    let mut remaining = s;

    while let Some(start) = remaining.find("feature") {
        remaining = &remaining[start..];
        if let Some(quote_start) = remaining.find('"') {
            remaining = &remaining[quote_start + 1..];
            if let Some(quote_end) = remaining.find('"') {
                let feature_name = &remaining[..quote_end];
                features.push(feature_name.to_string());
                remaining = &remaining[quote_end + 1..];
            } else {
                break;
            }
        } else {
            break;
        }
    }

    features
}

/// Get the active postgres features based on compile-time cfg.
#[allow(unused_mut, clippy::vec_init_then_push, dead_code)]
pub fn get_active_postgres_features() -> Vec<&'static str> {
    let mut features = vec![];

    #[cfg(feature = "postgres-14")]
    features.push("postgres-14");

    #[cfg(feature = "postgres-15")]
    features.push("postgres-15");

    #[cfg(feature = "postgres-16")]
    features.push("postgres-16");

    #[cfg(feature = "postgres-17")]
    features.push("postgres-17");

    #[cfg(feature = "postgres-18")]
    features.push("postgres-18");

    #[cfg(feature = "routines")]
    features.push("routines");

    #[cfg(feature = "serde")]
    features.push("serde");

    features
}

/// Checks if a column/field should be enabled given the active features.
#[allow(dead_code)]
pub fn is_enabled_for_current_features(gate: &FeatureGate) -> bool {
    let active = get_active_postgres_features();
    gate.is_enabled(&active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_gate_always() {
        let gate = FeatureGate::Always;
        assert!(gate.is_enabled(&[]));
        assert!(gate.is_enabled(&["postgres-17"]));
    }

    #[test]
    fn test_feature_gate_feature() {
        let gate = FeatureGate::Feature("postgres-17".to_string());
        assert!(!gate.is_enabled(&[]));
        assert!(gate.is_enabled(&["postgres-17"]));
        assert!(!gate.is_enabled(&["postgres-18"]));
    }

    #[test]
    fn test_feature_gate_not_feature() {
        let gate = FeatureGate::NotFeature("postgres-18".to_string());
        assert!(gate.is_enabled(&[]));
        assert!(gate.is_enabled(&["postgres-17"]));
        assert!(!gate.is_enabled(&["postgres-18"]));
    }
}
