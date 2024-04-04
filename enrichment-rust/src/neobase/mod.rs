use std::{collections::HashMap, fs::File};

#[derive(Debug, serde::Deserialize)]
struct Record {
    iata_code: String,
    icao_code: String,
    faa_code: String,
    is_geonames: String,
    geoname_id: String,
    envelope_id: String,
    name: String,
    asciiname: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    fclass: String,
    fcode: String,
    page_rank: String,
    date_from: String,
    date_until: String,
    comment: String,
    country_code: String,
    cc2: String,
    country_name: String,
    continent_name: String,
    adm1_code: String,
    adm1_name_utf: String,
    adm1_name_ascii: String,
    adm2_code: String,
    adm2_name_utf: String,
    adm2_name_ascii: String,
    adm3_code: String,
    adm4_code: String,
    population: String,
    elevation: String,
    gtopo30: String,
    timezone: String,
    gmt_offset: String,
    dst_offset: String,
    raw_offset: String,
    moddate: String,
    city_code_list: String,
    city_name_list: String,
    city_detail_list: String,
    tvl_por_list: String,
    iso31662: String,
    location_type: String,
    wiki_link: String,
    alt_name_section: String,
    wac: String,
    wac_name: String,
    ccy_code: String,
    unlc_list: String,
    uic_list: String,
    geoname_lat: String,
    geoname_lon: String,
}

struct Location {
    lat: Option<f64>,
    lng: Option<f64>,
    country_code: String,
    city_code_list: Vec<String>,
}

fn get_geodata(filepath: &str) -> HashMap<String, Location> {
    let csv_file = File::open(filepath).expect("File not found");
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'^')
        .from_reader(csv_file);

    let mut airports = HashMap::new();

    for result in csv_reader.deserialize() {
        let record: Record = result.expect("Error parsing record");
        let airport = Location {
            lat: record.latitude,
            lng: record.longitude,
            country_code: record.country_code,
            city_code_list: record
                .city_code_list
                .split(',')
                .map(|s| s.to_string())
                .collect(),
        };
        airports.insert(record.iata_code, airport);
    }

    airports
}
pub struct Locations {
    locations: HashMap<String, Location>,
}

impl Locations {
    pub fn new() -> Self {
        Locations {
            locations: get_geodata("src/neobase/data.csv"),
        }
    }

    pub fn get_country_from_city(&self, city: &str) -> String {
        match self.locations.get(city) {
            Some(loc) => loc.country_code.clone(),
            None => "".to_string(),
        }
    }

    pub fn get_city_from_location(&self, airport: &str) -> String {
        match self.locations.get(airport) {
            Some(loc) => loc.city_code_list[0].clone(),
            None => "".to_string(),
        }
    }

    pub fn get_round_distance_between_locations(
        &self,
        first_location: &str,
        second_location: &str,
    ) -> Option<u64> {
        let first_airport = self.locations.get(first_location)?;
        let second_airport = self.locations.get(second_location)?;

        let first_lat = first_airport.lat?;
        let first_lng = first_airport.lng?;
        let second_lat = second_airport.lat?;
        let second_lng = second_airport.lng?;

        let distance = haversine_distance(first_lat, first_lng, second_lat, second_lng);
        Some(distance)
    }
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> u64 {
    const R: f64 = 6371.0; // Radius of the Earth in km
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
        + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * (d_lon / 2.0).sin()
            * (d_lon / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    (R * c) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_geodata() {
        get_geodata("src/neobase/data.csv");
    }
}
