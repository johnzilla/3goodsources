use super::config::MatchConfig;
use super::error::MatchError;
use super::normalize;
use crate::registry::types::{Category, Registry};

/// Result of a successful query match
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched category slug
    pub slug: String,
    /// The match score (0.0 to 1.0)
    pub score: f64,
    /// Clone of the matched category
    pub category: Category,
}

/// Match a query against the registry and return the best match
pub fn match_query(
    query: &str,
    registry: &Registry,
    config: &MatchConfig,
) -> Result<MatchResult, MatchError> {
    unimplemented!("RED phase - test first")
}

fn calculate_fuzzy_score(query: &str, slug: &str, category: &Category) -> f64 {
    unimplemented!("RED phase - test first")
}

fn calculate_keyword_score(query: &str, slug: &str) -> f64 {
    unimplemented!("RED phase - test first")
}

fn calculate_score(
    query: &str,
    slug: &str,
    category: &Category,
    config: &MatchConfig,
) -> f64 {
    unimplemented!("RED phase - test first")
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn load_test_registry() -> Registry {
        let json_content = include_str!("../../registry.json");
        serde_json::from_str(json_content).expect("Failed to parse registry.json")
    }

    fn default_config() -> MatchConfig {
        MatchConfig {
            match_threshold: 0.4,
            match_fuzzy_weight: 0.7,
            match_keyword_weight: 0.3,
        }
    }

    #[test]
    fn test_learn_rust_matches_rust_learning() {
        let registry = load_test_registry();
        let config = default_config();

        let result = match_query("learn rust", &registry, &config).unwrap();

        assert_eq!(result.slug, "rust-learning");
        assert!(result.score > 0.4, "Score should be above threshold");
        assert_eq!(result.category.name, "Rust Learning");
    }

    #[test]
    fn test_bitcoin_node_matches_bitcoin_node_setup() {
        let registry = load_test_registry();
        let config = default_config();

        let result = match_query("bitcoin node", &registry, &config).unwrap();

        assert_eq!(result.slug, "bitcoin-node-setup");
        assert!(result.score > 0.4, "Score should be above threshold");
        assert_eq!(result.category.name, "Bitcoin Node Setup");
    }

    #[test]
    fn test_email_server_matches_self_hosted_email() {
        let registry = load_test_registry();
        let config = default_config();

        let result = match_query("email server", &registry, &config).unwrap();

        assert_eq!(result.slug, "self-hosted-email");
        assert!(result.score > 0.4, "Score should be above threshold");
        assert_eq!(result.category.name, "Self-Hosted Email");
    }

    #[test]
    fn test_below_threshold_returns_error() {
        let registry = load_test_registry();
        let config = default_config();

        let result = match_query("quantum physics supercollider", &registry, &config);

        match result {
            Err(MatchError::BelowThreshold {
                threshold,
                closest_slug,
                closest_score,
                all_slugs,
            }) => {
                assert_relative_eq!(threshold, 0.4, epsilon = 1e-6);
                assert!(!closest_slug.is_empty());
                assert!(closest_score < 0.4, "Closest score should be below threshold");
                assert_eq!(all_slugs.len(), 10, "Should return all 10 category slugs");
            }
            _ => panic!("Expected BelowThreshold error"),
        }
    }

    #[test]
    fn test_keyword_boost_increases_score() {
        let registry = load_test_registry();

        // Config with keyword boosting
        let config_with_boost = MatchConfig {
            match_threshold: 0.4,
            match_fuzzy_weight: 0.7,
            match_keyword_weight: 0.3,
        };

        // Config without keyword boosting (fuzzy only)
        let config_no_boost = MatchConfig {
            match_threshold: 0.4,
            match_fuzzy_weight: 1.0,
            match_keyword_weight: 0.0,
        };

        // Use a query that contains slug terms to trigger keyword boost
        let result_with_boost = match_query("bitcoin node", &registry, &config_with_boost).unwrap();
        let result_no_boost = match_query("bitcoin node", &registry, &config_no_boost).unwrap();

        assert!(
            result_with_boost.score > result_no_boost.score,
            "Score with keyword boost ({}) should be higher than fuzzy-only ({})",
            result_with_boost.score,
            result_no_boost.score
        );
    }

    #[test]
    fn test_empty_query_returns_error() {
        let registry = load_test_registry();
        let config = default_config();

        let result = match_query("", &registry, &config);

        assert!(matches!(result, Err(MatchError::EmptyQuery)));
    }

    #[test]
    fn test_all_stop_words_returns_error() {
        let registry = load_test_registry();
        let config = default_config();

        let result = match_query("the a an", &registry, &config);

        assert!(matches!(result, Err(MatchError::QueryAllStopWords)));
    }

    #[test]
    fn test_best_match_wins() {
        let registry = load_test_registry();
        let config = default_config();

        // This query could potentially match multiple categories,
        // but we should get exactly one result: the highest scorer
        let result = match_query("setup guide", &registry, &config).unwrap();

        // We should get a single result with the highest score
        assert!(!result.slug.is_empty());
        assert!(result.score > 0.4);
    }
}
