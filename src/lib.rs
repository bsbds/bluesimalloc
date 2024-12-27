use std::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
};

use buddy_system_allocator::LockedHeap;

pub use ctor;

const ORDER: usize = 32;
const SHM_PATH: &str = "/bluesim1\0";
const HEAP_BLOCK_SIZE: usize = 1024 * 1024 * 64;
pub static mut HEAP_START_ADDR: usize = 0;

/// Handle to the allocator
///
/// This type implements the `GlobalAlloc` trait, allowing usage a global allocator.
pub struct BlueSimalloc(LockedHeap<ORDER>);

impl BlueSimalloc {
    pub const fn new() -> Self {
        Self(LockedHeap::new())
    }
}

impl Default for BlueSimalloc {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for BlueSimalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.alloc(layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.0.alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.0.realloc(ptr, layout, new_size)
    }
}

#[macro_export]
macro_rules! setup_allocator {
    () => {
        use $crate::ctor;

        #[global_allocator]
        static HEAP_ALLOCATOR: $crate::BlueSimalloc = $crate::BlueSimalloc::new();

        #[ctor::ctor]
        fn init_global_allocator() {
            $crate::init_global_allocator(&HEAP_ALLOCATOR);
        }
    };
}

pub fn init_global_allocator(allocator: &BlueSimalloc) {
    unsafe {
        let shm_fd = libc::shm_open(
            SHM_PATH.as_ptr() as *const libc::c_char,
            libc::O_RDWR,
            0o600,
        );
        if shm_fd == -1 {
            libc::exit(shm_fd);
        }
        assert!(shm_fd != -1, "shm_open failed");

        let heap = libc::mmap(
            0x7f7e8e600000 as *mut c_void,
            HEAP_BLOCK_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            shm_fd,
            0,
        );

        let addr = heap as usize;
        let size = HEAP_BLOCK_SIZE;
        HEAP_START_ADDR = addr;

        allocator.0.lock().init(addr, size);
    }
}
