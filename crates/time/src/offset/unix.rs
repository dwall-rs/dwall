pub fn system_utc_offset_secs() -> i32 {
    unsafe {
        let ts = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&ts, &mut tm);
        tm.tm_gmtoff as i32
    }
}
