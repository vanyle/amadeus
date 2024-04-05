use serde::{Deserialize, Serialize};

use crate::{
    currency_exchange, neobase, serde_json_helpers::serialize_u64_optional_none_as_minus_one,
};

use super::{enriched_reco::EnrichedReco, typedefs::CountryCode, Search};

extern crate strum_macros;

#[derive(Serialize)]
pub enum TripType {
    #[serde(rename = "OW")]
    OneWay,
    #[serde(rename = "RT")]
    RoundTrip,
}

#[derive(Serialize, Deserialize)]
pub enum PassengerType {
    #[serde(rename = "ADT")]
    Adult,
    #[serde(rename = "CH")]
    Child,
}

#[derive(Serialize)]
pub struct Passenger {
    passenger_type: PassengerType,
    passenger_nb: u64,
}

#[derive(Serialize)]
pub enum GeoType {
    #[serde(rename = "D")]
    Domestic,
    #[serde(rename = "I")]
    International,
}

#[derive(Serialize)]
pub struct EnrichedSearch {
    // Enriched
    pub recos: Vec<EnrichedReco>, // overriden
    pub advance_purchase: u64,
    #[serde(serialize_with = "serialize_u64_optional_none_as_minus_one")]
    pub stay_duration: Option<u64>,
    pub trip_type: TripType,
    pub passengers: Vec<Passenger>,
    pub origin_country: CountryCode,
    pub destination_country: CountryCode,
    pub geo: Option<GeoType>,
    #[serde(rename = "OnD_distance")]
    pub ond_distance: u64,
}

impl EnrichedSearch {
    pub fn enrich_from(
        search: &Search,
        neobase_locations: &neobase::Locations,
        exchange_rates: &currency_exchange::ExchangeRates,
    ) -> EnrichedSearch {
        let advance_purchase = (search.request_dep_date - search.search_date).num_days() as u64;

        let stay_duration = search
            .request_return_date
            .map(|return_date| (return_date - search.request_dep_date).num_days() as u64);

        let trip_type = match &search.request_return_date {
            Some(_) => TripType::RoundTrip,
            None => TripType::OneWay,
        };

        // decoding passengers string: "ADT=1,CH=2" means 1 Adult and 2 children
        let passengers = search
            .passengers_string
            .split(',')
            .map(|passenger_string| {
                let mut iter = passenger_string.split('=');
                let passenger_type_val = serde_json::to_value(iter.next().unwrap())
                    .expect("Failed to parse passenger type string as value");
                let passenger_type = serde_json::from_value(passenger_type_val)
                    .expect("Failed to parse passenger type");
                let passenger_nb = iter.next().unwrap().parse::<u64>().unwrap();
                Passenger {
                    passenger_type,
                    passenger_nb,
                }
            })
            .collect();

        let origin_country = neobase_locations.get_country_from_city(&search.origin_city);
        let destination_country = neobase_locations.get_country_from_city(&search.destination_city);

        let geo = if origin_country == destination_country {
            Some(GeoType::Domestic)
        } else {
            Some(GeoType::International)
        };

        let ond_distance = neobase_locations
            .get_round_distance_between_locations(&search.origin_city, &search.destination_city)
            // TODO: handle error
            .unwrap_or(0);

        let recos = search
            .recos
            .iter()
            .map(|reco| {
                EnrichedReco::enrich_from(
                    reco,
                    neobase_locations,
                    exchange_rates,
                    &search.currency,
                )
            })
            .collect();

        EnrichedSearch {
            recos,
            advance_purchase,
            stay_duration,
            trip_type,
            passengers,
            origin_country,
            destination_country,
            geo,
            ond_distance,
        }
    }
}
