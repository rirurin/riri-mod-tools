use std::error::Error;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use libc;
use crate::protection::PageProtection;

#[cfg(feature = "reloaded")]
#[link(name = "riri_mod_runtime_reloaded", kind = "dylib")]
unsafe extern "C" {
    pub(crate) fn get_executable_hash_ex() -> u64;
}

#[derive(Debug)]
pub struct ProcessModule {
    owner: *const libc::dl_phdr_info,
    base_address: usize,
    module_size: usize,
    hash: u64,
    sections: Vec<MemorySection>,
}

impl ProcessModule {
    fn get_memory_sections(info: &libc::dl_phdr_info) -> Vec<MemorySection> {
        let mut out = vec![];
        for i in 0..info.dlpi_phnum as usize {
            out.push(MemorySection::from_section_header(info.dlpi_addr as _,unsafe { &*info.dlpi_phdr.add(i) }));
        }
        out
    }

    fn get_module_size(info: &libc::dl_phdr_info) -> usize {
        let mut end = 0;
        for i in 0..info.dlpi_phnum as usize {
            let phdr = unsafe { &*info.dlpi_phdr.add(i) };
            end = (phdr.p_vaddr + phdr.p_memsz) as usize;
        }
        end
    }

    pub fn get_base_address(&self) -> usize { self.base_address }
    pub fn get_memory_size(&self) -> usize { self.module_size }
    pub fn as_raw(&self) -> &libc::dl_phdr_info { unsafe { &*self.owner } }
}

unsafe extern "C" fn iterate_cb(info: *mut libc::dl_phdr_info, size: usize, data: *mut std::ffi::c_void) -> std::ffi::c_int {
    let data = unsafe { &mut *(data as *mut ProcessInfo) };
    let exec = &mut data.executable;
    exec.owner = info;
    let info = unsafe { &*info };
    exec.base_address = info.dlpi_addr as _;
    exec.sections = ProcessModule::get_memory_sections(info);
    exec.module_size = ProcessModule::get_module_size(info);
    /*
    #[cfg(feature = "reloaded")]
    exec.hash = get_executable_hash_ex();
    #[cfg(not(feature = "reloaded"))]
    */
    exec.hash = 0;
    -1
}

#[derive(Debug)]
pub struct ProcessInfo {
    pid: libc::pid_t,
    executable: ProcessModule
}

impl ProcessInfo {
    pub fn get_current_process() -> Result<Self, Box<dyn Error>> {
        let mut out: MaybeUninit<ProcessInfo> = MaybeUninit::zeroed();
        unsafe { out.assume_init_mut().pid =  libc::getpid() };
        unsafe { libc::dl_iterate_phdr(Some(iterate_cb), out.as_mut_ptr() as _) };
        Ok(unsafe { out.assume_init() })
    }
    pub fn get_main_module(&self) -> &ProcessModule { &self.executable }
    pub fn get_executable_address(&self) -> usize { self.get_main_module().get_base_address() }
    pub fn get_executable_size(&self) -> usize { self.get_main_module().get_memory_size() }
    pub fn get_executable_hash(&self) -> u64 { self.get_main_module().hash }

    pub fn change_protection(&mut self, region: &[u8], protection: PageProtection) {
        let prot_raw: libc::c_int = protection.into();
        unsafe { self.change_protection_raw(region.as_ptr(), region.len(), prot_raw as _) };
    }
    pub unsafe fn change_protection_raw(&mut self, address: *const u8, size: usize, protect: u32) {
        if unsafe { libc::mprotect(address as _, size, protect as _) } != 0 {
            println!("change_protection_raw failed: errno {}", unsafe { *libc::__errno_location() });
        }
    }
    pub fn get_process_id(&self) -> u32 { self.pid as _ }
    pub fn get_memory_sections(&self) -> Vec<MemorySection> {
        self.executable.sections.clone()
    }
}

unsafe impl Send for ProcessInfo {}
unsafe impl Sync for ProcessInfo {}

#[derive(Clone)]
pub struct MemorySection {
    address: *const u8,
    size: usize,
    _type: u32
}

impl MemorySection {
    fn from_section_header(
        base_address: usize,
        section: &libc::Elf64_Phdr
    ) -> Self {
        Self {
            address: (base_address + section.p_vaddr as usize) as *const u8,
            size: section.p_memsz as usize,
            _type: section.p_type,
        }
    }

    pub fn get_name(&self) -> &str {
        "None"
    }

    pub fn get_virtual_address(&self) -> *const u8 { self.address }

    pub fn get_size(&self) -> usize { self.size }
}

impl Debug for MemorySection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemorySection {{ name: {}, address: 0x{:x}, size: 0x{:x} }}",
               self.get_name(), self.address as usize, self.size)
    }
}

pub fn get_platform_thread_id() -> u64 {
    (unsafe { libc::gettid() }) as u64
}