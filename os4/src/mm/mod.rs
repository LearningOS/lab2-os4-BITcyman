mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;

pub use memory_set::KERNEL_SPACE, remap_test;
pub use heap_allocator::heap_test;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, VPNRange};
pub use page_table::{PageTable, PageTableEntry, PTEFlags};
pub use frame_allocator::{FrameTracker, frame_alloc };



pub fn init(){
    heap_allocator::init_heap();
    // heap_allocator::heap_test();
    frame_allocator::init_frame_allocator();
    // frame_allocator::frame_allocator_test();
    KERNEL_SPACE.exclusive_access().activate();
    remap_test();
}