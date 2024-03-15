"""

Generate a fake Kafka stream of travel searches

"""

import gzip
import json
import logging
import os
import random
import time
from collections import Counter
from datetime import datetime, timedelta

import kafka.errors
from dotenv import load_dotenv

import kafka
from kafka import KafkaProducer

load_dotenv()

_RECO_LAYOUT = [
    "version_nb",
    "search_id",
    "search_country",
    "search_date",
    "search_time",
    "origin_city",
    "destination_city",
    "request_dep_date",
    "request_return_date",
    "passengers_string",
    "currency",
    "price",
    "taxes",
    "fees",
    "nb_of_flights",
]

MSG_PER_SEC = -1  # int(os.environ.get("MSG_PER_SEC", 1000))  # Searches per second
KAFKA_URL = os.environ.get("KAFKA_URL", "localhost:1234")

producer = None
try:
    producer = KafkaProducer(bootstrap_servers=KAFKA_URL)
except kafka.errors.NoBrokersAvailable:
    logging.error("No brokers available")
    logging.error(
        "Make sure to start Kafka and specify the correct URL inside KAFKA_URL"
    )
    logging.info("Starting stream anyway...")
    time.sleep(1)

# Open the compressed file (traval_data_sample.csv.gz)
# with gzip.open("travel_data_sample.csv.gz", "rt") as f:


_RECO_LAYOUT = [
    "version_nb",
    "search_id",
    "search_country",
    "search_date",
    "search_time",
    "origin_city",
    "destination_city",
    "request_dep_date",
    "request_return_date",
    "passengers_string",
    "currency",
    "price",
    "taxes",
    "fees",
    "nb_of_flights",
]

_FLIGHT_LAYOUT = [
    "dep_airport",
    "dep_date",
    "dep_time",
    "arr_airport",
    "arr_date",
    "arr_time",
    "operating_airline",
    "marketing_airline",
    "flight_nb",
    "cabin",
]

# Log init
logging.basicConfig(
    format="%(levelname)-8s [%(filename)s:%(lineno)d] %(message)s",
    datefmt="%Y-%m-%d:%H:%M:%S",
    level=logging.DEBUG,
)
logger = logging.getLogger()
logger.setLevel(logging.INFO)


# CSV decoding
def decode_line(line):
    """
    Decodes a CSV line based on _RECO_LAYOUT and _FLIGHT_LAYOUT
    :param line: string containing a CSV line
    :return reco: dict with decoded CSV fields
    """

    try:
        # converting to text string
        if isinstance(line, bytes):
            line = line.decode()

        # splitting the CSV line
        array = line.rstrip().split("^")
        if len(array) <= 1:
            logger.warning("Empty line")
            return None  # skip empty line

        # decoding fields prior to flight details
        reco: dict[str, any] = dict(zip(_RECO_LAYOUT, array))
        read_columns_nb = len(_RECO_LAYOUT)

        # convert to integer
        reco["nb_of_flights"] = int(reco["nb_of_flights"])

        # decoding flights details
        reco["flights"] = []
        for i in range(0, reco["nb_of_flights"]):
            flight = dict(zip(_FLIGHT_LAYOUT, array[read_columns_nb:]))
            read_columns_nb += len(_FLIGHT_LAYOUT)
            reco["flights"].append(flight)

    except:
        logger.exception("Failed at decoding CSV line: %s" % line.rstrip())
        return None

    return reco


# Reco processing
_SEARCH_FIELDS = [
    "version_nb",
    "search_id",
    "search_country",
    "search_date",
    "search_time",
    "origin_city",
    "destination_city",
    "request_dep_date",
    "request_return_date",
    "passengers_string",
    "currency",
]


