# mrklar (Merkle Archive)
A minimal network archive using Merkle proof written in pure Rust.

For more information about the project, please refer to the project (notes)[https://github.com/0xalexbel/mrklar/blob/main/NOTES.md]

# Install

## Build

The repository requires the `protoc` compiler. To install `protoc`, follow instructions from the [protoc install page](https://grpc.io/docs/protoc-installation/)


```bash
# on Linux
apt install -y protobuf-compiler
```

```bash
# on Mac
brew install protobuf
```

To build the repostory, run: 

```bash
cargo build
```

To run the tests:

```bash
cargo test
```

## Usage

```bash
# Server help
cargo run --bin mrklar -- --help
```

```bash
# CLI help
cargo run --bin mrklar-cli -- --help
```

# How to use

## 1. Setup the server

To begin with, starts by creating a few directories
```bash
# where the server db file will be located
$ mkdir -p my_server/db

# where the server uploaded files will be stored
$ mkdir -p my_server/files

# (for testing purpose) where the client with store the downloaded files
$ mkdir -p my_client/downloads
```

From one terminal window, run the server:
```bash
$ cargo run --bin mrklar -- --db-dir ./my_server/db --files-dir ./my_server/files --host 127.0.0.1  --port 10000 --tracing
```

## 2. Upload a file

To upload a file, open a separate terminal window and execute the following commands:

```bash
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10000 upload <path/to/my/awsome/file>
```

The command output will display the uploaded file index (for later download) as well as the new server 
merkle root.

```bash
# upload output format: '<file index> <new merkle root>'
0 6baf2dbc2729dc5c218f11cb3ee01f274e332f3c24f9bbf7702e8cc4981ab3ea

# in the above example: 
# - file index: 0
# - merkle root: 6baf2dbc2729dc5c218f11cb3ee01f274e332f3c24f9bbf7702e8cc4981ab3ea
```

## 2. Query the server merkle root

Use the root command to query the current merkle root.

```bash
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10000 root
```
```bash
# 'root' command output is: '<current merkle root>'
6baf2dbc2729dc5c218f11cb3ee01f274e332f3c24f9bbf7702e8cc4981ab3ea
```

## 3. Download and verify a file

The download command automatically downloads the requested and file and performs verification using the provided merkle proof.
The download command usage is:

```bash
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10000 download <FILE_INDEX> --out-dir <PATH/TO/DOWNLOADS/DIR>
```

For example, to download the file at index `0`, verify it and then store it to the `./my_client/downloads` directory.

```bash
# Here, using the above merkle root, the command would look like this:
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10000 download 0 --out-dir ./my_client/downloads
```

## 4. Additional commands

- `count` : returns the number of stored files and the remote archive
- `proof` : returns the merkle proof of the file with the specified index

## 5. Environment Variables

- `MRKLAR_PORT=<NUM>` : The server port number to listen on.
- `MRKLAR_IP_ADDR=<NUM>` : The server host ip.
- `MRKLAR_DB_DIR=<PATH>` : Path of the directory on the server where the merkle tree db will be saved
- `MRKLAR_FILES_DIR=<PATH>` : Path of the directory on the server where the uploaded files will be saved
- `MRKLAR_TRACING=<true|false>` : Enable/disable server trace
- `MRKLAR_TRACING_LEVEL=<"error" | "warn" | "info" | "debug" | "trace">` : max server trace level

# Docker

## Dockerfile

Two docker images are available:
- 1x debug docker image (server compiled in debug mode)
- 1x release docker image (server compiled in release mode)

To build any docker image, proceed as follow:

```bash
# Go to the images folder
cd docker/images

# build the debug image
./build-debug.sh

# build the release image
./build-release.sh

# build both images at once
./build-all.sh
```

## Docker Compose

A sample docker compose file is provided. It will deploy 3 instances of the `mrklar` server
```bash
# 3 servers are listening to ports 10000, 10001 and 10002
cd docker/compose 
docker-compose up
```

# Tests

## Tests using scripts

To run a test server, run the following in one separate terminal window:

```bash
# server addr is 127.0.0.1:10002
cd scripts
./server.sh
```
From another terminal window run:

```bash
# This command will upload 200 times the same file (the './LICENSE' file)
# to the remote archive listening to port 10002
cd scripts
./cli_upload.sh
```

Wait a few uploads, then, from a thrid terminal window run the concurrent download script:

```bash
# This command will download the 200 uploaded files to the remote archive listening to port 10002
# (multiple copies of the same './LICENSE' file, since it is the only file that has been uploaded)
cd scripts
./cli_download.sh
```

## Tests using Docker

To start the server instances, run the following command from a separate terminal window:

```bash
cd docker/compose
docker-compose up
```

Run the `./cli_uload.sh` and `./cli_download.sh` as described above.

