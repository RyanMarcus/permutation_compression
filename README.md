# permutation_compression

[![Rust](https://github.com/RyanMarcus/permutation_compression/actions/workflows/rust.yml/badge.svg)](https://github.com/RyanMarcus/permutation_compression/actions/workflows/rust.yml)

A Rust library to compress permutations.

```rust
// create a random permutation by shuffling
let mut rng = rand::thread_rng();
let mut data: Vec<u32> = (0..1_000_000_u32).collect();
data.shuffle(&mut rng);
let orig_perm = data.clone();

// compress the permutation
let compressed = compress_permutation(CompressionMode::Slow, data);

// recover the compressed permutation
let recovered = decompress_permutation(CompressionMode::Slow, &compressed);

assert_eq!(recovered, orig_perm);
```

The information-theoretic lower bound for compressing a permutation of size `n` is `O(log2(n!))`. In fast mode, we achieve `O(n log n)` compression using bitpacking. In slow mode, we remove another `O(n)`.

On my personal hardware:

| mode |  n | compression time | decompression time | uncompressed size | compressed size | compression ratio |
|------|----|------------------|--------------------|-------------------|-----------------|-------------------|
| Fast | 1M |      < 5ms       |       < 5ms        |        4MB        |        2.5MB    |       1.595       |
| Slow | 1M |      220ms       |       185ms        |        4MB        |        2.4MB    |       1.684       |


This code is available under the AGPL-3.0 license. See `COPYING` for more information.