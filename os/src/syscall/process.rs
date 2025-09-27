//! Process management syscalls
use crate::{
    mm::translated_byte_buffer,
    timer::get_time_us,
    task::{change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_count, suspend_current_and_run_next}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    
    let tv = TimeVal {
        sec: us / 1000000,
        usec: us % 1000000,
    };

    let tv_size = core::mem::size_of::<TimeVal>();
    let tv_bytes = unsafe {
        core::slice::from_raw_parts(
            &tv as *const TimeVal as *const u8,
            tv_size)
    };

    
    let mut bufs = translated_byte_buffer(
        current_user_token(), 
        ts as *const u8, tv_size);
    
    // trace!("kernel: sys_get_time - 3ab8 - {} {} {}", bufs.len(), tv_size, bufs[0].len());

    if bufs.len() == 1 {
        bufs[0][..tv_size].copy_from_slice(&tv_bytes[..tv_size]);
    } else {
        let mut offset = 0;
        for buf in bufs.iter_mut() {
            let buf_len = buf.len().min(tv_size - offset);
            buf[..buf_len].copy_from_slice(&tv_bytes[offset..offset + buf_len]);
            offset += buf_len;
        }
    }
    
    // trace!("kernel: sys_get_time finish");
    0
}


/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let bufs = translated_byte_buffer(
                current_user_token(), id as *const u8, 1);
            if bufs.len() == 0 {
                -1
            } else {
                bufs[0][0].into()
            }
        },
        1 => {
            let mut bufs = translated_byte_buffer(
                current_user_token(), id as *const u8, 1);
            if bufs.len() == 0 {
                -1
            } else {
                bufs[0][0] = data as u8;
                0
            }
        },
        2 => {
            let i = get_syscall_count(id) as isize;
            trace!("get_syscall_count {} {}", id, i);
            i
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
