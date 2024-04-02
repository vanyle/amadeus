mod currency_exchange;
mod neobase;

use chrono::NaiveDate;
use currency_exchange::Currency;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use std::str::FromStr;

extern crate strum_macros;

#[derive(EnumString)]
enum PassengerType {
    // Adult
    ADT,
    // Child
    CH,
}

struct Passenger {
    passenger_type: PassengerType,
    passenger_nb: u32,
}

type CityCode = String;

type AirportCode = String;

type DateString = String;

#[derive(Serialize, Deserialize)]
struct Flight {
    dep_airport: AirportCode,
    arr_airport: AirportCode,
    marketing_airline: String,
    operating_airline: Option<String>,
    cabin: String,
}

#[derive(Serialize)]
struct EnrichedFlight {
    dep_airport: AirportCode,
    arr_airport: AirportCode,
    marketing_airline: String,
    // operating_airline: Option<String>, // overriden
    cabin: String,
    // Enriched
    dep_city: CityCode,
    arr_city: CityCode,
    distance: u32,
    operating_airline: String,
}

impl EnrichedFlight {
    fn enrich_from(flight: &Flight, neobase_locations: &neobase::Locations) -> EnrichedFlight {
        let dep_city = neobase_locations.get_city_from_location(&flight.dep_airport);
        let arr_city = neobase_locations.get_city_from_location(&flight.arr_airport);
        // TODO: handle error
        let distance = neobase_locations
            .get_round_distance_between_locations(&flight.dep_airport, &flight.arr_airport)
            .unwrap();

        EnrichedFlight {
            dep_airport: flight.dep_airport.clone(),
            arr_airport: flight.arr_airport.clone(),
            marketing_airline: flight.marketing_airline.clone(),
            cabin: flight.cabin.clone(),
            dep_city,
            arr_city,
            distance,
            operating_airline: flight
                .operating_airline
                .clone()
                .unwrap_or(flight.marketing_airline.clone()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Reco {
    price: f32,
    taxes: f32,
    fees: f32,
    flights: Vec<Flight>,
}

#[derive(Serialize)]
struct EnrichedReco {
    price: f32,
    taxes: f32,
    fees: f32,
    // flights: Vec<Flight>, // overriden
    // Enriched
    #[serde(rename = "price_EUR")]
    price_eur: f64,
    #[serde(rename = "taxes_EUR")]
    taxes_eur: f64,
    #[serde(rename = "fees_EUR")]
    fees_eur: f64,
    flights: Vec<EnrichedFlight>,
    flown_distance: u32,
    main_marketing_airline: String,
    main_operating_airline: String,
    main_cabin: String,
}

impl EnrichedReco {
    fn enrich_from(
        reco: &Reco,
        neobase_locations: &neobase::Locations,
        exchange_rates: &currency_exchange::ExchangeRates,
        currency: &Currency,
    ) -> EnrichedReco {
        let price_eur = exchange_rates.to_euros(reco.price as f64, currency);
        let taxes_eur = exchange_rates.to_euros(reco.taxes as f64, currency);
        let fees_eur = exchange_rates.to_euros(reco.fees as f64, currency);

        let flights: Vec<EnrichedFlight> = reco
            .flights
            .iter()
            .map(|flight| EnrichedFlight::enrich_from(flight, neobase_locations))
            .collect();

        let flown_distance: u32 = flights.iter().map(|flight| flight.distance).sum();
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

        return EnrichedReco {
            price: reco.price,
            taxes: reco.taxes,
            fees: reco.fees,
            flights,
            price_eur,
            taxes_eur,
            fees_eur,
            flown_distance,
            main_marketing_airline,
            main_operating_airline,
            main_cabin,
        };
    }
}

#[derive(Deserialize)]
struct Search {
    currency: Currency,
    search_date: DateString,
    request_dep_date: DateString,
    request_return_date: Option<DateString>,
    passengers_string: String,
    origin_city: CityCode,
    destination_city: CityCode,
    recos: Vec<Reco>,
}

#[derive(Serialize)]
enum TripType {
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

// 3-letter coutry code
type CountryCode = String;

enum GeoType {
    Domestic,
    International,
}

impl ToString for GeoType {
    fn to_string(&self) -> String {
        match self {
            GeoType::Domestic => "D".to_string(),
            GeoType::International => "I".to_string(),
        }
    }
}

struct EnrichedSearch {
    currency: Currency,
    search_date: DateString,
    request_dep_date: DateString,
    request_return_date: Option<DateString>,
    passengers_string: String,
    origin_city: CityCode,
    destination_city: CityCode,
    // recos: Vec<Reco>, // overriden
    // Enriched
    recos: Vec<EnrichedReco>,
    advance_purchase: u32,
    stay_duration: i32, // -1 if no return date // TODO: use Option<u32> with special serde feature ?
    trip_type: TripType,
    passengers: Vec<Passenger>,
    origin_country: CountryCode,
    destination_country: CountryCode,
    geo: Option<GeoType>,
    //#[serde(rename = "OnD_distance")]
    ond_distance: u32,
}

impl EnrichedSearch {
    fn enrich_from(search: &Search, neobase_locations: &neobase::Locations, exchange_rates: &currency_exchange::ExchangeRates) -> EnrichedSearch {
        let parsed_request_dep_date =
            NaiveDate::parse_from_str(&search.request_dep_date, "%Y-%m-%d")
                .expect("Failed to parse request_dep_date");
        let parsed_search_date = NaiveDate::parse_from_str(&search.search_date, "%Y-%m-%d")
            .expect("Failed to parse search_date");

        let advance_purchase = (parsed_request_dep_date - parsed_search_date).num_days() as u32;

        let stay_duration = match &search.request_return_date {
            Some(request_return_date) => {
                let parsed_request_return_date =
                    NaiveDate::parse_from_str(request_return_date, "%Y-%m-%d")
                        .expect("Failed to parse request_return_date");
                (parsed_request_return_date - parsed_request_dep_date).num_days() as i32
            }
            None => -1,
        };

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
                let passenger_nb = iter.next().unwrap().parse::<u32>().unwrap();
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
            .unwrap();

        let recos = search
            .recos
            .iter()
            .map(|reco| EnrichedReco::enrich_from(reco, neobase_locations, &exchange_rates, &search.currency))
            .collect();

        return EnrichedSearch {
            currency: search.currency,
            search_date: search.search_date.clone(),
            request_dep_date: search.request_dep_date.clone(),
            request_return_date: search.request_return_date.clone(),
            passengers_string: search.passengers_string.clone(),
            origin_city: search.origin_city.clone(),
            destination_city: search.destination_city.clone(),
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

fn main() {
    println!("Hello, world!");

    // TODO : read json
    // cast json to search, leaving unused fields untouched
    // enrich
    // create json from enriched search, adding old untouched fields
}
