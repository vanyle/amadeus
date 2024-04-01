#!/usr/bin/env python3

"""
Tool to enrich Travel recommendations.

> python3 enricher.py -h

"""

import argparse
import datetime
import json
import logging
import os
import time
from collections import Counter

import neobase
from confluent_kafka import OFFSET_BEGINNING, Consumer, Producer

# Log init
logging.basicConfig(
    format="%(levelname)-8s [%(filename)s:%(lineno)d] %(message)s",
    datefmt="%Y-%m-%d:%H:%M:%S",
    level=logging.DEBUG,
)
logger = logging.getLogger()
logger.setLevel(logging.DEBUG)

# geography module
neob = None


def get_neob() -> neobase.NeoBase:
    """
    Inits geography module if necessary
    :return: neob: neobase object to be used for geo conversions
    """
    global neob
    # geography module init
    if neob is None:
        logger.info("Init geography module neobase")
        neob = neobase.NeoBase()
    return neob


# Currency rates
def load_rates(rates_file):
    """
    Decodes currency rate file as provided by the BCE from https://www.ecb.europa.eu/stats/eurofxref/eurofxref.zip
    :param: rates_file: name of the rates CSV file
    :return: rates: dict currency code -> rate
    """

    header = None
    rates = []
    with open(rates_file, "r") as f:
        for line in f:
            array = line.rstrip().split(",")
            if len(array) <= 1:
                return None  # skip empty line
            array = [
                x.lstrip() for x in array if x != ""
            ]  # removing heading white spaces
            if header == None:
                # first line is the header: Date, USD, JPY, BGN, CZK, DKK, ...
                header = array
            else:
                # next lines are date and rates: 19 November 2021, 1.1271, 128.22, 1.9558, 25.413, 7.4366, ...
                # convert date into reasonable format
                rate_date = datetime.datetime.strptime(array[0], "%d %B %Y").strftime(
                    "%Y-%m-%d"
                )
                # convert next fields to float
                array = [rate_date] + list(map(float, array[1:]))
                # zip with header
                rates.append(dict(zip(header, array)))

    # only returns the last date for this simple version
    rates = rates[-1]
    logger.info("Currency rates loaded: %s" % rates)
    return rates


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


