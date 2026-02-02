use thiserror::Error;

#[derive(Error, Debug)]
pub enum MatchError {
    #[error("Query cannot be empty")]
    EmptyQuery,

    #[error("Query contains only stop words and has no searchable content")]
    QueryAllStopWords,

    #[error(
        "No category matches query (threshold: {threshold:.2}). Closest: {closest_slug} ({closest_score:.2}). Available: {all_slugs:?}. Request a new category at https://github.com/johnzilla/3goodsources"
    )]
    BelowThreshold {
        threshold: f64,
        closest_slug: String,
        closest_score: f64,
        all_slugs: Vec<String>,
    },
}
