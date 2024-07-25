#!/bin/bash

CUR=$PWD

./build-debug.sh

cd $CUR

./build-release.sh
