use std::fs;
use std::str::FromStr;

mod currency_exchange;
mod neobase;
mod search;
mod serde_json_helpers;

use search::Search;
use search::enriched_search::EnrichedSearch;
use serde_json_helpers::merge_jsons;

fn main() {
    // TODO : read json
    // cast json to search, leaving unused fields untouched
    // enrich
    // create json from enriched search, adding old untouched fields

    // Utils
    let neobase_locations = neobase::Locations::new();
    let exchange_rates = currency_exchange::ExchangeRates::new();

    // Read sample.json
    let input_json = serde_json::Value::from_str(&fs::read_to_string("sample.json").expect("Failed to read sample.json")).expect("Failed to parse sample.json");

    // Serialize
    let search: Search = serde_json::from_value(input_json.clone()).expect("Failed to parse search");
    
    // Enrich
    let enriched_search = EnrichedSearch::enrich_from(&search, &neobase_locations, &exchange_rates);
    
    // back to json
    let enriched_search_json = serde_json::to_value(&enriched_search).expect("Failed to serialize enriched search");
    
    // merge jsons
    let mut out_json = input_json;
    merge_jsons(&mut out_json, enriched_search_json);
    
    // write to file
    fs::write("out.json", serde_json::to_string(&out_json).expect("Failed to convert json to string")).expect("Failed to write out.json"); 
}
