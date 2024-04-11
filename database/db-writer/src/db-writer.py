#!/usr/bin/env python3

"""
Tool to flatten data before ingesting into db

> python3 db-writer.py -h

"""

import argparse
import json
import logging
import time
from collections import Counter

from confluent_kafka import OFFSET_BEGINNING, Consumer, Producer

# Log init
logging.basicConfig(
    format="%(levelname)-8s [%(filename)s:%(lineno)d] %(message)s",
    datefmt="%Y-%m-%d:%H:%M:%S",
    level=logging.INFO,
)
logger = logging.getLogger()


# Encoders
# Different ways to print the output. You can easily add one.
def encoder_json(search):
    return json.dumps(search)


def encoder_pretty_json(search):
    return json.dumps(search, indent=2)


def encoder_summary(search):
    return search["search_id"]


# encoders list
encoders = {
    "json": encoder_json,
    "pretty_json": encoder_pretty_json,
    "summary": encoder_summary,
}

kafka_config = {
    "bootstrap.servers": "127.0.0.1:9092",
    "group.id": "maingroup",
    "auto.offset.reset": "earliest",  # 'auto.offset.reset=earliest' to start reading from the beginning of the topic if no committed offsets exist
    "queue.buffering.max.messages": 10000000,  # 10M messages : dirty hack because of `BufferError: Local: Queue full`
}

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Travel data flattener")
    parser.add_argument(
        "-f",
        "--format",
        help="Desired output format. Default is summary.",
        choices=encoders.keys(),
        default="summary",
    )
    parser.add_argument(
        "--reset",
        action="store_true",
        help="Make db-writer process kafka topic from the beginning",
    )
    arguments = parser.parse_args()

    # start time
    start = time.time()

    # Create Consumer instance
    kafka_consumer = Consumer(kafka_config)

    # Set up a callback to handle the '--reset' flag.
    def reset_offset(kafka_consumer, partitions):
        if arguments.reset:
            for p in partitions:
                p.offset = OFFSET_BEGINNING
            kafka_consumer.assign(partitions)

    # Subscribe to topic
    consumer_topic = "enriched_recos"
    kafka_consumer.subscribe([consumer_topic], on_assign=reset_offset)

    # Create Producer instance
    kafka_producer = Producer(kafka_config)

    # Optional per-message delivery callback (triggered by poll() or flush())
    # when a message has been successfully delivered or permanently
    # failed delivery (after retries).
    def delivery_callback(err, msg):
        if err:
            logger.debug("ERROR: Message failed delivery: {}".format(err))
        else:
            logger.debug(
                "Produced event to topic {topic}: key = {key:12} value = {value:12}".format(
                    topic=msg.topic(),
                    key=msg.key().decode("utf-8"),
                    value=msg.value().decode("utf-8"),
                )
            )

    topic_recos = "db_recos"
    topic_searches = "db_searches"

    # Create encoder
    encoder = encoders[arguments.format]

    # Poll for new messages from Kafka and write them into DB
    logger.info("Flattening")
    cnt = Counter()
    # Poll for new messages from Kafka and split them
    try:
        while True:
            msg = kafka_consumer.poll(5.0)
            if msg is None:
                # Initial message consumption may take up to
                # `session.timeout.ms` for the kafka_consumer group to
                # rebalance and start consuming
                logger.info("Waiting...")
            elif msg.error():
                logger.debug("ERROR: %s".format(msg.error()))
            else:
                logger.debug(
                    "Consumed event from topic {topic}: key = {key:12} value = {value:12}".format(
                        topic=msg.topic(),
                        key=msg.key().decode("utf-8"),
                        value=msg.value().decode("utf-8"),
                    )
                )

                cnt["search_read"] += 1
                search = json.loads(msg.value().decode("utf-8"))
                logger.debug(f"Flatten : %s" % encoder(search))
                kafka_producer.produce(
                    topic_searches,
                    json.dumps(
                        {
                            key: search[key]
                            for key in [
                                "search_id",
                                "search_country",
                                "search_date",
                                "request_dep_date",
                                "advance_purchase",
                                "stay_duration",
                                "trip_type",
                                "OnD",
                            ]
                        }
                    ).encode("utf-8"),
                    search["search_id"],
                    callback=delivery_callback,
                )
                cnt["search_written"] += 1
                for reco in search["recos"]:
                    cnt["recos_read"] += 1
                    reco_dict = {
                        key: reco[key]
                        for key in [
                            "nb_of_flights",
                            "price_EUR",
                            "main_marketing_airline",
                            "main_operating_airline",
                        ]
                    }
                    reco_dict["search_id"] = search["search_id"]
                    kafka_producer.produce(
                        topic_recos,
                        json.dumps(reco_dict).encode("utf-8"),
                        search["search_id"],
                        callback=delivery_callback,
                    )
                    cnt["recos_written"] += 1
                    if cnt["recos_read"] % 10000 == 0:
                        # log every 10000 searches to show the script is alive
                        logger.info(f"Running: %s" % cnt)

                # Only needed so that delivery_callback is called
                kafka_producer.poll(0)

    except KeyboardInterrupt:  # TODO : catch other exceptions
        pass
    finally:
        # Leave group and commit final offsets
        kafka_consumer.close()
        # Block until the messages are sent.
        kafka_producer.flush()
        # end time
        end = time.time()

        logger.info(f"Finished in {round(end - start, 2)} seconds: %s" % cnt)
