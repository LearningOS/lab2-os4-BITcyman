mod heap_allocator;
mod address;
mod page_table;


pub use heap_allocator::init_heap;
pub use heap_allocator::heap_test;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}



pub fn init(){
    heap_allocator::init_heap();
    // heap_allocator::heap_test();
}