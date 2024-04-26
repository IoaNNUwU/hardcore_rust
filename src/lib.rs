#![cfg(target_endian = "little")]
#![no_std]

mod alloc;
pub use alloc::*;

#[rustfmt::skip]
pub fn count_primes(nums: &[u32], max_num: u32, alloc: &mut Alloc) -> usize {
    
    match FastPrimeTable::alloc_in(max_num, alloc) {
        Ok(table) => return count_primes_with_table(nums, table),
        Err(OutOfMemory) => {}
    };

    match SmallPrimeTable::alloc_in(max_num, alloc) {
        Ok(table) => return count_primes_with_table(nums, table),
        Err(OutOfMemory) => {}
    };

    count_primes_with_table(nums, RawPrimesTable)
}

fn count_primes_with_table(nums: &[u32], table: impl PrimeTable) -> usize {
    nums.iter().filter(|&&num| table.is_prime(num)).count()
}

trait PrimeTable {
    fn is_prime(&self, n: u32) -> bool;
}

struct SmallPrimeTable<'tab> {
    raw: &'tab [u32],
}

impl<'tab> SmallPrimeTable<'tab> {
    fn alloc_in<'mem>(max_num: u32, alloc: &mut Alloc<'mem>) -> AllocResult<Self>
    where
        'mem: 'tab,
    {
        let n_primes = count_primes_until(max_num);

        let mut primes = RawPrimesIter::new();

        let almost_prime_table =
            alloc.alloc_array_from_fn::<u32>(n_primes, |_| primes.next().unwrap())?;

        Ok(Self {
            raw: almost_prime_table,
        })
    }
}

fn count_primes_until(max_num: u32) -> usize {
    RawPrimesIter::new()
        .take_while(|&prime| prime <= max_num)
        .count()
}

impl<'tab> PrimeTable for SmallPrimeTable<'tab> {
    fn is_prime(&self, n: u32) -> bool {
        self.raw.contains(&n)
    }
}

struct FastPrimeTable<'tab> {
    raw: &'tab [Primality],
}

enum Primality { Prime, Composite }

impl<'tab> FastPrimeTable<'tab> {
    fn alloc_in<'mem>(max_num: u32, alloc: &mut Alloc<'mem>) -> AllocResult<Self>
    where
        'mem: 'tab,
    {
        let almost_prime_table = alloc.alloc_array_from_fn::<Primality>(
            max_num as usize + 1, |_| Primality::Composite
        )?;
        for num in 0..=max_num {
            if is_prime_raw(num) {
                almost_prime_table[num as usize] = Primality::Prime;
            }
        }
        Ok(Self { raw: almost_prime_table })
    }
}

impl<'tab> PrimeTable for FastPrimeTable<'tab> {
    fn is_prime(&self, n: u32) -> bool {
        match self.raw[n as usize] {
            Primality::Prime => true,
            Primality::Composite => false,
        }
    }
}

struct RawPrimesTable;

impl PrimeTable for RawPrimesTable {
    fn is_prime(&self, n: u32) -> bool {
        is_prime_raw(n)
    }
}

struct RawPrimesIter(u32);

impl RawPrimesIter {
    fn new() -> Self {
        Self(1)
    }
}

impl Iterator for RawPrimesIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.0 += 1;

            if is_prime_raw(self.0) {
                return Some(self.0);
            }
        }
    }
}

