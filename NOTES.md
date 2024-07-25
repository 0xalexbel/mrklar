## The Merkle Tree

Location:
- the `tree` crate
- `mem_db.rs` in the server `mrklar` crate

```rust
// From tree/src/merkle_tree.rs
struct MemDbInner {
    entries: Vec<MemDbEntry>,
    tree: MerkleTree,
}
```

The `tree` crate contains the source code of a minimal in-memory Merkle tree structure. Rather than using a classic straightforward binary tree architecture involving parent-children linked nodes, the tree is organized in levels. The leaves are stored at level 0, parents at level 1, etc., up to level N where the tree root is located. This approach minimizes the use of pointers while keeping enough flexibility.

Along with the tree structure, a simple one-dimensional array is used to store the metadata associated with each file entry, a simple yet memory-efficient choice since a file is always referred by its index. In the current version, each file's metadata contains the original filename to allow the user to download a requested file with its original filename rather than using a hash value (which would be less user-friendly). The metadata structure could store additional info in the future.

## The Merkle Proof

Location:
- `merkle_proof.rs` in the `common` crate

```rust
pub struct MerkleProof {
    // the tree merkle root
    root: Vec<u8>,
    // the tree merkle proof
    hashes: Vec<MerkleProofHash>,
}
```

## The Client-Server workflow

The Client-Server messaging relies on gRPC using the `protoc` compiler. 

### Upload
1. The client uploads a file to the server. The request includes the file SHA-256 hash as well as the filename.
2. The server streams the uploaded bytes into a temporary file and computes the SHA-256 hash chunk by chunk.
3. The server checks the file integrity by comparing the provided SHA-256 hash with its own computed SHA-256 hash.
4. The server adds the file's hash into the Merkle tree structure, saves the file metadata, and moves the temporary file into the final repository.
5. The server computes the new database Merkle root and sends the file index along with the new Merkle root back to the client.
6. The server saves the updated database on disk.

### Download
1. The client downloads a file from the server using a given file index.
2. The server retrieves the file content corresponding to the given file index as well as its original filename.
3. The server computes a Merkle proof associated with the given file index.
4. The server sends the file content, the Merkle proof, and the filename back to the client.
5. The client verifies the downloaded bytes using the Merkle proof.

## The Server-side storage
- Files are flat-stored in a single directory for performance reasons (specified via the `--files-dir` server option).
- The tree and metadata are stored in a single binary file serialized using Serde (specified via the `--db-dir` server option).
- Files are named by their individual index in the database. The first file is named '0', the second one is named '1', etc. This approach allows for easier database rebuilding in case of corrupted data (e.g., an interrupted save operation).

## The main Rust crate dependencies

- [tonic](https://github.com/hyperium/tonic) : a popular native Rust gRPC client & server implementation with async/await support. ()
- [tokio](https://github.com/tokio-rs/tokio) : the de-facto standard runtime for writing reliable asynchronous applications with Rust. 

## What's missing

The following important features are missing:

- add an option to send the merkle proof inside the server upload response. The current implementation only sends back the new merkle root.
- The client-server protocol encryption. This should be quite straightforward with the help of tonic's `ServerTlsConfig` and `Certificate` traits and structs
- A better in-memory/disk storage mechanism. The current implementation saves the database on disk after each upload which is obviously very bad in terms of perfomance.
- Use a Redis db to store the merkle tree ?
- The in-memory database roll-back.
- The in-memory database sanity and integrity check mechanism.
- JsonRPC protocol (for `curl` access)
- cli JSON output format for better output parsing using `jq` for example.

## Where to go from here ?

One of the most obvious improvements would be to add a sort of "IPFS-like" system with a Merkle tree distributed among network nodes. Each node would only keep a sub-tree of the global Merkle tree. One could also consider separating the Merkle tree data from the file storage to distribute the load pressure between nodes.