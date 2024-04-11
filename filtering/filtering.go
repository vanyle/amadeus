package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"math/rand"
	"time"


	kafka "github.com/segmentio/kafka-go"
)

var input_topic = "raw_recos"
var output_topic = "filtered_recos"
var KAFKA_URL = "localhost:1234"

func SetupEnvVars() {
	ku, found := os.LookupEnv("KAFKA_URL")
	if found {
		KAFKA_URL = ku
	}
}

func ShouldSearchBeFilteredOut(s *Search) bool {
	source := rand.NewSource(time.Now().UnixNano())
	rng := rand.New(source)

	return rng.Intn(100) >= 99
}

func FilterSearches(writer *kafka.Writer, search *Search, raw_search []byte) {
	if ShouldSearchBeFilteredOut(search) {
		return
	}

	writer.WriteMessages(context.Background(),
		kafka.Message{Value: raw_search},
	)
}

func main() {
	log.Println("Starting filtering service ...")
	SetupEnvVars()

	var msgCounter int = 0
	var malformedMsgCounter int = 0

	r := kafka.NewReader(kafka.ReaderConfig{
		Brokers:   []string{KAFKA_URL},
		Topic:     input_topic,
		Partition: 0,
		MaxBytes:  10e6, // 10MB
	})
	defer r.Close()

	// Writer autoretries on failure unlike kafka.DialLeader.
	w := &kafka.Writer{
		Addr:  kafka.TCP(KAFKA_URL),
		Topic: output_topic,
		// Balancer: &kafka.LeastBytes{},
		// AllowAutoTopicCreation: true,
	}
	defer w.Close()

	for {
		b, err := r.ReadMessage(context.Background())
		if err != nil {
			break
		}
		search := Search{}
		err = json.Unmarshal(b.Value, &search)
		msgCounter++
		if err != nil {
			malformedMsgCounter++
			continue // bad json is ignored
		}
		FilterSearches(w, &search, b.Value)

		if msgCounter%1000 == 0 {
			log.Printf("Received %d messages, %d malformed\n", msgCounter, malformedMsgCounter)
		}
	}
}
