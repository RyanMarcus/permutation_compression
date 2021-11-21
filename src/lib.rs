// Copyright 2021 Ryan Marcus, see COPYING
#![allow(clippy::needless_return)]

mod lr_array;

use std::{convert::TryInto, ops::Range};

use bitpacking::{BitPacker, BitPacker4x};
use lr_array::LRArray;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CompressionMode {
    Fast,
    Slow,
}

fn perm_to_lehmer(perm: &mut [u32]) {
    let mut lr = LRArray::new(perm.len());
    for curr_val in perm.iter_mut() {
        let prev_val = *curr_val;
        *curr_val = lr.unset_before(prev_val as usize) as u32;
        assert!(!lr.set_nth_bit(prev_val as usize));
    }
}

fn lehmer_to_perm(lehmer: &mut [u32]) {
    let mut lr = LRArray::new(lehmer.len());
    for curr_val in lehmer.iter_mut() {
        *curr_val = lr.set_kth_unset_bit(*curr_val as usize) as u32;
    }
}

pub fn compress_permutation(cmode: CompressionMode, mut perm: Vec<u32>) -> Vec<u8> {
    if cmode == CompressionMode::Slow {
        perm_to_lehmer(&mut perm);
    }

    let packer = BitPacker4x::new();
    let perm_len = usize::max(perm.len(), BitPacker4x::BLOCK_LEN);
    let mut compressed = vec![0_u8; 4 + (perm_len * 4) + (perm_len / BitPacker4x::BLOCK_LEN)];

    compressed[0..4].copy_from_slice(&(perm.len() as u32).to_le_bytes());

    let mut next_free_index = 4;

    for idx in (0..perm.len()).step_by(BitPacker4x::BLOCK_LEN) {
        let start = idx;
        let stop = usize::min(perm.len(), idx + BitPacker4x::BLOCK_LEN);
        let data = &perm[start..stop];

        let bytes_written = if data.len() == BitPacker4x::BLOCK_LEN {
            let num_bits = packer.num_bits(data);
            compressed[next_free_index] = num_bits;
            next_free_index += 1;

            packer.compress(data, &mut compressed[next_free_index..], num_bits)
        } else {
            let mut padded = vec![0_u32; BitPacker4x::BLOCK_LEN];
            padded[0..data.len()].copy_from_slice(data);

            let num_bits = packer.num_bits(&padded);
            compressed[next_free_index] = num_bits;
            next_free_index += 1;

            packer.compress(&padded, &mut compressed[next_free_index..], num_bits)
        };
        next_free_index += bytes_written;
    }

    compressed.truncate(next_free_index);
    return compressed;
}

pub fn decompress_permutation(cmode: CompressionMode, data: &[u8]) -> Vec<u32> {
    let packer = BitPacker4x::new();
    let perm_len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let mut next_byte = 4;
    let mut result = Vec::with_capacity(perm_len);

    let mut block = vec![0; BitPacker4x::BLOCK_LEN];
    while next_byte != data.len() {
        let num_bits = data[next_byte];
        next_byte += 1;

        next_byte += packer.decompress(&data[next_byte..], &mut block, num_bits);

        result.extend_from_slice(&block);
    }

    result.truncate(perm_len);
    if cmode == CompressionMode::Slow {
        lehmer_to_perm(&mut result);
    }
    return result;
}

