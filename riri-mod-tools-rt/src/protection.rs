use bitflags::bitflags;
#[cfg(target_os = "windows")]
use windows::Win32::System::Memory;
#[cfg(target_os = "linux")]
use libc::c_int;

bitflags! {
    #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
    pub struct PageProtection : u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
    }
}

#[cfg(target_os = "linux")]
impl From<PageProtection> for c_int {
    fn from(value: PageProtection) -> Self {
        value.bits() as _
    }
}

#[cfg(target_os = "windows")]
impl From<PageProtection> for Memory::PAGE_PROTECTION_FLAGS {
    fn from(value: PageProtection) -> Self {
        if value.contains(PageProtection::EXECUTE) {
            if value.contains(PageProtection::WRITE) {
                Memory::PAGE_EXECUTE_READWRITE
            } else if value.contains(PageProtection::READ) {
                Memory::PAGE_EXECUTE_READ
            } else {
                Memory::PAGE_EXECUTE
            }
        } else if value.contains(PageProtection::WRITE) {
            Memory::PAGE_READWRITE
        } else if value.contains(PageProtection::READ) {
            Memory::PAGE_READONLY
        } else {
            Memory::PAGE_NOACCESS
        }
    }
}