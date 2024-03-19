import psycopg2

# Connect to an existing database
conn = psycopg2.connect(
    dbname="postgres", user="postgres", password="password", host="127.0.0.1"
)

# Open a cursor to perform database operations
cur = conn.cursor()

# Reset tables
cur.execute("DROP TABLE searches, recos;")

# Create table for searches
cur.execute(
    "CREATE TABLE IF NOT EXISTS searches (\
      id serial PRIMARY KEY,\
      search_id VARCHAR UNIQUE NOT NULL,\
      search_country VARCHAR NOT NULL,\
      search_date DATE NOT NULL,\
      request_dep_date DATE NOT NULL,\
      advance_purchase INTEGER,\
      stay_duration INTEGER,\
      trip_type VARCHAR NOT NULL,\
      ond VARCHAR NOT NULL\
    );"
)

# Create table for recos
cur.execute(
    "CREATE TABLE IF NOT EXISTS recos (\
      id serial PRIMARY KEY,\
      search_id VARCHAR REFERENCES searches(search_id),\
      nb_of_flights INTEGER,\
      price_EUR REAL,\
      main_marketing_airline VARCHAR NOT NULL,\
      main_operating_airline VARCHAR NOT NULL\
    );"
)

# Make the changes to the database persistent
conn.commit()

# Close communication with the database
cur.close()
conn.close()
