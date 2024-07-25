# mrklar (Merkle Archive)
A minimal network archive using Merkle proof

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

To start the instances, run the following in one separate terminal window:

```bash
cd docker/compose
docker-compose up
```

In another terminal window run:

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
