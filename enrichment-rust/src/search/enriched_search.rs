use serde::{Deserialize, Serialize};

use crate::{
    currency_exchange, neobase, serde_json_helpers::serialize_u64_optional_none_as_minus_one,
};

use super::{
    enriched_reco::{EnrichRecoError, EnrichedReco},
    typedefs::CountryCode,
    Search,
};

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

#[derive(Debug, thiserror::Error)]
pub enum ParsePassengerError {
    #[error("No passenger type provided: {0:?}")]
    NoPassengerType(String),
    #[error("No passenger number provided: {0:?}")]
    NoPassengerNumber(String),
    #[error("Failed to parse passenger type string. Expected ADT or CH. Received: {0:?}. {1:?}")]
    FailedToParsePassengerTypeString(String, #[source] serde_json::Error),
    #[error("Failed to parse passenger number. {0:?}")]
    FailedToParsePassengerNumber(#[source] std::num::ParseIntError),
}

pub fn parse_one_passenger_string(
    passengers_string: &str,
) -> Result<Passenger, ParsePassengerError> {
    let mut iter = passengers_string.split('=');

    let passenger_type_val = serde_json::to_value(iter.next().ok_or(
        ParsePassengerError::NoPassengerType(passengers_string.to_string()),
    )?)
    .map_err(|e| {
        ParsePassengerError::FailedToParsePassengerTypeString(passengers_string.to_string(), e)
    })?;

    let passenger_type = serde_json::from_value(passenger_type_val).map_err(|e| {
        ParsePassengerError::FailedToParsePassengerTypeString(passengers_string.to_string(), e)
    })?;

    let passenger_nb = iter
        .next()
        .ok_or(ParsePassengerError::NoPassengerNumber(
            passengers_string.to_string(),
        ))?
        .parse::<u64>()
        .map_err(ParsePassengerError::FailedToParsePassengerNumber)?;

    Ok(Passenger {
        passenger_type,
        passenger_nb,
    })
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

#[derive(Debug, thiserror::Error)]
pub enum EnrichSearchError {
    #[error("Enriching reco failed: {0:?}")]
    EnrichReco(#[source] EnrichRecoError),
    #[error("Search date is after request departure date. Cannot compute advance purchase. {0:?}")]
    SearchDateAfterRequestDepDate(#[source] std::num::TryFromIntError),
    #[error(
        "Request departure date is after request return date. Cannot compute stay duration. {0:?}"
    )]
    RequestDepDateAfterRequestReturnDate(#[source] std::num::TryFromIntError),
    #[error("Failed to parse passengers string: {0:?}")]
    FailedToParsePassengersString(#[source] ParsePassengerError),
    #[error("Missing location in distance calculation")]
    MissingLocationInDistanceCalculation {
        origin_city: String,
        destination_city: String,
    },
}

impl EnrichedSearch {
    pub fn enrich_from(
        search: &Search,
        neobase_locations: &neobase::Locations,
        exchange_rates: &currency_exchange::ExchangeRates,
    ) -> Result<EnrichedSearch, EnrichSearchError> {
        let advance_purchase =
            u64::try_from((search.request_dep_date - search.search_date).num_days())
                .map_err(EnrichSearchError::SearchDateAfterRequestDepDate)?;

        let stay_duration = search
            .request_return_date
            .map(|return_date| u64::try_from((return_date - search.request_dep_date).num_days()))
            .transpose()
            .map_err(EnrichSearchError::RequestDepDateAfterRequestReturnDate)?;

        let trip_type = match &search.request_return_date {
            Some(_) => TripType::RoundTrip,
            None => TripType::OneWay,
        };

        // decoding passengers string: "ADT=1,CH=2" means 1 Adult and 2 children
        let passengers = search
            .passengers_string
            .split(',')
            .map(parse_one_passenger_string)
            .collect::<Result<Vec<Passenger>, ParsePassengerError>>()
            .map_err(EnrichSearchError::FailedToParsePassengersString)?;

        let origin_country = neobase_locations.get_country_from_city(&search.origin_city);
        let destination_country = neobase_locations.get_country_from_city(&search.destination_city);

        let geo = if origin_country == destination_country {
            Some(GeoType::Domestic)
        } else {
            Some(GeoType::International)
        };

        let ond_distance = neobase_locations
            .get_round_distance_between_locations(&search.origin_city, &search.destination_city)
            .ok_or(EnrichSearchError::MissingLocationInDistanceCalculation {
                origin_city: search.origin_city.clone(),
                destination_city: search.destination_city.clone(),
            })?;

        let recos = search
            .recos
            .iter()
            .map(|reco| {
                EnrichedReco::enrich_from(reco, neobase_locations, exchange_rates, &search.currency)
            })
            .collect::<Result<Vec<EnrichedReco>, EnrichRecoError>>()
            .map_err(EnrichSearchError::EnrichReco)?;

        Ok(EnrichedSearch {
            recos,
            advance_purchase,
            stay_duration,
            trip_type,
            passengers,
            origin_country,
            destination_country,
            geo,
            ond_distance,
        })
    }
}
