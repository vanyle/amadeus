package main

import (
	"context"
	"fmt"

	kafka "github.com/segmentio/kafka-go"
)

var input_topic = "raw_recos"
var output_topic = "filtered"

func Main() {
    
	log.Default().("Starting filtering service ...")
	partition := 0

	conn, err := kafka.DialLeader(context.Background(), "tcp", "localhost:9092", input_topic, partition)
	if err != nil {
		panic(err)
	}

	defer conn.Close()
}
