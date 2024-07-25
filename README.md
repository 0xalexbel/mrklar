# mrklar (Merkle Archive)
A minimal network archive using Merkle proof written in full Rust.

The project consists of the following rust crates. 
- `mrklar`: the server crate
- `mrklar-api`: a client-side api to interact with the server
- `mrklar-cli`: a cli to execute download/upload operations from the command-line using the `mrklar-api` crate.
- `tree`: the merkle tree crate
- `fs`: the file system helpers crate
- `common`: the shared `proto` code and generated rust code as well as other shared resources.

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

```bash
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10000 root
```
```bash
# 'root' command output is: '<current merkle root>'
6baf2dbc2729dc5c218f11cb3ee01f274e332f3c24f9bbf7702e8cc4981ab3ea
```

## 3. Download and verify a file

The download command usage is:

```bash
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10000 download <FILE_INDEX> --out-dir <PATH/TO/DOWNLOADS/DIR> --verify <THE_CURRENT_MERKLE_ROOT>
```

For example, to download the file at index `0`, verify it and then store it to the `./my_client/downloads` directory, the current merkle root must be provided as argument using the `--verify` option.

```bash
# Here, using the above merkle root, the command would look like this:
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10000 download 0 --out-dir ./my_client/downloads --verify 6baf2dbc2729dc5c218f11cb3ee01f274e332f3c24f9bbf7702e8cc4981ab3ea
```

## 4. Additional commands

- `count` : returns the number of stored files and the remote archive
- `proof` : returns the merkle proof of the file with the specified index

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
./cli.sh
```

## Tests using Docker

To start the server instances, run the following command from a separate terminal window:

```bash
cd docker/compose
docker-compose up
```

From another terminal window run:

```bash
# This command will upload 200 times the same file (the './LICENSE' file)
# to the remote archive listening to port 10002
cd scripts
./cli.sh
```

or

```bash
# From the cargo workspace root directory
cargo run --bin mrklar-cli -- --host 127.0.0.1 --port 10002 upload ./LICENSE
```
