package main

import (
	"compress/gzip"
	"context"
	"encoding/csv"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math/rand"
	"os"
	"reflect"
	"strconv"
	"time"

	kafka "github.com/segmentio/kafka-go"
)

// General parameters
const OUTPUT_TOPIC = "raw_recos"

var KAFKA_URL = "localhost:1234"

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

type Reco struct {
	version_nb        string
	search_id         string
	search_country    string
	search_date       string
	search_time       string
	origin_city       string
	destination_city  string
	request_dep_date  string
	request_ret_date  string
	passengers_string string
	currency          string
	price             string
	taxes             string
	fees              string
	nb_of_flights     int // set to -1 to indicate an error
	flights           []Flight
}

var RecoColumnCount = reflect.TypeFor[Reco]().NumField()
var FlightColumnCount = reflect.TypeFor[Flight]().NumField()

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

func SendSearch(conn *kafka.Conn, search *Search) {

	msg, err := json.Marshal(search)
	if err != nil {
		log.Println("Failed to marshal search")
		log.Fatal(err)
	}

	if conn != nil {
		conn.WriteMessages(
			kafka.Message{Value: msg},
		)
	}
}

func RandInt(min int, max int) int {
	return min + rand.Intn(max-min)
}

func MakeSearchMoreRealistic(search *Search) {
	search.Search_date = time.Now().Format("2006-01-02")
	search.Search_time = time.Now().Format("15:04:05")

	defaultDepatureDate, _ := time.Parse("2006-01-02", search.Request_dep_date)
	defaultReturnDate, _ := time.Parse("2006-01-02", search.Request_ret_date)
	stayDuration := defaultReturnDate.Sub(defaultDepatureDate)

	departureDate := time.Now().Add(time.Hour * time.Duration(24*RandInt(3, 100)))
	search.Request_dep_date = departureDate.Format("2006-01-02")

	if search.Request_ret_date != "" {
		returnDate := departureDate.Add(stayDuration)
		search.Request_ret_date = returnDate.Format("2006-01-02")
	}

	for _, reco := range search.Recos {
		for _, flight := range reco.Flights {
			flightTime := departureDate.Add(time.Hour * time.Duration(24*RandInt(-2, 2)))
			flight.Dep_date = flightTime.Format("2006-01-02")
			flight.Arr_date = flightTime.Format("2006-01-02")
		}
	}

}

func GroupAndDecorate(reco []Reco) Search {
	var search Search

	search.Version_nb = reco[0].version_nb
	search.Search_id = reco[0].search_id
	search.Search_country = reco[0].search_country
	search.Search_date = reco[0].search_date
	search.Search_time = reco[0].search_time
	search.Origin_city = reco[0].origin_city
	search.Destination_city = reco[0].destination_city
	search.Request_dep_date = reco[0].request_dep_date
	search.Request_ret_date = reco[0].request_ret_date
	search.Passengers_string = reco[0].passengers_string
	search.Currency = reco[0].currency

	search.Recos = make([]CompressedReco, len(reco))
	for i, r := range reco {
		var cr CompressedReco
		cr.Price, _ = strconv.ParseFloat(r.price, 32)
		cr.Taxes, _ = strconv.ParseFloat(r.taxes, 32)
		cr.Fees, _ = strconv.ParseFloat(r.fees, 32)
		cr.Nb_of_flights = r.nb_of_flights
		cr.Flights = r.flights

		for _, f := range cr.Flights {
			if f.Operating_airline == "" {
				f.Operating_airline = f.Marketing_airline
			}
		}
		search.Recos[i] = cr
	}

	return search
}

func DecodeLine(line []string) Reco {
	var reco Reco

	reco.version_nb = line[0]
	reco.search_id = line[1]
	reco.search_country = line[2]
	reco.search_date = line[3]
	reco.search_time = line[4]
	reco.origin_city = line[5]
	reco.destination_city = line[6]
	reco.request_dep_date = line[7]
	reco.request_ret_date = line[8]
	reco.passengers_string = line[9]
	reco.currency = line[10]
	reco.price = line[11]
	reco.taxes = line[12]
	reco.fees = line[13]

	columnsRead := RecoColumnCount - 1

	nb_of_flights, err := strconv.ParseFloat(line[14], 32)
	if err != nil {
		reco.nb_of_flights = -1
		return reco
	}
	reco.nb_of_flights = int(nb_of_flights)
	reco.flights = make([]Flight, reco.nb_of_flights)

	for i := 0; i < reco.nb_of_flights; i++ {
		var flight Flight
		flight.Dep_airport = line[columnsRead]
		flight.Dep_date = line[columnsRead+1]
		flight.Dep_time = line[columnsRead+2]
		flight.Arr_airport = line[columnsRead+3]
		flight.Arr_date = line[columnsRead+4]
		flight.Arr_time = line[columnsRead+5]
		flight.Operating_airline = line[columnsRead+6]
		flight.Marketing_airline = line[columnsRead+7]
		flight.Flight_nb = line[columnsRead+8]
		flight.Cabin = line[columnsRead+9]
		reco.flights[i] = flight
		columnsRead += FlightColumnCount
	}

	return reco
}

func main() {
	fmt.Println("Starting stream...")
	f, err := os.Open("./travel_data_sample.csv.gz")
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()
	gr, err := gzip.NewReader(f)
	if err != nil {
		log.Fatal(err)
	}

	conn, err := kafka.DialLeader(context.Background(), "tcp", KAFKA_URL, OUTPUT_TOPIC, 0)
	if err != nil {
		log.Println("failed to dial leader:", err)
		log.Println("No brokers available")
		log.Println(
			"Make sure to start Kafka and specify the correct URL inside KAFKA_URL",
		)
		log.Println("Starting stream anyway...")
	} else {
		conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
		defer conn.Close()
	}

	cr := csv.NewReader(gr)
	cr.Comma = '^'
	cr.Read() // skip header

	var reco_read = 0
	var reco_decoded = 0
	var search_read = 0
	var search_encoded = 0

	var recoBuffer []Reco = nil
	var currentSearchID string

	for {
		rec, err := cr.Read()
		if err == io.EOF || len(rec) <= RecoColumnCount-1 {
			// Reset cr.
			gr.Close()
			f.Seek(0, 0)
			gr, _ = gzip.NewReader(f)
			cr = csv.NewReader(gr)
			cr.Comma = '^'
			cr.Read() // skip header
			log.Println("Looping!!")
			continue
		}

		reco_read++
		reco := DecodeLine(rec)
		reco_decoded++

		if reco.nb_of_flights == -1 {
			continue
		}
		if reco.search_id != currentSearchID {
			currentSearchID = reco.search_id
			if len(recoBuffer) > 0 {
				search_read++
				var search = GroupAndDecorate(recoBuffer)
				MakeSearchMoreRealistic(&search)
				search_encoded++
				SendSearch(conn, &search)
				recoBuffer = nil

				if search_encoded%1000 == 0 {
					log.Printf("Read %d reco, decoded %d reco, read %d search, encoded %d search\n", reco_read, reco_decoded, search_read, search_encoded)
				}
			}
		}
		recoBuffer = append(recoBuffer, reco)

	}

}
