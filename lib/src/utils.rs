

pub const unsafe fn slice_to_array<T, const SIZE: usize>(value: &[T]) -> &[T; SIZE] {
    &*(value as *const [T]).cast()
}

pub const unsafe fn slice_to_array_mut<T, const SIZE: usize>(value: &mut [T]) -> &mut [T; SIZE] {
    &mut *(value as *mut [T]).cast()
}

pub const fn index_array<T, const SIZE: usize, const INDEX_SIZE: usize>(value: &[T; SIZE], at: usize) -> &[T; INDEX_SIZE] {
    unsafe { slice_to_array(&value[at .. at + INDEX_SIZE]) }
}

pub const fn index_array_mut<T, const SIZE: usize, const INDEX_SIZE: usize>(value: &mut [T; SIZE], at: usize) -> &mut [T; INDEX_SIZE] {
    unsafe { slice_to_array_mut(&mut value[at .. at + INDEX_SIZE]) }
}