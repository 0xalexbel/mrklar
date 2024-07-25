#!/bin/bash

# cargo run --bin mrklar -- --host 127.0.0.1 --port 10002  --db-dir ./tmp/db --files-dir ./tmp/files --tracing
# cargo run -r --bin mrklar -- --host 127.0.0.1 --port 10002  --db-dir ./tmp/db --files-dir ./tmp/files --tracing

# Uncomment the line below to start the server is release mode
# RELEASE=1

PORT=10002
TMP_DIR="./tmp"
DB_DIR="${TMP_DIR}/db"
FILES_DIR="${TMP_DIR}/files"

rm -rf "${TMP_DIR}"

mkdir -p "${DB_DIR}"
mkdir -p "${FILES_DIR}"

CARGO_RELEASE_FLAG=""
if [[ $RELEASE -eq 1 ]]
then 
    CARGO_RELEASE_FLAG="-r"
fi

MRKLAR_IP_ADDR="127.0.0.1" \
MRKLAR_PORT="${PORT}" \
MRKLAR_DB_DIR="${DB_DIR}" \
MRKLAR_FILES_DIR="${FILES_DIR}" \
MRKLAR_TRACING="true" \
MRKLAR_TRACING_LEVEL="info" \
cargo run ${CARGO_RELEASE_FLAG} --bin mrklar


