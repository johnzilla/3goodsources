use super::{Category, Registry, RegistryError};
use regex::Regex;
use std::path::Path;
use tokio::fs;

/// Load and validate registry from disk
pub async fn load(path: impl AsRef<Path>) -> Result<Registry, RegistryError> {
    let path = path.as_ref();
    let path_str = path.display().to_string();

    tracing::info!(path = %path.display(), "Loading registry");

    // Read file from disk
    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| RegistryError::FileRead {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    // Parse JSON with line/column error reporting
    let registry: Registry = serde_json::from_str(&contents).map_err(|e| {
        RegistryError::JsonParse {
            path: path_str.clone(),
            error: e.to_string(),
            line: e.line(),
            column: e.column(),
        }
    })?;

    // Validate business rules
    validate(&registry)?;

    tracing::info!(
        version = %registry.version,
        categories = registry.categories.len(),
        sources = count_sources(&registry),
        curator = %registry.curator.name,
        "Registry loaded successfully"
    );

    Ok(registry)
}

/// Validate registry business rules
fn validate(registry: &Registry) -> Result<(), RegistryError> {
    let slug_pattern = Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();

    for (slug, category) in &registry.categories {
        // Validate slug format (lowercase alphanumeric with hyphens)
        if !slug_pattern.is_match(slug) {
            return Err(RegistryError::InvalidSlug {
                slug: slug.clone(),
            });
        }

        // Validate source count (must be exactly 3 for v1)
        if category.sources.len() != 3 {
            return Err(RegistryError::InvalidSourceCount {
                category: category.name.clone(),
                expected: 3,
                actual: category.sources.len(),
            });
        }

        // Validate query patterns (minimum 3 required)
        if category.query_patterns.len() < 3 {
            return Err(RegistryError::InsufficientQueryPatterns {
                category: category.name.clone(),
                minimum: 3,
                actual: category.query_patterns.len(),
            });
        }

        // Validate source ranks (must be sequential 1, 2, 3)
        let mut ranks: Vec<u8> = category.sources.iter().map(|s| s.rank).collect();
        ranks.sort_unstable();
        let expected_ranks: Vec<u8> = (1..=3).collect();

        if ranks != expected_ranks {
            return Err(RegistryError::InvalidRanks {
                category: category.name.clone(),
                actual: ranks,
                expected: 3,
            });
        }
    }

    Ok(())
}

/// Count total sources across all categories
fn count_sources(registry: &Registry) -> usize {
    registry
        .categories
        .values()
        .map(|cat| cat.sources.len())
        .sum()
}
