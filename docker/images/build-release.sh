#!/bin/bash

CARGO_ROOT=../..

# run script from cargo workspace root dir
cd $CARGO_ROOT

docker build . -t mrklar-release -f docker/images/Dockerfile.release