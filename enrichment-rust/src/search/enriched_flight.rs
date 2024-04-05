use serde::Serialize;

use crate::neobase;

use super::{flight::Flight, typedefs::CityCode};

#[derive(Serialize)]
pub struct EnrichedFlight {
    // Enriched
    pub dep_city: CityCode,
    pub arr_city: CityCode,
    pub distance: u64,
    pub marketing_airline: String, // overriden (same value)
    pub operating_airline: String, // overriden
    pub cabin: String,             // overriden (same value)
}

#[derive(Debug, thiserror::Error)]
pub enum EnrichFlightError {
    #[error("Missing location in distance calculation")]
    MissingLocationInDistanceCalculation {
        dep_airport: String,
        arr_airport: String,
    },
}

impl EnrichedFlight {
    pub fn enrich_from(
        flight: &Flight,
        neobase_locations: &neobase::Locations,
    ) -> Result<EnrichedFlight, EnrichFlightError> {
        let dep_city = neobase_locations.get_city_from_location(&flight.dep_airport);
        let arr_city = neobase_locations.get_city_from_location(&flight.arr_airport);

        let distance = neobase_locations
            .get_round_distance_between_locations(&flight.dep_airport, &flight.arr_airport)
            .ok_or(EnrichFlightError::MissingLocationInDistanceCalculation {
                dep_airport: flight.dep_airport.clone(),
                arr_airport: flight.arr_airport.clone(),
            })?;

        let operating_airline = flight
            .operating_airline
            .clone()
            .unwrap_or(flight.marketing_airline.clone());

        Ok(EnrichedFlight {
            dep_city,
            arr_city,
            distance,
            marketing_airline: flight.marketing_airline.clone(),
            operating_airline,
            cabin: flight.cabin.clone(),
        })
    }
}
