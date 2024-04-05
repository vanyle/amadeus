use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::serde_json_helpers::ymd_date_format;
use crate::{currency_exchange::Currency, serde_json_helpers::ymd_date_format_optional};

use self::{reco::Reco, typedefs::CityCode};

pub mod enriched_flight;
pub mod enriched_reco;
pub mod enriched_search;
pub mod flight;
pub mod reco;
pub mod typedefs;

#[derive(Serialize, Deserialize)]
pub struct Search {
    pub currency: Currency,
    #[serde(with = "ymd_date_format")]
    pub search_date: NaiveDate,
    #[serde(with = "ymd_date_format")]
    pub request_dep_date: NaiveDate,
    #[serde(with = "ymd_date_format_optional")]
    pub request_return_date: Option<NaiveDate>,
    pub passengers_string: String,
    pub origin_city: CityCode,
    pub destination_city: CityCode,
    pub recos: Vec<Reco>,
}
