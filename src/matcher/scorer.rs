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
    // Stage 1: Normalize query (propagates EmptyQuery/QueryAllStopWords errors)
    let normalized_query = normalize::normalize_text(query)?;

    // Stage 2: Score all categories
    let mut scores: Vec<(String, f64, Category)> = registry
        .categories
        .iter()
        .map(|(slug, category)| {
            let score = calculate_score(&normalized_query, slug, category, config);
            (slug.clone(), score, category.clone())
        })
        .collect();

    // Stage 3: Sort by score descending
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Get best match
    let (best_slug, best_score, best_category) = scores
        .first()
        .expect("Registry should have at least one category");

    // Stage 4: Threshold check
    if *best_score >= config.match_threshold {
        Ok(MatchResult {
            slug: best_slug.clone(),
            score: *best_score,
            category: best_category.clone(),
        })
    } else {
        // Collect all slugs for error message
        let all_slugs: Vec<String> = registry.categories.keys().cloned().collect();

        Err(MatchError::BelowThreshold {
            threshold: config.match_threshold,
            closest_slug: best_slug.clone(),
            closest_score: *best_score,
            all_slugs,
        })
    }
}

/// Calculate fuzzy similarity score across all match surfaces
fn calculate_fuzzy_score(query: &str, slug: &str, category: &Category) -> f64 {
    let mut max_score: f64 = 0.0;

    // Surface 1: Compare against slug with hyphens replaced by spaces
    let slug_as_text = slug.replace('-', " ");
    max_score = max_score.max(strsim::normalized_levenshtein(query, &slug_as_text));

    // Surface 2: Compare against category name lowercased
    let name_lower = category.name.to_lowercase();
    max_score = max_score.max(strsim::normalized_levenshtein(query, &name_lower));

    // Surface 3: Compare against each query pattern (normalized)
    for pattern in &category.query_patterns {
        // Normalize pattern before comparison
        if let Ok(normalized_pattern) = normalize::normalize_text(pattern) {
            let score = strsim::normalized_levenshtein(query, &normalized_pattern);
            max_score = max_score.max(score);
        }
    }

    max_score
}

/// Calculate keyword boost score based on slug term presence in query
fn calculate_keyword_score(query: &str, slug: &str) -> f64 {
    // Split slug on hyphens to get slug terms
    let slug_terms: Vec<&str> = slug.split('-').collect();
    let total_terms = slug_terms.len() as f64;

    // Count how many slug terms appear in the query
    let matches = slug_terms
        .iter()
        .filter(|term| query.contains(*term))
        .count() as f64;

    // Return fraction of slug terms found in query
    matches / total_terms
}

/// Calculate combined score using weighted sum
fn calculate_score(
    query: &str,
    slug: &str,
    category: &Category,
    config: &MatchConfig,
) -> f64 {
    let fuzzy_score = calculate_fuzzy_score(query, slug, category);
    let keyword_score = calculate_keyword_score(query, slug);

    // Weighted sum combination
    let combined = (config.match_fuzzy_weight * fuzzy_score)
        + (config.match_keyword_weight * keyword_score);

    // Clamp to [0.0, 1.0] range
    combined.min(1.0)
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
        let category = registry.categories.get("bitcoin-node-setup").unwrap();

        // Query containing exact slug terms
        let query = "bitcoin node";
        let normalized_query = normalize::normalize_text(query).unwrap();

        // Config with keyword boosting
        let config_with_boost = MatchConfig {
            match_threshold: 0.4,
            match_fuzzy_weight: 0.7,
            match_keyword_weight: 0.3,
        };

        // Config without keyword boosting (fuzzy only)
        let config_no_boost = MatchConfig {
            match_threshold: 0.4,
            match_fuzzy_weight: 0.7,
            match_keyword_weight: 0.0,
        };

        // Calculate scores directly to verify keyword boost effect
        let score_with_boost = super::calculate_score(
            &normalized_query,
            "bitcoin-node-setup",
            category,
            &config_with_boost,
        );
        let score_no_boost = super::calculate_score(
            &normalized_query,
            "bitcoin-node-setup",
            category,
            &config_no_boost,
        );

        assert!(
            score_with_boost > score_no_boost,
            "Score with keyword boost ({}) should be higher than without ({})",
            score_with_boost,
            score_no_boost
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
        let result = match_query("rust programming", &registry, &config).unwrap();

        // We should get a single result with the highest score
        assert_eq!(result.slug, "rust-learning");
        assert!(result.score > 0.4);
        // Verify we got exactly one result (not a list)
        assert!(!result.category.sources.is_empty());
    }
}