def group_and_decorate(recos_in):
    """
    Groups recos to build a search object. Adds interesting fields.
    :param recos_in: set of dict (decoded travel recommendations belonging to the same search)
    :return search: decorated dict describing a search
    """

    # don't decorate empty search or empty recos
    if recos_in is None or len(recos_in) == 0:
        return None
    recos = [reco for reco in recos_in if reco is not None]

    try:
        # some fields are common to the search, others are specific to recos
        # taking search fields from the first reco
        search = {
            key: value for key, value in recos[0].items() if key in _SEARCH_FIELDS
        }
        # keeping other fields only in reco
        search["recos"] = [
            {key: value for key, value in reco.items() if key not in _SEARCH_FIELDS}
            for reco in recos
        ]

        # advance purchase & stay duration & OW/RT
        search_date = datetime.strptime(search["search_date"], "%Y-%m-%d")
        request_dep_date = datetime.strptime(search["request_dep_date"], "%Y-%m-%d")
        # approximate since the dep date is local to the origin city (whereas the search date is UTC)
        search["advance_purchase"] = (request_dep_date - search_date).days
        if search["request_return_date"] == "":
            search["stay_duration"] = -1
            search["trip_type"] = "OW"  # One Way trip
        else:
            request_return_date = datetime.strptime(
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

        # search["origin_country"] = get_neob().get(search["origin_city"], "country_code")

        # search["destination_country"] = get_neob().get(
        #     search["destination_city"], "country_code"
        # )
        # geo: D=Domestic I=International
        # search["geo"] = (
        #     "D" if search["origin_country"] == search["destination_country"] else "I"
        # )

        # OnD (means Origin and Destination. E.g. "PAR-NYC")
        search["OnD"] = f"{search['origin_city']}-{search['destination_city']}"
        # search["OnD_distance"] = round(
        #     get_neob().distance(search["origin_city"], search["destination_city"])
        # )

    except:
        # logger.exception("Failed at building search from: %s", search)
        return None

    # reco decoration
    for reco in search["recos"]:

        try:
            # currency conversion
            for field in ["price", "taxes", "fees"]:
                reco[field] = float(reco[field])
                # reco[field + "_EUR"] = to_euros(reco[field])

            # will be computed from flights
            marketing_airlines: dict[str, int] = {}
            operating_airlines: dict[str, int] = {}
            cabins: dict[str, int] = {}
            reco["flown_distance"] = 0

            # flight decoration
            for f in reco["flights"]:
                # getting cities (a city can have several airports like PAR has CDG and ORY)
                # f["dep_city"] = get_neob().get(f["dep_airport"], "city_code_list")[0]
                # f["arr_city"] = get_neob().get(f["arr_airport"], "city_code_list")[0]

                # f["distance"] = round(
                #     get_neob().distance(f["dep_airport"], f["arr_airport"])
                # )
                # reco["flown_distance"] += f["distance"]
                # marketing_airlines[f["marketing_airline"]] = (
                #     marketing_airlines.get(f["marketing_airline"], 0) + f["distance"]
                # )
                if f["operating_airline"] == "":
                    f["operating_airline"] = f["marketing_airline"]
                # operating_airlines[f["operating_airline"]] = (
                #     operating_airlines.get(f["operating_airline"], 0) + f["distance"]
                # )
                # cabins[f["cabin"]] = cabins.get(f["cabin"], 0) + f["distance"]

            # the main airline is the one that covers the longuest part of the trip
            # reco["main_marketing_airline"] = max(
            #     marketing_airlines, key=marketing_airlines.get  # type: ignore
            # )
            # reco["main_operating_airline"] = max(
            #     operating_airlines, key=operating_airlines.get  # type: ignore
            # )
            # reco["main_cabin"] = max(cabins, key=cabins.get)  # type: ignore

        except:
            logger.exception("Failed at decorating reco: %s" % reco)
            # filter out recos when we fail at decorating them
            return None

    return search


def make_search_more_realistic(search):
    search["search_date"] = datetime.now().strftime("%Y-%m-%d")
    search["search_time"] = datetime.now().strftime("%H:%M:%S")

    departure_date = datetime.now() + timedelta(days=random.randint(3, 100))
    search["request_dep_date"] = departure_date.strftime("%Y-%m-%d")

    if search["request_return_date"]:
        return_date = departure_date + timedelta(days=search["stay_duration"])
        search["request_return_date"] = return_date.strftime("%Y-%m-%d")

    for reco in search["recos"]:
        for flight in reco["flights"]:
            flight_time = departure_date + timedelta(days=random.randint(-2, 2))
            flight["dep_date"] = flight_time.strftime("%Y-%m-%d")
            flight["arr_date"] = flight_time.strftime("%Y-%m-%d")

    return search


def process_search(search):
    if producer is not None:
        producer.send("searches", json.dumps(search).encode("utf-8"))

    if MSG_PER_SEC > 0:
        time.sleep(1 / MSG_PER_SEC)


# Main function
def process():
    cnt = Counter()
    recos = []
    current_search_id = 0
    # open input file (or stdin if there is none)
    with gzip.open("travel_data_sample.csv.gz", "r") as f:
        lineCount = 0
        while True:
            if not f.readline():
                break
            lineCount += 1
        logging.info(f"Total lines: {lineCount}")
        logging.info("Starting stream")
        f.seek(0)

        while True:
            line = f.readline()
            if not line:  # Loop forever.
                logger.info("End of file, looping")
                f.seek(0)
                continue

            cnt["reco_read"] += 1
            reco = decode_line(line)
            if reco:
                cnt["reco_decoded"] += 1
                # new search_id means new search: we can process the collected recos
                if reco["search_id"] != current_search_id:
                    current_search_id = reco["search_id"]
                    if len(recos) > 0:
                        if cnt["search_read"] % 1000 == 0:
                            # log every 1000 searches to show the script is alive
                            logger.info(f"Running: %s" % cnt)
                        cnt["search_read"] += 1
                        search = group_and_decorate(recos)
                        if search:
                            cnt["search_encoded"] += 1
                            search = make_search_more_realistic(search)
                            process_search(search)

                        recos = []
                recos.append(reco)


process()
