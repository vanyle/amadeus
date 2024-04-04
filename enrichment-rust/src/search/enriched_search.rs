use std::str::FromStr;

use serde::Serialize;
use strum_macros::EnumString;

use crate::{currency_exchange, neobase, serde_json_helpers::serialize_u64_optional_none_as_minus_one};

use super::{enriched_reco::EnrichedReco, typedefs::CountryCode, Search};

extern crate strum_macros;

#[derive(Serialize)]
pub enum TripType {
    OneWay,
    RoundTrip,
}

impl ToString for TripType {
    fn to_string(&self) -> String {
        match self {
            TripType::OneWay => "OW".to_string(),
            TripType::RoundTrip => "RT".to_string(),
        }
    }
}


#[derive(EnumString, Serialize)]
pub enum PassengerType {
    // Adult
    ADT,
    // Child
    CH,
}

#[derive(Serialize)]
pub struct Passenger {
    passenger_type: PassengerType,
    passenger_nb: u64,
}

#[derive(Serialize)]
pub enum GeoType {
    Domestic,
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
    pub fn enrich_from(search: &Search, neobase_locations: &neobase::Locations, exchange_rates: &currency_exchange::ExchangeRates) -> EnrichedSearch {
        let advance_purchase = (search.request_dep_date - search.search_date).num_days() as u64;

        let stay_duration = search.request_return_date.map(
            |return_date| (return_date - search.request_dep_date).num_days() as u64
        );

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
                let passenger_type = PassengerType::from_str(iter.next().unwrap()).unwrap();
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
            .map(|reco| EnrichedReco::enrich_from(reco, neobase_locations, &exchange_rates, &search.currency))
            .collect();

        return EnrichedSearch {
            recos,
            advance_purchase,
            stay_duration,
            trip_type,
            passengers,
            origin_country,
            destination_country,
            geo,
            ond_distance,
        };
    }
}