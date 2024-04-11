# Filtering

Inputs:

- KAFKA_URL
  We listen to the "raw_recos" topic
  We post the output unchanged to the "filtered_recos" topic.

## Overview of the filtering service

```
while true:
    msg = kafka.receive_message("raw_recos")
    if is_filtered(msg):
        kafka.post_message("filtered_recos", msg)
```
