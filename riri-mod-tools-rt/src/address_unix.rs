#[derive(Debug, Clone)]
pub struct ProcessInfo {
    process: libc::pid_t
}

impl ProcessInfo {
    pub fn get_current_process() -> Result<Self> {
        let process = libc::getpid();
        Ok(Self { process })
    }
}

pub fn get_platform_thread_id() -> u64 {
    (unsafe { libc::gettid() }) as u64
}
