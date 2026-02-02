use super::error::MatchError;
use regex::Regex;
use std::collections::HashSet;

/// Normalize text for matching through a 4-stage pipeline:
/// 1. Lowercase
/// 2. Strip punctuation
/// 3. Remove stop words
/// 4. Normalize whitespace
///
/// Returns an error if the input is empty or contains only stop words.
pub fn normalize_text(text: &str) -> Result<String, MatchError> {
    // Pre-check: empty query
    if text.trim().is_empty() {
        return Err(MatchError::EmptyQuery);
    }

    // Stage 1: Lowercase
    let mut normalized = text.to_lowercase();

    // Stage 2: Strip punctuation (keep only word characters and whitespace)
    let punct_re = Regex::new(r"[^\w\s]").unwrap();
    normalized = punct_re.replace_all(&normalized, "").to_string();

    // Stage 3: Remove stop words
    let stop_words: HashSet<String> = stop_words::get(stop_words::LANGUAGE::English)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    normalized = normalized
        .split_whitespace()
        .filter(|word| !stop_words.contains(*word))
        .collect::<Vec<_>>()
        .join(" ");

    // Stage 4: Normalize whitespace (trim + collapse multiple spaces)
    let ws_re = Regex::new(r"\s+").unwrap();
    normalized = ws_re.replace_all(normalized.trim(), " ").to_string();

    // Post-check: all stop words removed
    if normalized.is_empty() {
        return Err(MatchError::QueryAllStopWords);
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_normalization() {
        let result = normalize_text("Learn Rust Programming").unwrap();
        assert_eq!(result, "learn rust programming");
    }

    #[test]
    fn test_punctuation_removal() {
        let result = normalize_text("don't panic! it's fine.").unwrap();
        // "don't" -> "dont", "it's" -> "its", removes punctuation
        // Then removes stop words: "dont" is not a stop word, "panic" not a stop word, "its" is a stop word, "fine" not a stop word
        assert_eq!(result, "dont panic fine");
    }

    #[test]
    fn test_stop_word_removal() {
        let result = normalize_text("how do I run a bitcoin node").unwrap();
        // Removes: "how", "do", "i" (from "I"), "a"
        // Keeps: "run", "bitcoin", "node"
        assert_eq!(result, "run bitcoin node");
    }

    #[test]
    fn test_whitespace_normalization() {
        let result = normalize_text("  too   many    spaces  ").unwrap();
        // Trims and collapses to single spaces
        // "too" is a stop word in NLTK, so it gets removed
        assert_eq!(result, "many spaces");
    }

    #[test]
    fn test_empty_query() {
        let result = normalize_text("");
        assert!(matches!(result, Err(MatchError::EmptyQuery)));
    }

    #[test]
    fn test_whitespace_only_query() {
        let result = normalize_text("   ");
        assert!(matches!(result, Err(MatchError::EmptyQuery)));
    }

    #[test]
    fn test_all_stop_words() {
        let result = normalize_text("the a an");
        assert!(matches!(result, Err(MatchError::QueryAllStopWords)));
    }

    #[test]
    fn test_mixed_case_and_punctuation() {
        let result = normalize_text("What's the BEST way?").unwrap();
        // "What's" -> "whats", "the" removed, "BEST" -> "best", "way" -> "way"
        // "whats" is not a stop word, "best" not a stop word
        // Note: "way" is NOT a stop word in NLTK
        assert_eq!(result, "whats best way");
    }

    #[test]
    fn test_preserves_content_words() {
        let result = normalize_text("bitcoin node setup guide").unwrap();
        // None of these are stop words
        assert_eq!(result, "bitcoin node setup guide");
    }

    #[test]
    fn test_query_with_numbers() {
        let result = normalize_text("How to setup Bitcoin 2.0 node").unwrap();
        // Removes: "how", "to", "a" (none here actually)
        // "2.0" -> "20" (punctuation stripped)
        // Keeps: "setup", "bitcoin", "20", "node"
        assert_eq!(result, "setup bitcoin 20 node");
    }

    #[test]
    fn test_stop_words_only_after_normalization() {
        let result = normalize_text("how to do it");
        // All of these are stop words: "how", "to", "do", "it"
        assert!(matches!(result, Err(MatchError::QueryAllStopWords)));
    }
}
