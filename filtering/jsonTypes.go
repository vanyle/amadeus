package main

type Flight struct {
	Dep_airport       string `json:"dep_airport"`
	Dep_date          string `json:"dep_date"`
	Dep_time          string `json:"dep_time"`
	Arr_airport       string `json:"arr_airport"`
	Arr_date          string `json:"arr_date"`
	Arr_time          string `json:"arr_time"`
	Operating_airline string `json:"operating_airline"`
	Marketing_airline string `json:"marketing_airline"`
	Flight_nb         string `json:"flight_nb"`
	Cabin             string `json:"cabin"`
}

type CompressedReco struct {
	Price         float64  `json:"price"`
	Taxes         float64  `json:"taxes"`
	Fees          float64  `json:"fees"`
	Nb_of_flights int      `json:"nb_of_flights"`
	Flights       []Flight `json:"flights"`
}

type Search struct {
	Version_nb        string `json:"version_nb"`
	Search_id         string `json:"search_id"`
	Search_country    string `json:"search_country"`
	Search_date       string `json:"search_date"`
	Search_time       string `json:"search_time"`
	Origin_city       string `json:"origin_city"`
	Destination_city  string `json:"destination_city"`
	Request_dep_date  string `json:"request_dep_date"`
	Request_ret_date  string `json:"request_return_date"`
	Passengers_string string `json:"passengers_string"`
	Currency          string `json:"currency"`

	Recos []CompressedReco `json:"recos"`
}
