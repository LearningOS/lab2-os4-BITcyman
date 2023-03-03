//! Process management syscalls

use crate::config::{MAX_SYSCALL_NUM};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_user_token, TaskInfo, get_task_info, current_mmap};
use crate::mm::{MapPermission, PageTable, VirtAddr, PhysAddr};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}



/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}
pub fn get_pa(va: usize) -> usize{ 
    let current_token = current_user_token();
    let pt: PageTable = PageTable::from_token(current_token);
    let va: VirtAddr = VirtAddr::from(va);  
    let ppn = pt.translate(va.floor()).unwrap().ppn();
    let pa: PhysAddr = ppn.into();
    let pa: usize = pa.into();
    pa + va.page_offset()
}


// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_us();
    let pa = get_pa(_ts as usize);
    unsafe{
        let time = pa as *mut TimeVal;
        * time = TimeVal {
            sec: _us / 1_000_000,
            usec: _us % 1_000_000,
        }
    }

    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    let va = VirtAddr(_start);
    if !va.is_align() {
        return -1
    }
    if ((_port & 0x7) == 0) || _port > 7 {
        return -1
    }
    let perm = MapPermission::from_bits(((_port<<1) + 16) as u8).unwrap();
    current_mmap(va, _len, perm)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    -1
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let pa = get_pa(ti as usize);
    if get_task_info(pa as *mut TaskInfo) == 0 {
        return 0 
    }
    -1  //  失败
}

