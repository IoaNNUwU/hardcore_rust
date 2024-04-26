use hardcore::*;

fn main() {
    print_allocated_after_alloc_was_dropped();
    visualize_layout();
}

pub fn visualize_layout() {
    let mut heap: [u8; 64] = std::array::from_fn(|_| 9);

    let mut alloc = Alloc::new(&mut heap);

    let _: &u8 = alloc.alloc(8).unwrap();
    let _: &u64 = alloc.alloc(64).unwrap();
    let _: &u16 = alloc.alloc(16).unwrap();
    let _: &u32 = alloc.alloc(32).unwrap();

    println!("heap: {:?}", heap);
}

fn print_allocated_after_alloc_was_dropped() {
    let mut heap: [u8; 32] = Default::default();
    let mut alloc = Alloc::new(&mut heap);

    let allocated_num = alloc.alloc::<u32>(873).unwrap();

    drop(alloc);

    println!("num: {allocated_num}");
}
