package main

import (
	"compress/gzip"
	"encoding/csv"
	"fmt"
	"log"
	"os"
)

func main() {
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

	cr := csv.NewReader(gr)
	rec, err := cr.Read()
	if err != nil {
		log.Fatal(err)
	}
	for _, v := range rec {
		fmt.Println(v)
		break
	}
}
