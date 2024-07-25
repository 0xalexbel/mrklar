#!/bin/bash

# cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10002 upload ../LICENSE
# cargo run -r --bin mrklar-cli -- --host 127.0.0.1 --port 10002 upload ../LICENSE

# Uncomment the line below to start the server is release mode
# RELEASE=1

PORT=10002
FILE_TO_UPLOAD="../LICENSE"

CARGO_RELEASE_FLAG=""
if [[ $RELEASE -eq 1 ]]
then 
    CARGO_RELEASE_FLAG="-r"
fi

for i in $(seq 1 1 200)
do
   echo "Upload #$i/200"
   cargo run ${CARGO_RELEASE_FLAG} --bin mrklar-cli -- --host 127.0.0.1 --port ${PORT} upload ${FILE_TO_UPLOAD}
   sleep .5
done
