use std::ptr::NonNull;

pub struct Vec<T> {
    ptr: NonNull<T>, // 指向数组第一个元素的指针
    capacity: usize, // 数组的容量
    len: usize, // 数组的长度
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(std::mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        Vec {
            ptr: NonNull::dangling(),
            capacity: 0,
            len: 0,
        }
    }
}