# syntax=docker/dockerfile:1

FROM golang:1.22 as build
COPY go.mod go.sum ./
RUN go mod download
COPY *.go ./
RUN CGO_ENABLED=0 GOOS=linux go build -o /fakestreamgen

FROM scratch
WORKDIR /
COPY --from=build /fakestreamgen /fakestreamgen
COPY ./travel_data_sample.csv.gz /travel_data_sample.csv.gz
CMD ["/fakestreamgen"]
