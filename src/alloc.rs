use core::mem;

pub struct Alloc<'mem> {
    remaining_memory: &'mem mut [u8],
}

#[rustfmt::skip]
impl<'mem> Alloc<'mem> {

    pub fn new(heap: &'mem mut [u8]) -> Self {
        Alloc { remaining_memory: heap }
    }

    pub fn alloc<'item, T>(&mut self, item: T) -> AllocResult<&'item mut T>
    where
        'mem: 'item,
    {
        let align = mem::align_of::<T>();

        self.waste_some_mem_to_reach_align(align)?;

        unsafe { self.alloc_aligned(item) }
    }

    pub fn alloc_arr<'arr, T: Default, const SIZE: usize>(&mut self, arr: [T; SIZE]) -> AllocResult<&'arr mut [T]>
    where 
        'mem: 'arr,
    {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        self.waste_some_mem_to_reach_align(align)?;

        if self.remaining_memory.len() < size * arr.len() {
            return Err(OutOfMemory)
        }

        let almost_arr_ptr = self.remaining_memory as *mut [u8] as *mut [T];

        for item in arr {
            unsafe { self.alloc_aligned(item).unwrap_unchecked(); }
        }

        let arr_ref = &mut unsafe { &mut *almost_arr_ptr }[0..SIZE];

        Ok(arr_ref)
    }

    pub fn alloc_arr_from_fn<'arr, T>(
        &mut self, 
        arr_size: usize, 
        init_item_fn: impl Fn(usize) -> T) -> AllocResult<&'arr mut [T]>
    where 
        'mem: 'arr,
    {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        self.waste_some_mem_to_reach_align(align)?;

        if self.remaining_memory.len() < size * arr_size {
            return Err(OutOfMemory)
        }

        let almost_arr_ptr = self.remaining_memory as *mut [u8] as *mut [T];

        for i in 0..arr_size {
            unsafe { self.alloc_aligned(init_item_fn(i)).unwrap_unchecked(); }
        }

        let arr_ref = &mut unsafe { &mut *almost_arr_ptr }[0..arr_size];

        Ok(arr_ref)
    }

    fn waste_some_mem_to_reach_align(&mut self, align: usize) -> AllocResult<()> {

        let how_many_bytes_to_waste: usize = {

            let remainig_memory_ptr = self.remaining_memory.as_ptr() as usize;

            let mut temp = remainig_memory_ptr;

            while temp % align != 0 {
                temp += 1; // There is more efficient way
            }
            temp - remainig_memory_ptr
        };

        if self.remaining_memory.len() < how_many_bytes_to_waste {
            return Err(OutOfMemory);
        }

        let remaining_memory = mem::take(&mut self.remaining_memory);
        let (_, remaining_memory) = remaining_memory.split_at_mut(how_many_bytes_to_waste);
        self.remaining_memory = remaining_memory;

        Ok(())
    }

    // SAFETY: This function has to be called only after
    // self.remaining_memory was aligned for T.
    unsafe fn alloc_aligned<'item, T>(&mut self, item: T) -> AllocResult<&'item mut T>
    where
        'mem: 'item,
    {
        let size = mem::size_of::<T>();

        if self.remaining_memory.len() < size {
            return Err(OutOfMemory);
        }

        let remaining_memory = mem::take(&mut self.remaining_memory);

        let (almost_item_ref, remaining_memory) = remaining_memory.split_at_mut(size);

        self.remaining_memory = remaining_memory;

        let item_ref: &mut T = {
            let almost_item_ptr = almost_item_ref as *mut [u8] as *mut T;

            unsafe {
                core::ptr::write(almost_item_ptr, item);
                &mut *almost_item_ptr // *mut T as &mut T
            }
        };

        Ok(item_ref)
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
        alloc.alloc::<u8>(3).unwrap();
        alloc.alloc::<u8>(4).unwrap();

        assert_eq!(pseudo_heap, [1, 2, 3, 4])
    }

    #[test]
    fn works_with_i64() {
        let mut pseudo_heap: [u8; 16] = core::array::from_fn(|_| 9);

        let mut alloc = Alloc::new(&mut pseudo_heap);

        let _ = alloc.alloc::<i64>(1).unwrap();
        let _ = alloc.alloc::<i64>(2).unwrap();

        assert_eq!(pseudo_heap, 
            [
                1, 0, 0, 0, 0, 0, 0, 0,
                2, 0, 0, 0, 0, 0, 0, 0,
            ]
        );
    }

    #[test]
    fn works_with_alignment() {

        let mut pseudo_heap: [u8; 16] = core::array::from_fn(|_| 9);

        let mut alloc = Alloc::new(&mut pseudo_heap);

        let _ = alloc.alloc::<u8>(1).unwrap();
        let _ = alloc.alloc::<u16>(1).unwrap();

        assert_eq!(pseudo_heap,
            [
                1, 9, 1, 0,
                9, 9, 9, 9,
                9, 9, 9, 9,
                9, 9, 9, 9,
            ]
        );
    }

    #[test]
    fn alloc_arr_fn_works() {

        let mut pseudo_heap: [u8; 16] = core::array::from_fn(|_| 9);

        let mut alloc = Alloc::new(&mut pseudo_heap);

        let _ = alloc.alloc::<u8>(1).unwrap();
        let _ = alloc.alloc_arr_from_fn::<u16>(3, |i| i as u16).unwrap();

        assert_eq!(pseudo_heap,
            [
                1, 9, 0, 0, 1, 0, 2, 0,
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );
    }

}
