pragma circom 2.0.0;

// NFT71 — ZK proof of the 71-shard Hurrian Hymn h.6 collection.
//
// Proves: "I know 71 DA51 CBOR shards whose Poseidon hashes form a valid
//          Merkle tree with the claimed public root, AND the shard count
//          equals 71 (the crown prime), AND the eigenspace commitment matches."
//
// Public inputs:  merkle_root, eigenspace_commitment, shard_count (=71)
// Private witness: 71 shard hashes, Merkle proof paths
//
// Uses Poseidon (ZK-friendly) for the Merkle tree.
// SHA-256 of actual CBOR bytes is verified off-chain; Poseidon binds
// the shard hashes into the on-chain commitment.
//
// Tree structure: 128-leaf binary Merkle (71 real + 57 zero-padded)
// Depth: 7 (2^7 = 128 ≥ 71)

include "circomlib/poseidon.circom";
include "circomlib/comparators.circom";
include "circomlib/bitify.circom";

// Hash two children into a parent using Poseidon
template PoseidonPair() {
    signal input left;
    signal input right;
    signal output out;

    component h = Poseidon(2);
    h.inputs[0] <== left;
    h.inputs[1] <== right;
    out <== h.out;
}

// Verify a single leaf's Merkle inclusion at a given depth
template MerkleProof(depth) {
    signal input leaf;
    signal input pathElements[depth];
    signal input pathIndices[depth];  // 0 = left, 1 = right
    signal output root;

    component hashers[depth];
    component mux_l[depth];
    component mux_r[depth];

    signal levelHash[depth + 1];
    levelHash[0] <== leaf;

    for (var i = 0; i < depth; i++) {
        // pathIndices[i] must be binary
        pathIndices[i] * (1 - pathIndices[i]) === 0;

        // Swap based on path index
        mux_l[i] = Mux1();
        mux_l[i].c[0] <== levelHash[i];
        mux_l[i].c[1] <== pathElements[i];
        mux_l[i].s <== pathIndices[i];

        mux_r[i] = Mux1();
        mux_r[i].c[0] <== pathElements[i];
        mux_r[i].c[1] <== levelHash[i];
        mux_r[i].s <== pathIndices[i];

        hashers[i] = PoseidonPair();
        hashers[i].left <== mux_l[i].out;
        hashers[i].right <== mux_r[i].out;

        levelHash[i + 1] <== hashers[i].out;
    }

    root <== levelHash[depth];
}

// Mux1 — select between two values
template Mux1() {
    signal input c[2];
    signal input s;
    signal output out;
    out <== c[0] + s * (c[1] - c[0]);
}

// Main circuit: prove knowledge of 71 shards forming a Merkle tree
template NFT71() {
    var SHARD_COUNT = 71;
    var TREE_DEPTH = 7;  // 2^7 = 128 leaves

    // === Public inputs ===
    signal input merkleRoot;
    signal input eigenspaceCommitment;  // Poseidon(earth, spoke, hub)
    signal input shardCount;            // must equal 71

    // === Private witness ===
    signal input shardHashes[SHARD_COUNT];
    signal input merklePathElements[SHARD_COUNT][TREE_DEPTH];
    signal input merklePathIndices[SHARD_COUNT][TREE_DEPTH];
    signal input earthPct;   // eigenspace percentages (scaled ×100)
    signal input spokePct;
    signal input hubPct;

    // === Constraint 1: shard count = 71 (the crown prime) ===
    shardCount === SHARD_COUNT;

    // === Constraint 2: eigenspace commitment ===
    component eigenHash = Poseidon(3);
    eigenHash.inputs[0] <== earthPct;
    eigenHash.inputs[1] <== spokePct;
    eigenHash.inputs[2] <== hubPct;
    eigenspaceCommitment === eigenHash.out;

    // === Constraint 3: eigenspace sums to 10000 (100.00%) ===
    earthPct + spokePct + hubPct === 10000;

    // === Constraint 4: each shard hash is non-zero ===
    component nonZero[SHARD_COUNT];
    for (var i = 0; i < SHARD_COUNT; i++) {
        nonZero[i] = IsZero();
        nonZero[i].in <== shardHashes[i];
        nonZero[i].out === 0;  // must NOT be zero
    }

    // === Constraint 5: all 71 shards are in the Merkle tree ===
    component proofs[SHARD_COUNT];
    for (var i = 0; i < SHARD_COUNT; i++) {
        proofs[i] = MerkleProof(TREE_DEPTH);
        proofs[i].leaf <== shardHashes[i];
        for (var j = 0; j < TREE_DEPTH; j++) {
            proofs[i].pathElements[j] <== merklePathElements[i][j];
            proofs[i].pathIndices[j] <== merklePathIndices[i][j];
        }
        // Every proof must yield the same root
        proofs[i].root === merkleRoot;
    }

    // === Constraint 6: shard 71 (index 70) is the crown — colophon ===
    // The crown shard hash must differ from all others (uniqueness of colophon)
    component crownCheck[SHARD_COUNT - 1];
    for (var i = 0; i < SHARD_COUNT - 1; i++) {
        crownCheck[i] = IsEqual();
        crownCheck[i].in[0] <== shardHashes[70];
        crownCheck[i].in[1] <== shardHashes[i];
        crownCheck[i].out === 0;  // crown must be unique
    }
}

component main {public [merkleRoot, eigenspaceCommitment, shardCount]} = NFT71();
