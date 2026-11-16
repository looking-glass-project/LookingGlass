//! Symbols the PS5 SDK's unwinder lacks but Rust std's backtrace code references.
//! They only affect backtrace symbolisation (a panic prints fewer frames), never
//! normal execution.
use core::ffi::c_void;

/// libgcc returns the start of the function containing `pc`; returning `pc`
/// itself is the documented fallback and is what std tolerates.
#[no_mangle]
pub extern "C" fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void {
    pc
}

/// Canonical frame address of an unwind context; 0 means "unknown".
#[no_mangle]
pub extern "C" fn _Unwind_GetCFA(_ctx: *mut c_void) -> usize {
    0
}

/// pipe2(2) is FreeBSD 10+, but the console's libc does not export it; std's process and
/// thread-wakeup code calls it. pipe + fcntl gives the same result (not atomic, which only
/// matters for fork/exec races; nothing forks on the console).
#[no_mangle]
pub extern "C" fn pipe2(fds: *mut i32, flags: i32) -> i32 {
    extern "C" {
        fn pipe(fds: *mut i32) -> i32;
        fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    }
    const F_SETFD: i32 = 2;
    const F_GETFL: i32 = 3;
    const F_SETFL: i32 = 4;
    const FD_CLOEXEC: i32 = 1;
    const O_NONBLOCK: i32 = 0x0004;
    const O_CLOEXEC: i32 = 0x0010_0000;
    unsafe {
        if pipe(fds) != 0 {
            return -1;
        }
        for i in 0..2 {
            let fd = *fds.add(i);
            if flags & O_CLOEXEC != 0 {
                fcntl(fd, F_SETFD, FD_CLOEXEC);
            }
            if flags & O_NONBLOCK != 0 {
                let fl = fcntl(fd, F_GETFL);
                fcntl(fd, F_SETFL, fl | O_NONBLOCK);
            }
        }
    }
    0
}

/// gai_strerror(3): std calls it to word a DNS lookup error. On the console only modules a
/// title does not load export it (the link would bind it to null), so give the messages here.
#[no_mangle]
pub extern "C" fn gai_strerror(code: i32) -> *const core::ffi::c_char {
    let msg: &'static [u8] = match code {
        1 => b"address family for hostname not supported\0",
        2 => b"temporary failure in name resolution\0",
        3 => b"invalid value for ai_flags\0",
        4 => b"non-recoverable failure in name resolution\0",
        5 => b"ai_family not supported\0",
        6 => b"memory allocation failure\0",
        8 => b"hostname nor servname provided, or not known\0",
        9 => b"servname not supported for ai_socktype\0",
        10 => b"ai_socktype not supported\0",
        11 => b"system error returned in errno\0",
        _ => b"unknown name resolution error\0",
    };
    msg.as_ptr() as *const core::ffi::c_char
}
