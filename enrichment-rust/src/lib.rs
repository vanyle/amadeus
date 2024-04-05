use search::enriched_search::EnrichedSearch;
use search::Search;
use serde_json_helpers::merge_jsons;

pub mod currency_exchange;
pub mod neobase;
mod search;
mod serde_json_helpers;

/// Enriches a search with additional data, from Neobase, currency exchange rates and fields from the search itself.
/// Returns a new JSON object with the enriched data merged with the input JSON.
/// Fields in the input JSON will be overridden by the enriched data, but fields not present in the enriched data will be kept.
pub fn enrich_json(
    input_json: serde_json::Value,
    neobase_locations: &neobase::Locations,
    exchange_rates: &currency_exchange::ExchangeRates,
) -> serde_json::Value {
    // Serialize
    let search: Search =
        serde_json::from_value(input_json.clone()).expect("Failed to parse search");

    // Enrich
    let enriched_search = EnrichedSearch::enrich_from(&search, neobase_locations, exchange_rates);

    // back to json
    let enriched_search_json =
        serde_json::to_value(enriched_search).expect("Failed to serialize enriched search");

    // merge jsons
    let mut out_json = input_json;
    merge_jsons(&mut out_json, enriched_search_json);

    out_json
}
