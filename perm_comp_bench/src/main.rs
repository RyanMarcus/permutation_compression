// Copyright 2021 Ryan Marcus, see COPYING

use std::time::SystemTime;

use bytesize::ByteSize;
use permutation_compression::{CompressionMode, compress_permutation, decompress_permutation};
use rand::seq::SliceRandom;

fn benchmark(cmode: CompressionMode) {
    let mut rng = rand::thread_rng();
    let mut data: Vec<u32> = (0..1_000_000_u32).collect();
    data.shuffle(&mut rng);

    let before = data.clone();

    let start = SystemTime::now();
    let compressed = compress_permutation(cmode, data);
    let compression_time = start.elapsed().unwrap().as_millis();

    let start = SystemTime::now();
    let recovered = decompress_permutation(cmode, &compressed);
    let decompression_time = start.elapsed().unwrap().as_millis();
    assert_eq!(recovered, before);

    println!(
        "Uncom / Com / Ratio: {} / {} / {}",
        ByteSize::b(before.len() as u64 * 4),
        ByteSize::b(compressed.len() as u64),
        (before.len() * 4) as f64 / compressed.len() as f64
    );

    println!("Compression time: {}ms", compression_time);
    println!("Decompression time: {}ms", decompression_time);

}

fn main() {
    println!("Fast mode");
    benchmark(CompressionMode::Fast);
    
    println!();

    println!("Slow mode");
    benchmark(CompressionMode::Slow);
}
