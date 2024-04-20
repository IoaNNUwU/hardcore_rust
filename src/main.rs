use hardcore::*;

fn main() {
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

