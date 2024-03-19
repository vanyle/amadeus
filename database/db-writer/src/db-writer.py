#!/usr/bin/env python3

"""
Tool to insert travel recommendations into DB

> python3 db-writer.py -h

"""

import logging
import argparse
import datetime
import sys
import json
import time
import os
import psycopg2
from collections import Counter
from configparser import ConfigParser
from confluent_kafka import Producer, Consumer, OFFSET_BEGINNING

# Log init
logging.basicConfig(
    format="%(levelname)-8s [%(filename)s:%(lineno)d] %(message)s",
    datefmt="%Y-%m-%d:%H:%M:%S",
    level=logging.DEBUG,
)
logger = logging.getLogger()
logger.setLevel(logging.DEBUG)


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
}

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Travel data enricher")
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

    # Create encoder
    encoder = encoders[arguments.format]

    # Poll for new messages from Kafka and write them into DB
    logger.info("Writing into DB")
    cnt = Counter()

    # Connect to an existing database
    conn = psycopg2.connect(
        dbname="postgres", user="postgres", password="password", host="127.0.0.1"
    )

    # Open a cursor to perform database operations
    cur = conn.cursor()
    try:
        while True:
            msg = kafka_consumer.poll(5.0)
            if msg is None:
                # Initial message consumption may take up to
                # `session.timeout.ms` for the kafka_consumer group to
                # rebalance and start consuming
                logger.debug("Waiting...")
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

                # Pass data to fill a query placeholders and let Psycopg perform
                # the correct conversion (no more SQL injections!)
                cur.execute(
                    "INSERT INTO searches (search_id, search_country, search_date, request_dep_date, advance_purchase, stay_duration, trip_type, ond) VALUES (%s, %s, %s, %s, %s, %s, %s, %s)",
                    (
                        search["search_id"],
                        search["search_country"],
                        search["search_date"],
                        search["request_dep_date"],
                        search["advance_purchase"],
                        search["stay_duration"],
                        search["trip_type"],
                        search["OnD"],
                    ),
                )
                cnt["search_written"] += 1
                for reco in search["recos"]:
                    cnt["recos_read"] += 1
                    cur.execute(
                        "INSERT INTO recos (search_id, nb_of_flights, price_EUR, main_marketing_airline, main_operating_airline) VALUES (%s, %s, %s, %s, %s)",
                        (
                            search["search_id"],
                            reco["nb_of_flights"],
                            reco["price_EUR"],
                            reco["main_marketing_airline"],
                            reco["main_operating_airline"],
                        ),
                    )
                    cnt["recos_written"] += 1

                # Make the changes to the database persistent
                conn.commit()

                # encode and print
                logger.info(f"Wrote search: %s" % encoder(search))

    except KeyboardInterrupt:  # TODO : catch other exceptions
        pass
    except KeyError:
        logger.exception("Failed at writing search from: %s" % search)
    finally:
        # Close communication with the database
        cur.close()
        conn.close()
        # Leave group and commit final offsets
        kafka_consumer.close()
        # end time
        end = time.time()

        logger.info(f"Finished in {round(end - start, 2)} seconds: %s" % cnt)
