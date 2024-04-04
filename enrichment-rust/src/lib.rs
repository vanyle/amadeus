use search::enriched_search::EnrichedSearch;
use search::Search;
use serde_json_helpers::merge_jsons;

pub mod currency_exchange;
pub mod neobase;
mod search;
mod serde_json_helpers;

pub fn enrich_json(
    input_json: serde_json::Value,
    neobase_locations: &neobase::Locations,
    exchange_rates: &currency_exchange::ExchangeRates,
) -> serde_json::Value {
    // Serialize
    let search: Search =
        serde_json::from_value(input_json.clone()).expect("Failed to parse search");

    // Enrich
    let enriched_search = EnrichedSearch::enrich_from(&search, &neobase_locations, &exchange_rates);

    // back to json
    let enriched_search_json =
        serde_json::to_value(&enriched_search).expect("Failed to serialize enriched search");

    // merge jsons
    let mut out_json = input_json;
    merge_jsons(&mut out_json, enriched_search_json);

    out_json
}
