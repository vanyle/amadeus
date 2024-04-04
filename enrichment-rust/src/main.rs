use std::fs;

use enrichment_rust::currency_exchange;
use enrichment_rust::enrich_json;
use enrichment_rust::neobase;

fn main() {
    // Utils
    let neobase_locations = neobase::Locations::new();
    let exchange_rates = currency_exchange::ExchangeRates::new();

    // Read sample.json
    let input_json = serde_json::from_str(
        &fs::read_to_string("sample.json").expect("Failed to read sample.json"),
    )
    .expect("Failed to parse sample.json");

    // Enrich
    let output_json = enrich_json(input_json, &neobase_locations, &exchange_rates);

    // write to file
    fs::write(
        "out.json",
        serde_json::to_string(&output_json).expect("Failed to convert json to string"),
    )
    .expect("Failed to write out.json");
}
