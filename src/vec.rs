use core::ops::{Deref, DerefMut};

use super::*;

pub struct Vec<'arr, 'all, T> {
    alloc: Alloc<'all>,
    arr: &'arr mut [T],
    initialized: usize,
}

#[rustfmt::skip]
impl<'arr, 'all, T: Default> Vec<'arr, 'all, T>
where
    'all: 'arr,
{
    pub fn with_capacity(cap: usize, mut alloc: Alloc<'all>) -> AllocResult<Self> {
        let arr = alloc.alloc_arr_from_fn(cap, |_| T::default())?;

        Ok(Self {
            initialized: 0,
            arr,
            alloc,
        })
    }

    pub fn capacity(&self) -> usize {
        self.arr.len()
    }

    pub fn len(&self) -> usize {
        self.initialized
    }

    pub fn push(&mut self, item: T) -> AllocResult<()> {
        if self.len() < self.capacity() {
            self.arr[self.len()] = item;
        } 
        else {
            let new_arr = self
                .alloc
                .alloc_arr_from_fn(self.len() * 2, |_| T::default())?;

            for i in 0..self.arr.len() {
                new_arr[i] = core::mem::take(&mut self.arr[i]);
            }
            new_arr[self.arr.len()] = item;

            self.arr = new_arr;
        }
        self.initialized += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() > 0 {
            self.initialized -= 1;
            Some(core::mem::take(&mut self.arr[self.len()]))
        }
        else {
            None
        }
    }
}

impl<'arr, 'all, T> Deref for Vec<'arr, 'all, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.arr[0..self.initialized]
    }
}

impl<'arr, 'all, T> DerefMut for Vec<'arr, 'all, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arr[0..self.initialized]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn push_within_capacity() {
        let mut heap: [u8; 32] = core::array::from_fn(|_| 9);

        assert_eq!(
            heap,
            [
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );

        {
            let alloc = Alloc::new(&mut heap);

            let mut vec = Vec::<i16>::with_capacity(4, alloc).unwrap();
            vec.push(1).unwrap();
            vec.push(1).unwrap();
            vec.push(1).unwrap();
            vec.push(1).unwrap();
        }

        assert_eq!(
            heap,
            [
                1, 0, 1, 0, 1, 0, 1, 0, 
                9, 9, 9, 9, 9, 9, 9, 9, 
                9, 9, 9, 9, 9, 9, 9, 9, 
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );
    }

    #[test]
    fn push_beyond_capacity() {
        let mut heap: [u8; 32] = core::array::from_fn(|_| 9);

        assert_eq!(
            heap,
            [
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );

        {
            let alloc = Alloc::new(&mut heap);

            let mut vec = Vec::<i16>::with_capacity(4, alloc).unwrap();
            vec.push(1).unwrap();
            vec.push(2).unwrap();
            vec.push(3).unwrap();
            vec.push(4).unwrap();
            vec.push(5).unwrap();
        }

        assert_eq!(
            heap,
            [
                0, 0, 0, 0, 0, 0, 0, 0, // forgotten and zeroed by mem::take()
                1, 0, 2, 0, 3, 0, 4, 0, 
                5, 0, 0, 0, 0, 0, 0, 0,
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );
    }

    #[test]
    fn pop_works() {
        let mut heap: [u8; 32] = core::array::from_fn(|_| 9);

        assert_eq!(
            heap,
            [
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );

        {
            let alloc = Alloc::new(&mut heap);

            let mut vec = Vec::<i16>::with_capacity(4, alloc).unwrap();
            vec.push(1).unwrap();
            vec.push(2).unwrap();

            assert_eq!(vec.pop(), Some(2));
            assert_eq!(vec.pop(), Some(1));
            assert_eq!(vec.pop(), None);
        }

        assert_eq!(
            heap,
            [
                0, 0, 0, 0, 0, 0, 0, 0,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
                9, 9, 9, 9, 9, 9, 9, 9,
            ]
        );
    }
}
