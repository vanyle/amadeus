package main

import (
	"compress/gzip"
	"context"
	"encoding/csv"
	"fmt"
	"io"
	"log"
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
	dep_airport       string
	dep_date          string
	dep_time          string
	arr_airport       string
	arr_date          string
	arr_time          string
	operating_airline string
	marketing_airline string
	flight_nb         string
	cabin             string
}

type Reco struct {
	version_nb        string
	search_id         string
	search_country    string
	search_date       string
	origin_city       string
	destination_city  string
	request_dep_date  string
	request_ret_date  string
	passengers_string string
	currency          string
	price             string
	taxes             string
	fees              string
	nb_of_flights     int
	flights           []Flight
}

var RecoColumnCount = reflect.TypeFor[Reco]().NumField()
var FlightColumnCount = reflect.TypeFor[Flight]().NumField()

func SendMessage(conn *kafka.Conn, msg string) {
	conn.WriteMessages(
		kafka.Message{Value: []byte(msg)},
	)
}

func DecodeLine(line []string) Reco {
	var reco Reco

	reco.version_nb = line[0]
	reco.search_id = line[1]
	reco.search_country = line[2]
	reco.search_date = line[3]
	reco.origin_city = line[4]
	reco.destination_city = line[5]
	reco.request_dep_date = line[6]
	reco.request_ret_date = line[7]
	reco.passengers_string = line[8]
	reco.currency = line[9]
	reco.price = line[10]
	reco.taxes = line[11]
	reco.fees = line[12]

	columnsRead := RecoColumnCount

	nb_of_flights, err := strconv.ParseFloat(line[13], 32)
	if err != nil {
		log.Fatal(err)
	}
	reco.nb_of_flights = int(nb_of_flights)
	reco.flights = make([]Flight, reco.nb_of_flights)

	for i := 0; i < reco.nb_of_flights; i++ {
		var flight Flight
		flight.dep_airport = line[columnsRead]
		flight.dep_date = line[columnsRead+1]
		flight.dep_time = line[columnsRead+2]
		flight.arr_airport = line[columnsRead+3]
		flight.arr_date = line[columnsRead+4]
		flight.arr_time = line[columnsRead+5]
		flight.operating_airline = line[columnsRead+6]
		flight.marketing_airline = line[columnsRead+7]
		flight.flight_nb = line[columnsRead+8]
		flight.cabin = line[columnsRead+9]
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
	defer gr.Close()

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

	var c = 0
	for {
		rec, err := cr.Read()
		if err == io.EOF {
			break
		}
		reco := DecodeLine(rec)
		fmt.Println(reco)

		c++
		if c > 10 {
			break
		}
	}
}
