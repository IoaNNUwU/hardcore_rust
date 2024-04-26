use diol::prelude::*;

use hardcore::{count_primes, Alloc};

fn main() -> std::io::Result<()> {
    let mut bench = Bench::new(BenchConfig::from_args());

    bench.register(raw_primes_table, [1]);
    bench.register(fast_primes_table, [1]);
    bench.register(small_primes_table, [1]);

    bench.run()?;
    Ok(())
}

use const_array_init::make_const_arr;

make_const_arr!(NUMS, [u32; 1024], |i| i as u32 / 20);

fn raw_primes_table(bencher: Bencher, _: i32) {
    let mut alloc = Alloc::new(&mut []);

    let max = *NUMS.iter().max().unwrap();

    bencher.bench(|| {
        black_box(count_primes(&NUMS, max, &mut alloc));
    });
}

fn fast_primes_table(bencher: Bencher, _: i32) {
    let mut heap: [u8; 4096] = std::array::from_fn(|_| 0);
    let mut alloc = Alloc::new(&mut heap);

    let max = *NUMS.iter().max().unwrap();

    bencher.bench(|| {
        black_box(count_primes(&NUMS, max, &mut alloc));
    });
}

fn small_primes_table(bencher: Bencher, _: i32) {
    let mut heap: [u8; 2048] = std::array::from_fn(|_| 0);
    let mut alloc = Alloc::new(&mut heap);

    let max = *NUMS.iter().max().unwrap();

    bencher.bench(|| {
        black_box(count_primes(&NUMS, max, &mut alloc));
    });
}