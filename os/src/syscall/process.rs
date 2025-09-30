//! Process management syscalls
use crate::{
    config::PAGE_SIZE, 
    mm::{translated_byte_buffer, MapPermission, VirtAddr, PageTable}, 
    task::{change_program_brk, current_task_id, current_user_token, exit_current_and_run_next, get_syscall_count, mmap, munmap, suspend_current_and_run_next}, timer::get_time_us
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
    // trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    // trace!("kernel: sys_get_time");
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
            let token = current_user_token();
            let pgtbl = PageTable::from_token(token);
            let vpn = VirtAddr::from(id).floor().into();
            if let Some(pte) = pgtbl.translate(vpn) {
                if pte.is_user() && pte.readable() {
                    let pa = pte.ppn().0 << 12 | (id & 0xfff);
                    let ptr = pa as *const u8;
                    let val = unsafe { *ptr };
                    return val as isize;
                }   
            }
            -1
        },
        1 => {
            let token = current_user_token();
            let pgtbl = PageTable::from_token(token);
            let vpn = VirtAddr::from(id).floor().into();
            if let Some(pte) = pgtbl.translate(vpn) {
                if pte.is_user() && pte.writable() {
                    let pa = pte.ppn().0 << 12 | (id & 0xfff);
                    let ptr = pa as *mut u8;
                    unsafe {
                        ptr.write(data as u8);
                    }
                    return 0;       
                }   
            }
            -1
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
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    // [ch4] 检查页边界对其
    if start % PAGE_SIZE != 0 {
        trace!("kernel: sys_mmap {} {:x}/{:x}/{:x} {}", current_task_id(), start, len, port, -1);
        return -1;
    }

    // [ch4] 检查port合法性
    if port > 0x7 || port & 0x7 == 0 {
        trace!("kernel: sys_mmap {} {:x}/{:x}/{:x} {}", current_task_id(), start, len, port, -1);
        return -1;
    }

    // [ch4] port转换成MapPermission
    let mut map_perm =  MapPermission::U;
    if port & 0x1 != 0{
        map_perm |= MapPermission::R;
    }
    if port & 0x2 != 0{
        map_perm |= MapPermission::W;
    }
    if port & 0x4 != 0{
        map_perm |= MapPermission::X;
    }

    let i = mmap(start, len, map_perm);
    trace!("kernel: sys_mmap {} {:x}/{:x}/{:x} {}", current_task_id(), start, len, port, i);
    i
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {  
    trace!("kernel: sys_munmap {} {start:x}/{len:x}", current_task_id());
    // 检查参
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    // len 可以为0，如果为0直接返回成功
    if len == 0 {
        return 0;
    }

    trace!("kernel: sys_munmap-1 {} {start:x}/{len:x}", current_task_id());
    // size 对其页面边界
    let len = (len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
    trace!("kernel: sys_munmap-2 {} {start:x}/{len:x}", current_task_id());

    let r = munmap(start, len);
    trace!("kernel: sys_munmap {} {start:x}/{len:x} {r}", current_task_id());
    r
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
