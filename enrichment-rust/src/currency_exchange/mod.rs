use csv::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(EnumString, Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum Currency {
    EUR,
    USD,
    JPY,
    BGN,
    CZK,
    DKK,
    GBP,
    HUF,
    PLN,
    RON,
    SEK,
    CHF,
    ISK,
    NOK,
    HRK,
    RUB,
    TRY,
    AUD,
    BRL,
    CAD,
    CNY,
    HKD,
    IDR,
    ILS,
    INR,
    KRW,
    MXN,
    MYR,
    NZD,
    PHP,
    SGD,
    THB,
    ZAR,
}

type Rate = f64;

pub struct ExchangeRates {
    rates: HashMap<Currency, Rate>,
    _record_date: String,
}

impl ExchangeRates {
    pub fn new() -> Self {
        let file = File::open("src/currency_exchange/eurofxref.csv").unwrap();
        let mut reader = Reader::from_reader(file);

        let mut rates = HashMap::new();

        let header = reader.headers().unwrap().clone();
        let header = header.into_iter().collect::<Vec<_>>();

        let last_record = reader.records().last().unwrap().unwrap();
        let mut last_record = last_record.iter().enumerate();

        let record_date = last_record.next().unwrap().1.to_string();

        for (currency, rate) in last_record {
            let currency_name = header[currency].trim();
            if currency_name.is_empty() {
                continue;
            }

            let currency = Currency::from_str(currency_name).unwrap();
            let rate = rate.trim().parse().unwrap();
            rates.insert(currency, rate);
        }

        ExchangeRates {
            rates,
            _record_date: record_date,
        }
    }

    pub fn to_euros(&self, amount: f64, currency: &Currency) -> f64 {
        match currency {
            Currency::EUR => amount,
            // TODO: handle missing rates
            _ => amount / self.rates.get(currency).unwrap(),
        }
    }
}

impl Default for ExchangeRates {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_euros_to_euros() {
        let exchange_rates = ExchangeRates::new();

        assert_eq!(exchange_rates.to_euros(100.0, &Currency::EUR), 100.0);
    }
}
