SELECT PERCENTILE_DISC(0.5) WITHIN GROUP(ORDER BY price), airline, advance_purchase
  FROM
  (
   SELECT MIN(recos.price_EUR) AS price, main_marketing_airline AS airline, advance_purchase
      FROM searches JOIN recos on searches.search_id LIKE recos.search_id
      WHERE searches.ond LIKE 'PAR-LIS' AND searches.trip_type LIKE 'RT'
         AND searches.search_country IN ('US', 'RU')
         AND searches.search_date >= '2021-11-17' AND searches.search_date <= '2021-11-17'
         AND searches.stay_duration IN (2, 7)
         AND recos.nb_of_flights IN (1, 2, 3)
      GROUP BY searches.search_id, recos.main_marketing_airline, searches.advance_purchase
    ) t
GROUP BY airline, advance_purchase
