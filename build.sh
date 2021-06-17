#!/bin/bash

TAG=$(date +"%Y%m%d%H%M")
cargo build --release
docker build -t plork/check:$TAG .
kind load docker-image plork/check:$TAG
echo plork/check:$TAG
