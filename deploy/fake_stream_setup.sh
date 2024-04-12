#!/bin/bash

sudo add-apt-repository ppa:longsleep/golang-backports
sudo apt update
sudo apt install golang-1.22

mkdir /fake_stream
cd /fake_stream

git clone https://github.com/vanyle/Amadeus/
cd fake_stream_generator

go build .
go run .