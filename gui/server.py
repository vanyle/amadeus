from pydruid.db import connect
from sqlalchemy import *
from sqlalchemy.schema import *

# engine = create_engine('druid://localhost:8082/druid/v2/sql/')
# engine = create_engine(
#    "druid+https://user:BSrLoanZurN4a8a9@druid.k8s.sysware.fr/druid/v2/sql/?header=true",
# )

conn = connect(
    host="druid.k8s.sysware.fr",
    port=443,
    path="/druid/v2/sql/",
    scheme="https",
    user="user",
    password="BSrLoanZurN4a8a9",
)
curs = conn.cursor()
curs.execute(
    """
        SELECT
      search_id,
      advance_purchase
    FROM db_searches
    WHERE db_searches.OnD LIKE 'PAR-LIS' AND db_searches.trip_type LIKE 'RT'
"""
)
for row in curs:
    print(row)
