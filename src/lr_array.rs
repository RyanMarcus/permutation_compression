// Copyright 2021 Ryan Marcus, see COPYING
use bitvec::prelude::BitVec;

/// A specialized fixed-size bit vector with the following operations:
/// 1) find (and optionally set) the kth unset bit in O(log(n))
/// 2) set the nth bit in O(log(n))
/// 3) count the number of unset bits before index k in O(log(n))
///
/// Loosely based on the LRArray from Jörg Arndt's FXT book.
pub struct LRArray {
    /// total number of bits
    total_bits: usize,

    /// total set bits
    total_set_bits: usize,

    /// raw bit values
    vals: BitVec,

    /// laid out as a tree, the number of bits set of each node's children
    f: Vec<usize>,
}

impl LRArray {
    pub fn new(size: usize) -> LRArray {
        let mut vals = BitVec::with_capacity(size);
        for _ in 0..size {
            vals.push(false);
        }

        let f = vec![0; size * 2];

        return LRArray {
            vals,
            f,
            total_bits: size,
            total_set_bits: 0,
        };
    }

    pub fn unset_bits(&self) -> usize {
        return self.total_bits - self.total_set_bits;
    }

    pub fn set_bits(&self) -> usize {
        return self.total_set_bits;
    }

    pub fn total_bits(&self) -> usize {
        return self.total_bits;
    }

    #[cfg(test)]
    pub fn get_bit(&self, n: usize) -> bool {
        return self.vals[n];
    }

    pub fn unset_before(&self, n: usize) -> usize {
        if n >= self.total_bits() {
            return self.unset_bits();
        }

        let mut curr_start = 0;
        let mut curr_stop = self.total_bits;
        let mut f_idx = 0;
        let mut num_bits_before = 0;
        debug_assert_eq!(self.f[0], self.set_bits());

        while curr_stop - curr_start > 2 {
            let left_child_idx = f_idx * 2 + 1;
            let right_child_idx = f_idx * 2 + 2;

            let child_range_size = (curr_stop - curr_start) / 2;
            let free_bits_left = child_range_size - self.f[left_child_idx];
            let midpoint = curr_start + child_range_size;

            if n < midpoint {
                // go left
                curr_stop -= child_range_size;
                f_idx = left_child_idx;
            } else {
                // go right
                curr_start += child_range_size;
                f_idx = right_child_idx;
                num_bits_before += free_bits_left;
            }
        }

        // the binary search above narrows it down to a range of size 2
        if n > curr_start && !self.vals[n - 1] {
            num_bits_before += 1;
        }

        return num_bits_before;
    }

    pub fn set_nth_bit(&mut self, n: usize) -> bool {
        if self.vals[n] {
            return true;
        }

        let mut curr_start = 0;
        let mut curr_stop = self.total_bits;
        let mut f_idx = 0;
        debug_assert_eq!(self.f[0], self.set_bits());
        self.f[0] += 1;

        while curr_stop - curr_start > 2 {
            let left_child_idx = f_idx * 2 + 1;
            let right_child_idx = f_idx * 2 + 2;

            let child_range_size = (curr_stop - curr_start) / 2;
            let midpoint = curr_start + child_range_size;

            if n < midpoint {
                // go left
                curr_stop -= child_range_size;
                f_idx = left_child_idx;
            } else {
                // go right
                curr_start += child_range_size;
                f_idx = right_child_idx;
            }
            self.f[f_idx] += 1;
        }

        self.vals.set(n, true);
        self.total_set_bits += 1;
        return false;
    }

    pub fn set_kth_unset_bit(&mut self, mut k: usize) -> usize {
        if k >= self.unset_bits() {
            panic!(
                "Trying to set {}th free bit, but only {} free bits left",
                k,
                self.unset_bits()
            );
        }

        let mut curr_start = 0;
        let mut curr_stop = self.total_bits;
        let mut f_idx = 0;
        debug_assert_eq!(self.f[0], self.set_bits());
        self.f[0] += 1;

        while curr_stop - curr_start > 2 {
            let left_child_idx = f_idx * 2 + 1;
            let right_child_idx = f_idx * 2 + 2;

            let child_range_size = (curr_stop - curr_start) / 2;

            #[cfg(test)]
            let free_bits_left = child_range_size - self.f[left_child_idx];
            
            #[cfg(not(test))]
            let free_bits_left = unsafe { child_range_size - self.f.get_unchecked(left_child_idx) };

            if free_bits_left > k {
                // go left
                curr_stop -= child_range_size;
                f_idx = left_child_idx;
            } else {
                // go right
                curr_start += child_range_size;
                f_idx = right_child_idx;
                k -= free_bits_left;
            }
            self.f[f_idx] += 1;
        }

        // the binary search above narrows it down to a range of size 2
        debug_assert!(k < 2);
        let idx = if k == 1 || self.vals[curr_start] {
            curr_start + 1
        } else {
            curr_start
        };

        debug_assert!(!self.vals[idx]);
        self.vals.set(idx, true);
        self.total_set_bits += 1;
        return idx;
    }
}

#[cfg(test)]
mod lr_tests {
    use super::*;

    #[test]
    fn test_set_kth_unset_bit() {
        let mut array = LRArray::new(50);
        assert_eq!(array.unset_bits(), 50);

        assert_eq!(array.set_kth_unset_bit(4), 4);
        assert_eq!(array.get_bit(4), true);
        assert_eq!(array.get_bit(3), false);

        assert_eq!(array.set_kth_unset_bit(4), 5);
        assert_eq!(array.set_kth_unset_bit(4), 6);
        assert_eq!(array.set_kth_unset_bit(0), 0);
        assert_eq!(array.set_kth_unset_bit(4), 8);
        assert_eq!(array.set_kth_unset_bit(25), 30);
    }

    #[test]
    fn test_unset_before() {
        let mut array = LRArray::new(50);
        assert_eq!(array.set_kth_unset_bit(4), 4);
        assert_eq!(array.unset_before(5), 4);
        assert_eq!(array.unset_before(4), 4);
        assert_eq!(array.unset_before(3), 3);

        assert_eq!(array.set_kth_unset_bit(15), 16);
        assert_eq!(array.unset_before(5), 4);
        assert_eq!(array.unset_before(20), 18);
    }

    #[test]
    fn test_set_nth() {
        let mut array = LRArray::new(50);
        assert_eq!(array.set_nth_bit(4), false);
        assert_eq!(array.set_nth_bit(4), true);

        assert_eq!(array.unset_before(4), 4);
        assert_eq!(array.unset_before(5), 4);
        assert_eq!(array.unset_before(40), 39);

        assert_eq!(array.set_kth_unset_bit(2), 2);
        assert_eq!(array.unset_before(5), 3);

        assert_eq!(array.set_nth_bit(20), false);
        assert_eq!(array.unset_before(40), 37);
    }

    #[test]
    fn test_set_last() {
        let mut array = LRArray::new(5);
        assert_eq!(array.set_kth_unset_bit(3), 3);
        assert_eq!(array.set_kth_unset_bit(3), 4);
    }
}
