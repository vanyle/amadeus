use serde::{Deserialize, Serialize};

use super::typedefs::AirportCode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Flight {
    pub dep_airport: AirportCode,
    pub arr_airport: AirportCode,
    pub marketing_airline: String,
    pub operating_airline: Option<String>,
    pub cabin: String,
}
