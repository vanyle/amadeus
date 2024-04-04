# Fake stream generator

This python scripts generate a stream of fake travel searches and sends them as messages
to a kafka instance.

Because I'm too poor to afford Git LFS, the data is stored as a 7z file. You need to decompress this file
first and recompress it as a `.tar.gz` file using `tar -czvf travel_data_sample.tar.gz travel_data_sample.csv` to
get something usable

You can configure the script using the following environment variables:

- `MSG_PER_SEC`: (default: 1000), Number of simulated searches per second. Set to -1 to remove limits.
- `KAFKA_URL`: (default: localhost:1234), URL of the kafka instance to connect to.

EC2 = 8 euros par mois. As soon as an instance is terminated, you no longer incur costs for it.

A search might look something like this:

```json
{
  "version_nb": "1.0",
  "search_id": "urn:uuid:f8301604-e716-4f9f-ac4b-ded68f93dbab",
  "search_country": "FR",
  "search_date": "2024-03-15",
  "search_time": "23:52:02",
  "origin_city": "PAR",
  "destination_city": "RUN",
  "request_dep_date": "2024-03-31",
  "request_return_date": "",
  "passengers_string": "ADT=1",
  "currency": "EUR",
  "recos": [
    {
      "price": 327.36,
      "taxes": 208.59,
      "fees": 0.0,
      "nb_of_flights": 1,
      "flights": [
        {
          "dep_airport": "ORY",
          "dep_date": "2024-03-31",
          "dep_time": "16:20",
          "arr_airport": "RUN",
          "arr_date": "2024-03-31",
          "arr_time": "06:45",
          "operating_airline": "SS",
          "marketing_airline": "SS",
          "flight_nb": "904",
          "cabin": "M"
        }
      ]
    },
    {
      "price": 1549.4,
      "taxes": 208.52,
      "fees": 0.0,
      "nb_of_flights": 2,
      "flights": [
        {
          "dep_airport": "CDG",
          "dep_date": "2024-04-02",
          "dep_time": "16:20",
          "arr_airport": "MRU",
          "arr_date": "2024-04-02",
          "arr_time": "06:30",
          "operating_airline": "MK",
          "marketing_airline": "MK",
          "flight_nb": "15",
          "cabin": "M"
        },
        {
          "dep_airport": "MRU",
          "dep_date": "2024-03-29",
          "dep_time": "10:05",
          "arr_airport": "RUN",
          "arr_date": "2024-03-29",
          "arr_time": "10:50",
          "operating_airline": "MK",
          "marketing_airline": "MK",
          "flight_nb": "218",
          "cabin": "M"
        }
      ]
    }
  ],
  "stay_duration": -1,
  "trip_type": "OW"
}
```
