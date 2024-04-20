#![no_std]
#![cfg(target_endian = "little")]

mod alloc;
pub use alloc::*;

mod vec;
pub use vec::*;