def enrich(search, rates):
    """
    Adds interesting fields to search object.
    :param search: search object
    :return search: decorated dict describing a search
    """

    def to_euros(amount):
        if search["currency"] == "EUR":
            return amount
        else:
            return round(amount / rates[search["currency"]], 2)

    try:
        # advance purchase & stay duration & OW/RT
        search_date = datetime.datetime.strptime(search["search_date"], "%Y-%m-%d")
        request_dep_date = datetime.datetime.strptime(
            search["request_dep_date"], "%Y-%m-%d"
        )
        # approximate since the dep date is local to the origin city (whereas the search date is UTC)
        search["advance_purchase"] = (request_dep_date - search_date).days
        if search["request_return_date"] == "":
            search["stay_duration"] = -1
            search["trip_type"] = "OW"  # One Way trip
        else:
            request_return_date = datetime.datetime.strptime(
                search["request_return_date"], "%Y-%m-%d"
            )
            # approximative since the return date is local to the destination city
            search["stay_duration"] = (request_return_date - request_dep_date).days
            search["trip_type"] = "RT"  # Round trip

        # decoding passengers string: "ADT=1,CH=2" means 1 Adult and 2 children
        passengers = []
        for pax_string in search["passengers_string"].rstrip().split(","):
            pax_array = pax_string.split("=")
            passengers.append(
                {"passenger_type": pax_array[0], "passenger_nb": int(pax_array[1])}
            )
        search["passengers"] = passengers

        # countries
        search["origin_country"] = get_neob().get(search["origin_city"], "country_code")
        search["destination_country"] = get_neob().get(
            search["destination_city"], "country_code"
        )
        # geo: D=Domestic I=International
        search["geo"] = (
            "D" if search["origin_country"] == search["destination_country"] else "I"
        )

        # OnD (means Origin and Destination. E.g. "PAR-NYC")
        # search["OnD"] = f"{search['origin_city']}-{search['destination_city']}" # Already done by aggregator
        distance = get_neob().distance(
            search["origin_city"], search["destination_city"]
        )
        if type(distance) == float:
            search["OnD_distance"] = round(distance)
        else:
            logger.error(
                f"Failed to get distance between {search['origin_city']} and {search['destination_city']}"
            )
            raise Exception("Failed to get distance")

    except:
        logger.exception("Failed at building search from: %s" % search["recos"][0])
        # filter out recos when we fail at decorating them
        return None

    # reco decoration
    for reco in search["recos"]:

        try:
            # currency conversion
            for field in ["price", "taxes", "fees"]:
                reco[field] = float(reco[field])
                reco[field + "_EUR"] = to_euros(reco[field])

            # will be computed from flights
            marketing_airlines = {}
            operating_airlines = {}
            cabins = {}
            reco["flown_distance"] = 0

            # flight decoration
            for flight in reco["flights"]:
                # getting cities (a city can have several airports like PAR has CDG and ORY)
                flight["dep_city"] = get_neob().get(
                    flight["dep_airport"], "city_code_list"
                )[0]
                flight["arr_city"] = get_neob().get(
                    flight["arr_airport"], "city_code_list"
                )[0]

                flight["distance"] = round(
                    get_neob().distance(flight["dep_airport"], flight["arr_airport"])
                )
                reco["flown_distance"] += flight["distance"]
                marketing_airlines[flight["marketing_airline"]] = (
                    marketing_airlines.get(flight["marketing_airline"], 0)
                    + flight["distance"]
                )
                if flight["operating_airline"] == "":
                    flight["operating_airline"] = flight["marketing_airline"]
                operating_airlines[flight["operating_airline"]] = (
                    operating_airlines.get(flight["operating_airline"], 0)
                    + flight["distance"]
                )
                cabins[flight["cabin"]] = (
                    cabins.get(flight["cabin"], 0) + flight["distance"]
                )

            # the main airline is the one that covers the longuest part of the trip
            reco["main_marketing_airline"] = max(
                marketing_airlines, key=marketing_airlines.get
            )
            reco["main_operating_airline"] = max(
                operating_airlines, key=operating_airlines.get
            )
            reco["main_cabin"] = max(cabins, key=cabins.get)

        except:
            logger.exception("Failed at decorating reco: %s" % reco)
            # filter out recos when we fail at decorating them
            return None

    return search


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
        help="Make enricher process kafka topic from the beginning",
    )
    # default rates file
    def_rates_file = os.path.join(os.path.dirname(__file__), "etc/eurofxref.csv")
    parser.add_argument(
        "-r",
        "--rates_file",
        help=f"Data file with currency rates. Default is {def_rates_file}",
        default=def_rates_file,
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
    consumer_topic = "aggregated_recos"
    kafka_consumer.subscribe([consumer_topic], on_assign=reset_offset)

    # Create Producer instance
    kafka_producer = Producer(kafka_config)
    producer_topic = "enriched_recos"

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

    # Create encoder
    encoder = encoders[arguments.format]

    # Loading currency rates
    rates = load_rates(arguments.rates_file)

    # Poll for new messages from Kafka and enrich them
    logger.info("Enriching")
    cnt = Counter()
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
                cnt["search_read"] += 1
                logger.debug(
                    "Consumed event from topic {topic}: key = {key:12} value = {value:12}".format(
                        topic=msg.topic(),
                        key=msg.key().decode("utf-8"),
                        value=msg.value().decode("utf-8"),
                    )
                )

                search = json.loads(msg.value().decode("utf-8"))
                enriched_search = enrich(search, rates)
                if enriched_search is not None:
                    cnt["search_enriched"] += 1
                    # encode and print
                    logger.info(f"Enriched search: %s" % encoder(search))
                    kafka_producer.produce(
                        producer_topic,
                        json.dumps(enriched_search).encode("utf-8"),
                        search["search_id"],
                        callback=delivery_callback,
                    )
                    # Only needed so that delivery_callback is called
                    kafka_producer.poll(1)

    except KeyboardInterrupt:
        pass
    finally:
        # Leave group and commit final offsets
        kafka_consumer.close()
        # Block until the messages are sent.
        kafka_producer.flush()
        # end time
        end = time.time()

        logger.info(f"Finished in {round(end - start, 2)} seconds: %s" % cnt)
