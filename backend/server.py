import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydruid.db import connect

conn = connect(
    host="druid.k8s.sysware.fr",
    port=443,
    path="/druid/v2/sql/",
    scheme="https",
    user="user",
    password="BSrLoanZurN4a8a9",
)
curs = conn.cursor()


def get_sql_request(
    OnD="PAR-LIS",
    trip_type="RT",
    search_date_min="2021-11-17",
    search_date_max="2023-11-17",
):
    # SQL injectable.
    return f"""
    SELECT
        APPROX_QUANTILE_DS(price, 0.5),
        airline,
        advance_purchase
    FROM (
        SELECT
            MIN(db_recos.price_EUR) AS price,
            db_recos.main_marketing_airline AS airline,
            s.advance_purchase
        FROM db_recos
        JOIN (
            SELECT
                search_id,
                advance_purchase
            FROM db_searches
            WHERE db_searches.OnD LIKE '{OnD}' AND db_searches.trip_type LIKE '{trip_type}' AND db_searches.search_date >= '{search_date_min}' AND db_searches.search_date <= '{search_date_max}'
        ) AS s ON s.search_id = db_recos.search_id
        GROUP BY db_recos.search_id, db_recos.main_marketing_airline, s.advance_purchase
    ) AS t
    GROUP BY airline, advance_purchase
    """


app = FastAPI()


@app.get("/")
async def root(OnD: str, trip_type: str, search_date_min: str, search_date_max: str):
    try:
        request = get_sql_request(OnD, trip_type, search_date_min, search_date_max)

        curs.execute(request)
        result = curs.fetchall()
        print(result)
        return [
            {
                "airline": row.airline,
                "advance_purchase": row.advance_purchase,
                "price": row._0,
            }
            for row in result
        ]
    except:
        return []


# We believe in security.
origins = ["*", "http://13.38.29.32/*"]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# )
# for row in curs:
#    print(row)

if __name__ == "__main__":
    uvicorn.run(app)
