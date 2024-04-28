use std::{alloc::{self, dealloc, Layout}, mem::ManuallyDrop, ops::{Deref, DerefMut}, ptr::{self, NonNull}};


pub struct Vec<T> {
    // ptr: NonNull<T>, // 指向数组第一个元素的指针
    // capacity: usize, // 数组的容量
    buf: RawVec<T>,
    len: usize, // 数组的长度
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn capacity(&self) -> usize {
        self.buf.capacity
    }

    pub fn new() -> Self {
        assert!(std::mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        Vec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    // pub fn grow(&mut self) {
    //     let (new_size, new_layout) = if self.capacity == 0 {
    //         (1, Layout::array::<T>(1).unwrap())
    //     } else {
    //         let new_cap = self.capacity * 2;
    //         let new_layout = Layout::array::<T>(new_cap).unwrap();
    //         (new_cap, new_layout)
    //     };
    //     assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");
        
    //     let new_ptr = if self.capacity == 0 {
    //         unsafe { alloc::alloc(new_layout) }
    //     } else {
    //         let old_ptr = self.ptr.as_ptr() as *mut u8;
    //         let old_layout = Layout::array::<T>(self.capacity).unwrap();
    //         unsafe { alloc::realloc(old_ptr, old_layout, new_size)}
    //     };
    //     self.ptr = match NonNull::new(new_ptr as *mut T) {
    //         Some(p) => p,
    //         None => alloc::handle_alloc_error(new_layout),
    //     };
    //     self.capacity = new_size;
    // }
    
    pub fn push(&mut self, elem: T) {
        if self.len == self.capacity() {
            self.buf.grow();
        }
        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len)))}
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "Index out of bounds");
        if self.len == self.capacity() {
            self.buf.grow();
        }
        unsafe {
            // ptr::copy(src, dest, len) 的含义： "从 src 复制连续的 len 个元素到 dst "
            ptr::copy(self.ptr().add(index), self.ptr().add(index + 1), self.len - index);
            ptr::write(self.ptr().add(index), elem);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        unsafe {
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(self.ptr().add(index + 1), self.ptr().add(index), self.len - index - 1);
            self.len -= 1;
            result
        }
    }

}

impl <T> Drop for Vec<T>  {
    fn drop(&mut self) {
        if self.capacity() != 0 {
            while let Some(_) = self.pop() {}
            let layout = Layout::array::<T>(self.capacity()).unwrap();
            unsafe {
                alloc::dealloc(self.ptr() as *mut u8, layout)
            }
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr(), self.len)
        }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr(), self.len)
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let vec = ManuallyDrop::new(self);
        let buf = unsafe {
            ptr::read(&self.buf)
        };
        let len = vec.len();

        IntoIter {
            start: buf.ptr.as_ptr(),
            end: if buf.capacity == 0 {
                buf.ptr.as_ptr()
            } else {
                unsafe {buf.ptr.as_ptr().add(len) }
            },
            _buf: buf,

        }
    }
}



pub struct IntoIter<T> {
    _buf: RawVec<T>,
    start: *const T,
    end: *const T,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        } else {
            unsafe {
                let item = ptr::read(self.start);
                self.start = self.start.add(1);
                Some(item)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / std::mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        } else {
            unsafe {
                self.end = self.end.sub(1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // 我们只需要确保 Vec 中所有元素都被读取了，
        // 在这之后这些元素会被自动清理
        for _ in &mut *self {}
    }
}



struct RawVec<T> {
    ptr: NonNull<T>,
    capacity: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        assert!(std::mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        RawVec {
            ptr: NonNull::dangling(),
            capacity: 0,
        }
    }

    pub fn grow(&mut self) {
        // 保证新申请的内存没有超出 `isize::MAX` 字节
        let new_cap = if self.capacity == 0 {
            1
        } else {
            self.capacity * 2
        };
        // `Layout::array` 会检查申请的空间是否小于等于 usize::MAX，
        // 但是因为 old_layout.size() <= isize::MAX，
        // 所以这里的 unwrap 永远不可能失败
        let new_layout = Layout::array::<T>(new_cap).unwrap();
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_cap) }
        };
        // 如果分配失败，`new_ptr` 就会成为空指针，我们需要对应 abort 的操作
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.capacity = new_cap;
    }
}

impl <T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            let layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}


