mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_SYSCALL_NUM};
use crate::loader::{};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use crate::timer::get_time_us;
use core::arch::asm;
use lazy_static::*;
use alloc::vec::Vec;
use crate::loader::{get_num_app, get_app_data};
use crate::mm::{VirtAddr, MapPermission};

pub use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};
pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}


struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

pub struct TaskInfo {
    status: TaskStatus,
    syscall_times: [u32; MAX_SYSCALL_NUM],
    time: usize,
}

lazy_static! {
    
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks : Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            // println!("{}th app's tcb is being created",i);
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ));
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            if inner.tasks[next].task_start_time==0 {
                inner.tasks[next].task_start_time = get_time_us();
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }

    fn get_current_task_info(&self, ti:*mut TaskInfo) -> usize {
        let inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        unsafe{
            *ti = TaskInfo{
                status: inner.tasks[current_task].task_status,
                syscall_times: inner.tasks[current_task].task_syscall_times,
                time: (get_time_us() - inner.tasks[current_task].task_start_time) / 1000,
            }
        }
        0
    }

    fn increase_task_syscall(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        inner.tasks[current_task].task_syscall_times[syscall_id] += 1;
    }
    
    fn current_m_map(&self, start: VirtAddr, len: usize, perm: MapPermission) -> isize{
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        inner.tasks[current_task].memory_set.mmap(start, len, perm)
    }

    fn get_curren_task_note(&self) -> usize {
        let mut inner = self.inner.exclusive_access();
        let current_task = inner.current_task;
        drop(inner);
        current_task
    }
}



pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn current_task_note() -> usize {
    TASK_MANAGER.get_curren_task_note()
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

pub fn get_task_info(ti: *mut TaskInfo) -> usize {
    TASK_MANAGER.get_current_task_info(ti)
}

pub fn increase_task_syscall_times(syscall_id: usize) {
    TASK_MANAGER.increase_task_syscall(syscall_id);
}

pub fn current_mmap(start: VirtAddr, len: usize, perm: MapPermission) -> isize {
    TASK_MANAGER.current_m_map(start, len, perm)
}