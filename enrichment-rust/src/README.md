# Enrichment (Rust)

A rust implementation of the enrichment algorithm.

The enrichment is done in `lib.rs`, in `enrich_json` :

```rust
fn enrich_json(
    input_json: serde_json::Value,
    neobase_locations: &neobase::Locations,
    exchange_rates: &currency_exchange::ExchangeRates,
) -> serde_json::Value
```

This function takes a JSON object to enrich it, and needs two modules to work: `neobase`, named after the equivalent python package, that will help compute distances and find location codes from the GeoBase database, and `currency_exchange`, that will help convert currencies to euros.

It returns a rust json object, ready to be written to a file, a string, etc.

## Input

```json
{
    "version_nb": "1.0",
    "search_id": "LRX-51980-1637149713-8763",
    "search_country": "RU",                     // Country code
    "search_date": "2021-11-17",                // YYYY-MM-DD date
    "search_time": "11:48:39",                  // HH:MM:SS time
    "origin_city": "PAR",                       // City code
    "destination_city": "LIS",                  // City code
    "request_dep_date": "2021-12-17",           // YYYY-MM-DD date
    "request_return_date": "2021-12-19",        // YYYY-MM-DD date   OPTIONAL!
    "passengers_string": "ADT=2",               // ADT=[number of adults],CHD=[number of children]
    "currency": "RUB",                          // 3-letter currency code
    "recos": [                                  // Array of recommendations
        {
            "price": "47925.36",                // Price in the currency specified above. Format: "X.XX" or X.XX
            "taxes": "16412.46",                // Idem
            "fees": "0.00",                     // Idem
            "nb_of_flights": 3,
            "flights": [                        // Array of flights
                {
                    "dep_airport": "CDG",       // Airport code
                    "dep_date": "2021-12-17",   // YYYY-MM-DD date
                    "dep_time": "20:55",
                    "arr_airport": "AMS",       // Airport code
                    "arr_date": "2021-12-17",   // YYYY-MM-DD date
                    "arr_time": "22:10",
                    "operating_airline": "KL",  // Airline code     OPTIONAL!
                    "marketing_airline": "KL",  // Airline code
                    "flight_nb": "1246",
                    "cabin": "M"
                }
            ]
        }
    ],
    "OnD": "PAR-LIS"
}
```

## Output

Order of the fields is not guaranteed.

```json
{
    "OnD": "PAR-LIS",
    "OnD_distance": 1452,
    "advance_purchase": 30,
    "currency": "RUB",
    "destination_city": "LIS",
    "destination_country": "PT",
    "geo": "I",                         // I for international, D for domestic
    "origin_city": "PAR",
    "origin_country": "FR",
    "passengers": [                     // Array of passengers, translated from passengers_string
        {
            "passenger_nb": 2,
            "passenger_type": "ADT"
        },
        {
            "passenger_nb": 3,
            "passenger_type": "CH"
        }
    ],
    "passengers_string": "ADT=2,CH=3",  // Kept from input
    "recos": [
        {
            "fees": "0.00",
            "fees_EUR": 0.0,
            "flights": [
                {
                    "arr_airport": "AMS",
                    "arr_city": "AMS",
                    "arr_date": "2021-12-17",
                    "arr_time": "22:10",
                    "cabin": "M",
                    "dep_airport": "CDG",
                    "dep_city": "PAR",
                    "dep_date": "2021-12-17",
                    "dep_time": "20:55",
                    "distance": 397,
                    "flight_nb": "1246",
                    "marketing_airline": "KL",
                    "operating_airline": "KL"
                }
            ],
            "flown_distance": 3724,
            "main_cabin": "M",
            "main_marketing_airline": "KL",
            "main_operating_airline": "KL",
            "nb_of_flights": 3,
            "price": "47925.36",
            "price_EUR": 578.72,
            "taxes": "16412.46",
            "taxes_EUR": 198.19
        }
    ],
    "request_dep_date": "2021-12-17",
    "request_return_date": "2021-12-19",
    "search_country": "RU",
    "search_date": "2021-11-17",
    "search_id": "LRX-51980-1637149713-8763",
    "search_time": "11:48:39",
    "stay_duration": 2,
    "trip_type": "RT",
    "version_nb": "1.0"
}
```