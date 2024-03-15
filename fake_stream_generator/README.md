# Fake stream generator

This python scripts generate a stream of fake travel searches and sends them as messages
to a kafka instance.

You can configure the script using the following environment variables:

- `MSG_PER_SEC`: (default: 1000), Number of simulated searches per second. Set to -1 to remove limits.
- `KAFKA_URL`: (default: localhost:1234), URL of the kafka instance to connect to.
