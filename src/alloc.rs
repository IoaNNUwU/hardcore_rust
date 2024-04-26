use core::mem;

pub struct Alloc<'mem> {
    remaining_mem: &'mem mut [u8],
}

#[rustfmt::skip]
impl<'mem> Alloc<'mem> {

    pub fn new(heap: &'mem mut [u8]) -> Self {
        Alloc { remaining_mem: heap }
    }

    pub fn alloc<'item, T>(&mut self, item: T) -> 
     AllocResult<&'item mut T>
    where
        'mem: 'item
    {
        self.waste_some_mem_to_reach_align::<T>()?;

        unsafe { self.alloc_aligned(item) }
    }

    fn waste_some_mem_to_reach_align<T>(&mut self) -> AllocResult<()> {

        let align = mem::align_of::<T>();

        let how_many_bytes_to_waste: usize = {

            let remainig_memory_ptr = self.remaining_mem.as_ptr() as usize;

            let mut temp = remainig_memory_ptr;

            while temp % align != 0 {
                temp += 1; // There is more efficient way
            }
            temp - remainig_memory_ptr
        };

        if self.remaining_mem.len() < how_many_bytes_to_waste {
            return Err(OutOfMemory);
        }

        let remaining_memory = mem::take(&mut self.remaining_mem);
        let (_, remaining_memory) = remaining_memory.split_at_mut(how_many_bytes_to_waste);
        self.remaining_mem = remaining_memory;

        Ok(())
    }

    // SAFETY: This function has to be called only after
    // self.remaining_memory was aligned for T.
    unsafe fn alloc_aligned<'item, T>(&mut self, item: T) -> AllocResult<&'item mut T>
    where
        'mem: 'item
    {
        let size = mem::size_of::<T>();

        if self.remaining_mem.len() < size {
            return Err(OutOfMemory);
        }

        let remaining_mem = mem::take(&mut self.remaining_mem);

        let (almost_item_ref, remaining_memory) = remaining_mem.split_at_mut(size);

        self.remaining_mem = remaining_memory;

        let item_ref: &mut T = {
            let almost_item_ptr = almost_item_ref as *mut [u8] as *mut T;

            unsafe {
                core::ptr::write(almost_item_ptr, item);
                &mut *almost_item_ptr // *mut T as &mut T
            }
        };
        Ok(item_ref)
    }

    pub fn alloc_array_from_fn<'arr, T>(
        &mut self, size: usize, mut init_t: impl FnMut(usize) -> T
    ) 
            -> AllocResult<&'arr mut [T]>
    where 'mem: 'arr {

        self.waste_some_mem_to_reach_align::<T>()?;

        if self.remaining_mem.len() < size {
            return Err(OutOfMemory);
        }

        let arr_ptr = self.remaining_mem as *mut [u8] as *mut [T];

        for i in 0..size {
            let _ = unsafe { 
                self.alloc_aligned(init_t(i)).unwrap_unchecked() 
            };
        };

        Ok(&mut unsafe { &mut *arr_ptr }[0..size])
    }
}

pub type AllocResult<T> = core::result::Result<T, OutOfMemory>;

#[derive(Debug)]
pub struct OutOfMemory;

#[rustfmt::skip]
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn works_with_bytes() {
        let mut pseudo_heap: [u8; 4] = core::array::from_fn(|_| 9);

        let mut alloc = Alloc::new(&mut pseudo_heap);

        alloc.alloc::<u8>(1).unwrap();
        alloc.alloc::<u8>(2).unwrap();

        assert_eq!(pseudo_heap, [1, 2, 9, 9])
    }

    #[test]
    fn works_with_i64() {
        let mut pseudo_heap: [u8; 32] = core::array::from_fn(|_| 9);

        let mut alloc = Alloc::new(&mut pseudo_heap);

        let _ = alloc.alloc::<i64>(1).unwrap();
        let _ = alloc.alloc::<i64>(2).unwrap();

        assert_eq!(pseudo_heap, 
            [
                1, 0, 0, 0, 0, 0, 0, 0,
                2, 0, 0, 0, 0, 0, 0, 0,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );
    }

    #[test]
    fn works_with_alignment() {

        let mut pseudo_heap: [u8; 16] = core::array::from_fn(|_| 9);

        let mut alloc = Alloc::new(&mut pseudo_heap);

        let u8_addr = alloc.alloc::<u8>(1).unwrap();

        assert!(u8_addr as *mut u8 as usize % 2 == 0);

        let _ = alloc.alloc::<u16>(2).unwrap();

        assert_eq!(pseudo_heap,
            [
                1, 9, 2, 0,
                9, 9, 9, 9,
                9, 9, 9, 9,
                9, 9, 9, 9,
            ]
        );
    }

    #[test]
    fn works_with_arrays() {

        let mut pseudo_heap: [u8; 16] = core::array::from_fn(|_| 9);

        let mut alloc = Alloc::new(&mut pseudo_heap);

        let _ = alloc.alloc::<u8>(1).unwrap();

        let _  = alloc.alloc_array_from_fn::<u16>(3, |_| 2).unwrap();

        assert_eq!(pseudo_heap,
            [
                1, 9, 2, 0,
                2, 0, 2, 0,
                9, 9, 9, 9,
                9, 9, 9, 9,
            ]
        );
    }
}
