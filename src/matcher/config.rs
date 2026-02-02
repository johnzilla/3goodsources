use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MatchConfig {
    /// Minimum score to accept a match (default: 0.4)
    #[serde(default = "default_threshold")]
    pub match_threshold: f64,

    /// Weight for fuzzy similarity score (default: 0.7)
    #[serde(default = "default_fuzzy_weight")]
    pub match_fuzzy_weight: f64,

    /// Weight for keyword boost score (default: 0.3)
    #[serde(default = "default_keyword_weight")]
    pub match_keyword_weight: f64,
}

fn default_threshold() -> f64 {
    0.4
}

fn default_fuzzy_weight() -> f64 {
    0.7
}

fn default_keyword_weight() -> f64 {
    0.3
}

impl MatchConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        envy::from_env::<MatchConfig>().map_err(|e| {
            anyhow::anyhow!("Failed to load match config: {}", e)
        })
    }

    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.match_threshold < 0.0 || self.match_threshold > 1.0 {
            anyhow::bail!("MATCH_THRESHOLD must be between 0.0 and 1.0");
        }
        if (self.match_fuzzy_weight + self.match_keyword_weight - 1.0).abs() > 0.01 {
            anyhow::bail!(
                "MATCH_FUZZY_WEIGHT + MATCH_KEYWORD_WEIGHT must sum to 1.0 (got {:.2} + {:.2} = {:.2})",
                self.match_fuzzy_weight,
                self.match_keyword_weight,
                self.match_fuzzy_weight + self.match_keyword_weight
            );
        }
        Ok(())
    }
}
