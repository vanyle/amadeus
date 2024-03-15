FROM python:3.11.7-slim as python-base

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
    # deps for installing poetry
    curl \
    git \
    # deps for building python deps
    build-essential

# Install poetry, our package manager
RUN pip install poetry

WORKDIR /app
COPY poetry.lock pyproject.toml ./
RUN poetry install --no-dev
COPY . ./
CMD [ "poetry", "run", "python", "kafkaFakeStreamer.py" ]