#[rustfmt::skip]
fn is_prime_raw(num: u32) -> bool {
    if num == 0 || num == 1 { return false; }

    let mut is_prime_flag = true;
    
    for n in 2..=(num / 2) {
        if num % n == 0 {
            is_prime_flag = false;
            break;
        }
    }
    is_prime_flag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_prime_raw_test() {
        assert!(!is_prime_raw(0));
        assert!(!is_prime_raw(1));
        assert!(is_prime_raw(2));
        assert!(is_prime_raw(3));
        assert!(!is_prime_raw(4));
        assert!(is_prime_raw(5));
        assert!(!is_prime_raw(6));
        assert!(is_prime_raw(7));
        assert!(!is_prime_raw(8));
        assert!(!is_prime_raw(9));
        assert!(!is_prime_raw(10));
        assert!(is_prime_raw(11));
        assert!(!is_prime_raw(12));
    }

    #[test]
    fn count_primes_until_test() {
        assert_eq!(count_primes_until(2), 1);
        assert_eq!(count_primes_until(3), 2);
        assert_eq!(count_primes_until(11), 5);
        assert_eq!(count_primes_until(12), 5);
    }

    #[test]
    fn raw_primes_iter_test() {
        let mut primes = RawPrimesIter::new();

        assert_eq!(primes.next(), Some(2));
        assert_eq!(primes.next(), Some(3));
        assert_eq!(primes.next(), Some(5));
        assert_eq!(primes.next(), Some(7));
        assert_eq!(primes.next(), Some(11));
        assert_eq!(primes.next(), Some(13));
        assert_eq!(primes.next(), Some(17));
        assert_eq!(primes.next(), Some(19));
        assert_eq!(primes.next(), Some(23));
    }

    #[test]
    fn raw_primes_table_test() {
        let table = RawPrimesTable;

        assert!(!table.is_prime(0));
        assert!(!table.is_prime(1));
        assert!(table.is_prime(2));
        assert!(table.is_prime(3));
        assert!(!table.is_prime(4));
        assert!(table.is_prime(5));
        assert!(!table.is_prime(6));
        assert!(table.is_prime(7));
        assert!(!table.is_prime(8));
        assert!(!table.is_prime(9));
        assert!(!table.is_prime(10));
        assert!(table.is_prime(11));
        assert!(!table.is_prime(12));
        assert!(table.is_prime(13));
    }

    #[test]
    fn small_primes_table_test() {
        let mut heap: [u8; 1024] = core::array::from_fn(|_| 0);
        let mut alloc = Alloc::new(&mut heap);

        let table = SmallPrimeTable::alloc_in(50, &mut alloc).unwrap();

        assert!(!table.is_prime(0));
        assert!(!table.is_prime(1));
        assert!(table.is_prime(2));
        assert!(table.is_prime(3));
        assert!(!table.is_prime(4));
        assert!(table.is_prime(5));
        assert!(!table.is_prime(6));
        assert!(table.is_prime(7));
        assert!(!table.is_prime(8));
        assert!(!table.is_prime(9));
        assert!(!table.is_prime(10));
        assert!(table.is_prime(11));
        assert!(!table.is_prime(12));
        assert!(table.is_prime(13));
    }

    #[test]
    fn fast_primes_table_test() {
        let mut heap: [u8; 2048] = core::array::from_fn(|_| 0);
        let mut alloc = Alloc::new(&mut heap);

        let table = FastPrimeTable::alloc_in(50, &mut alloc).unwrap();

        assert!(!table.is_prime(0));
        assert!(!table.is_prime(1));
        assert!(table.is_prime(2));
        assert!(table.is_prime(3));
        assert!(!table.is_prime(4));
        assert!(table.is_prime(5));
        assert!(!table.is_prime(6));
        assert!(table.is_prime(7));
        assert!(!table.is_prime(8));
        assert!(!table.is_prime(9));
        assert!(!table.is_prime(10));
        assert!(table.is_prime(11));
        assert!(!table.is_prime(12));
        assert!(table.is_prime(13));
    }

    #[test]
    fn count_primes_test() {
        let mut heap: [u8; 1024] = core::array::from_fn(|_| 0);

        let mut alloc = Alloc::new(&mut heap);

        let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

        let n_primes = count_primes(&nums, 11, &mut alloc);

        assert_eq!(n_primes, 5);
    }

    #[test]
    fn count_primes_multiple_test() {
        let mut heap: [u8; 1024] = core::array::from_fn(|_| 0);

        let mut alloc = Alloc::new(&mut heap);

        let nums = [1, 2, 2, 2, 3, 4, 4, 4, 5, 6, 7, 8, 9, 10, 11];

        let n_primes = count_primes(&nums, 11, &mut alloc);

        assert_eq!(n_primes, 7);
    }

    #[test]
    fn bench_ways() {
        
    }
}
