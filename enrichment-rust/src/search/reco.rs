use serde::{Deserialize, Serialize};

use super::super::serde_json_helpers::deserialize_f64;
use super::flight::Flight;

#[derive(Serialize, Deserialize, Debug)]
pub struct Reco {
    #[serde(deserialize_with = "deserialize_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub taxes: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub fees: f64,
    pub flights: Vec<Flight>,
}