pub fn decompress_permutation_range(
    cmode: CompressionMode,
    data: &[u8],
    range: Range<usize>,
) -> Vec<u32> {
    if cmode == CompressionMode::Slow {
        let perm = decompress_permutation(cmode, data);
        return perm[range].to_vec();
    }

    let packer = BitPacker4x::new();
    let perm_len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let mut next_byte = 4;
    let mut result = Vec::with_capacity(range.len());

    let mut block = vec![0; BitPacker4x::BLOCK_LEN];

    // inclusive bounds
    let first_block_idx = range.start / BitPacker4x::BLOCK_LEN;
    let last_block_idx = range.end / BitPacker4x::BLOCK_LEN;

    let mut curr_block_idx = 0;
    while next_byte != data.len() {
        let num_bits = data[next_byte];
        next_byte += 1;

        next_byte += packer.decompress(&data[next_byte..], &mut block, num_bits);

        if curr_block_idx >= first_block_idx && curr_block_idx <= last_block_idx {
            let curr_block_start = curr_block_idx * BitPacker4x::BLOCK_LEN;
            let curr_block_stop = curr_block_start + BitPacker4x::BLOCK_LEN;

            let rel_start = if range.start > curr_block_start {
                range.start - curr_block_start
            } else {
                0
            };

            let rel_end = if range.end < curr_block_stop {
                range.end - curr_block_start
            } else {
                BitPacker4x::BLOCK_LEN
            };

            result.extend_from_slice(&block[rel_start..rel_end]);
        }

        curr_block_idx += 1;
    }

    return result;
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rand::Rng;

    use super::*;

    fn random_lehmer(sz: usize) -> Vec<u32> {
        let mut v = Vec::new();
        let mut rng = rand::thread_rng();
        while v.len() < sz {
            v.push(rng.gen_range(0..sz - v.len()) as u32);
        }

        return v;
    }

    #[test]
    fn test_generate_random_perm() {
        for _ in 0..1000 {
            let mut perm = random_lehmer(20);
            lehmer_to_perm(&mut perm);
            assert!(perm.iter().all(|v| *v < 20));
            assert_eq!(perm.iter().unique().count(), 20);
        }
    }

    #[test]
    fn wiki_example() {
        let mut test = vec![1, 5, 0, 6, 3, 4, 2];

        perm_to_lehmer(&mut test);
        assert_eq!(test, vec![1, 4, 0, 3, 1, 1, 0]);

        lehmer_to_perm(&mut test);
        assert_eq!(test, vec![1, 5, 0, 6, 3, 4, 2]);
    }

    #[test]
    fn test_random_perm() {
        for _ in 0..1000 {
            let mut perm = random_lehmer(20);
            lehmer_to_perm(&mut perm);
            let orig = perm.clone();

            perm_to_lehmer(&mut perm);
            lehmer_to_perm(&mut perm);
            assert_eq!(perm, orig);
        }
    }

    #[test]
    fn test_compress_random_perm_slow() {
        for _ in 0..1000 {
            let mut perm = random_lehmer(20);
            lehmer_to_perm(&mut perm);
            let orig = perm.clone();

            let compressed = compress_permutation(CompressionMode::Slow, perm);
            let recovered = decompress_permutation(CompressionMode::Slow, &compressed);

            assert_eq!(recovered, orig);
        }
    }

    #[test]
    fn test_compress_random_perm_fast() {
        for _ in 0..1000 {
            let mut perm = random_lehmer(20);
            lehmer_to_perm(&mut perm);
            let orig = perm.clone();

            let compressed = compress_permutation(CompressionMode::Fast, perm);
            let recovered = decompress_permutation(CompressionMode::Fast, &compressed);

            assert_eq!(recovered, orig);
        }
    }

    #[test]
    fn test_compress_random_perm_fast_subset() {
        let mut perm = random_lehmer(500);
        lehmer_to_perm(&mut perm);
        let orig = perm.clone();

        let compressed = compress_permutation(CompressionMode::Fast, perm);
        let recovered = decompress_permutation(CompressionMode::Fast, &compressed);

        let slc = decompress_permutation_range(CompressionMode::Fast, &compressed, 0..10);
        assert_eq!(slc, &recovered[0..10]);

        let slc = decompress_permutation_range(CompressionMode::Fast, &compressed, 100..200);
        assert_eq!(slc, &recovered[100..200]);

        let slc = decompress_permutation_range(CompressionMode::Fast, &compressed, 100..490);
        assert_eq!(slc, &recovered[100..490]);

        assert_eq!(recovered, orig);
    }
}
