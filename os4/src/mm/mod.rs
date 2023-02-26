mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
mod memory_set;

pub use memory_set::{MemorySet, MapPermission, KERNEL_SPACE, remap_test};
pub use heap_allocator::heap_test;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, VPNRange, StepByOne};
pub use page_table::{PageTable, PageTableEntry, PTEFlags, translated_byte_buffer};
pub use frame_allocator::{FrameTracker, frame_alloc };



pub fn init(){
    heap_allocator::init_heap();
    // heap_allocator::heap_test();
    frame_allocator::init_frame_allocator();
    // frame_allocator::frame_allocator_test();
    KERNEL_SPACE.exclusive_access().activate();
    remap_test();
}