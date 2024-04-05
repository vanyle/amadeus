use search::enriched_search::{EnrichSearchError, EnrichedSearch};
use search::Search;
use serde_json_helpers::merge_jsons;

pub mod currency_exchange;
pub mod neobase;
mod search;
mod serde_json_helpers;

#[derive(Debug, thiserror::Error)]
pub enum EnrichJsonError {
    #[error("Failed to parse search: {0:?}")]
    FailedToParseSearch(#[source] serde_json::Error),
    #[error("Failed to enrich search: {0:?}")]
    FailedToEnrichSearch(#[source] EnrichSearchError),
    #[error("Failed to serialize enriched search: {0:?}")]
    FailedToSerializeEnrichedSearch(#[source] serde_json::Error),
}

/// Enriches a search with additional data, from Neobase, currency exchange rates and fields from the search itself.
/// Returns a new JSON object with the enriched data merged with the input JSON.
/// Fields in the input JSON will be overridden by the enriched data, but fields not present in the enriched data will be kept.
pub fn enrich_json(
    input_json: serde_json::Value,
    neobase_locations: &neobase::Locations,
    exchange_rates: &currency_exchange::ExchangeRates,
) -> Result<serde_json::Value, EnrichJsonError> {
    // Serialize
    let search: Search =
        serde_json::from_value(input_json.clone()).map_err(EnrichJsonError::FailedToParseSearch)?;

    // Enrich
    let enriched_search = EnrichedSearch::enrich_from(&search, neobase_locations, exchange_rates)
        .map_err(EnrichJsonError::FailedToEnrichSearch)?;

    // back to json
    let enriched_search_json = serde_json::to_value(enriched_search)
        .map_err(EnrichJsonError::FailedToSerializeEnrichedSearch)?;

    // merge jsons
    let mut out_json = input_json;
    merge_jsons(&mut out_json, enriched_search_json);

    Ok(out_json)
}
