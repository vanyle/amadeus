use serde::Serialize;

use crate::{currency_exchange, neobase, serde_json_helpers::serialize_f64_2_decimals};

use super::{enriched_flight::EnrichedFlight, reco::Reco};

#[derive(Serialize)]
pub struct EnrichedReco {
    // Enriched
    #[serde(rename = "price_EUR", serialize_with = "serialize_f64_2_decimals")]
    pub price_eur: f64,
    #[serde(rename = "taxes_EUR", serialize_with = "serialize_f64_2_decimals")]
    pub taxes_eur: f64,
    #[serde(rename = "fees_EUR", serialize_with = "serialize_f64_2_decimals")]
    pub fees_eur: f64,
    pub flights: Vec<EnrichedFlight>, // overriden
    pub flown_distance: u64,
    pub main_marketing_airline: String,
    pub main_operating_airline: String,
    pub main_cabin: String,
}

impl EnrichedReco {
    pub fn enrich_from(
        reco: &Reco,
        neobase_locations: &neobase::Locations,
        exchange_rates: &currency_exchange::ExchangeRates,
        currency: &currency_exchange::Currency,
    ) -> EnrichedReco {
        let price_eur = exchange_rates.to_euros(reco.price, currency);
        let taxes_eur = exchange_rates.to_euros(reco.taxes, currency);
        let fees_eur = exchange_rates.to_euros(reco.fees, currency);

        let flights: Vec<EnrichedFlight> = reco
            .flights
            .iter()
            // TODO : avoid cloning ?
            .map(|flight| EnrichedFlight::enrich_from(flight, neobase_locations))
            .collect();

        let flown_distance: u64 = flights.iter().map(|flight| flight.distance).sum();
        let main_marketing_airline = flights
            .iter()
            .max_by_key(|flight| flight.distance)
            .unwrap()
            .marketing_airline
            .clone();
        let main_operating_airline = flights
            .iter()
            .max_by_key(|flight| flight.distance)
            .unwrap()
            .operating_airline
            .clone();
        let main_cabin = flights
            .iter()
            .max_by_key(|flight| flight.distance)
            .unwrap()
            .cabin
            .clone();

        EnrichedReco {
            flights,
            price_eur,
            taxes_eur,
            fees_eur,
            flown_distance,
            main_marketing_airline,
            main_operating_airline,
            main_cabin,
        }
    }
}
