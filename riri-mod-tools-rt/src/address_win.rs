#![allow(dead_code, unused_variables, unused_imports)]
use std::{
    ffi::CStr,
    fmt::Debug,
    mem::MaybeUninit
};
use twox_hash::XxHash3_64;
use windows::Win32::{
    Foundation,
    System::{
        Diagnostics,
        Memory, 
        ProcessStatus,
        SystemServices,
        Threading 
    }
};

#[cfg(feature = "reloaded")]
#[link(name = "riri_mod_runtime_reloaded", kind = "raw-dylib")]
unsafe extern "C" {
    pub(crate) unsafe fn get_executable_hash_ex() -> u64;
}

// Based on .NET's System.Diagnostics.Process:
// https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process

type Handle = Foundation::HANDLE;
type Module = Foundation::HMODULE;
type Window = Foundation::HWND;

#[derive(Debug, Clone)]
pub struct ProcessModule {
    owner: Handle,
    handle: Module,
    module_size: usize,
    hash: u64,
}
use std::sync::OnceLock;

impl ProcessModule {
    pub unsafe fn new(own: Handle, hndl: Module) -> windows::core::Result<Self> {
        let mut pinfo: std::mem::MaybeUninit<ProcessStatus::MODULEINFO> = std::mem::MaybeUninit::uninit();
        ProcessStatus::GetModuleInformation(
            own, hndl, pinfo.as_mut_ptr(), std::mem::size_of::<ProcessStatus::MODULEINFO>() as u32
        )?;
        Ok(ProcessModule {
            owner: own,
            handle: hndl,
            module_size: pinfo.assume_init_ref().SizeOfImage as usize,
            #[cfg(feature = "reloaded")]
            hash: get_executable_hash_ex(),
            #[cfg(not(feature = "reloaded"))]
            hash: u64::MAX,
        })
    }
    pub fn get_base_address(&self) -> usize { self.handle.0 as usize }
    pub fn get_memory_size(&self) -> usize { self.module_size }
    pub fn as_raw(&self) -> Module { self.handle }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    process: Handle,
    process_id: u32,
    executable: ProcessModule
}

unsafe impl Send for ProcessInfo { }
unsafe impl Sync for ProcessInfo { }

impl ProcessInfo {
    pub fn get_current_process() -> windows::core::Result<Self> {
        let process = unsafe { Threading::GetCurrentProcess() };
        let process_id = unsafe { Threading::GetCurrentProcessId() };
        let executable = Self::try_get_main_module(process)?;
        Ok(ProcessInfo { process, process_id, executable })
    }
    fn try_get_main_module(handle: Handle) -> windows::core::Result<ProcessModule> {
        unsafe {
            let mut module_list: Vec<Module> = Vec::with_capacity(64);
            let mod_sz = (module_list.capacity() * std::mem::size_of::<Module>()) as u32;
            let mod_ptr = module_list.as_mut_ptr();
            let mut mod_sz_total: u32 = 0;
            ProcessStatus::EnumProcessModules(handle, mod_ptr, mod_sz, (&mut mod_sz_total) as *mut u32)?;
            let new_len = (if mod_sz_total > mod_sz {
                mod_sz
            } else {
                mod_sz_total
            } as usize) / std::mem::size_of::<Module>();
            module_list.set_len(new_len);
            Ok(ProcessModule::new(handle, module_list[0])?)
        }
    }
    pub fn get_main_module(&self) -> &ProcessModule { &self.executable }
    pub fn get_executable_address(&self) -> usize { self.get_main_module().get_base_address() }
    pub fn get_executable_size(&self) -> usize { self.get_main_module().get_memory_size() }
    pub fn get_executable_hash(&self) -> u64 { self.get_main_module().hash }
    // pub fn get_main_window_handle(&self) {}
    // pub fn get_main_window_title(&self) {}
    pub unsafe fn change_protection_raw(&mut self, address: *const u8, size: usize, protect: u32) {
        let flags = Memory::PAGE_PROTECTION_FLAGS(protect);
        let mut old_flags: MaybeUninit<Memory::PAGE_PROTECTION_FLAGS> = MaybeUninit::uninit();
        if let Err(e) = Memory::VirtualProtectEx(
            self.process,
            address as *const std::ffi::c_void,
            size,
            flags,
            old_flags.as_mut_ptr()
        ) {
            println!("change_protection_raw failed: {}", e);
        }
    }
    pub fn get_process_id(&self) -> u32 {
        self.process_id
    }
    pub fn get_process_memory(&self) -> Vec<u8> {
        let mut process: Vec<u8> = Vec::with_capacity(self.get_executable_size());
        unsafe { process.set_len(self.get_executable_size()) }
        
        if let Err(e) = unsafe { Diagnostics::Debug::ReadProcessMemory(
            self.get_main_module().owner,
            self.get_executable_address() as *const std::ffi::c_void,
            process.as_mut_ptr() as *mut std::ffi::c_void,
            self.get_executable_size(),
            None
        ) } {
            println!("get_process_memory failed: {}", e);
        }
        process
    }

    const fn get_maximum_memory_section_size() -> usize {
        size_of::<SystemServices::IMAGE_DOS_HEADER>() +
        size_of::<Diagnostics::Debug::IMAGE_NT_HEADERS64>() +
        (size_of::<Diagnostics::Debug::IMAGE_SECTION_HEADER>() * u8::MAX as usize)
    }

    pub fn get_memory_sections(&self) -> Vec<MemorySection> {
        let mut buf: MaybeUninit<[u8; const { Self::get_maximum_memory_section_size() }]> = MaybeUninit::uninit();
        // Read beginning of executable, including DOS_HEADER, NT_HEADER and section headers
        unsafe { Diagnostics::Debug::ReadProcessMemory(
            self.get_main_module().owner,
            self.get_executable_address() as *const std::ffi::c_void,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            Self::get_maximum_memory_section_size(),
            None
        ) }.unwrap();
        let dos_header = unsafe { &*(buf.as_ptr() as *const SystemServices::IMAGE_DOS_HEADER) };
        let nt_header_start = dos_header.e_lfanew as usize;
        let nt_header = unsafe { &*((buf.as_ptr() as *const u8).add(nt_header_start) as *const Diagnostics::Debug::IMAGE_NT_HEADERS64) };
        let sections_start = nt_header_start + size_of::<Diagnostics::Debug::IMAGE_NT_HEADERS64>();
        let section_ptr = unsafe { (buf.as_ptr() as *const u8).add(sections_start) as *const Diagnostics::Debug::IMAGE_SECTION_HEADER };
        let mut result = Vec::with_capacity(nt_header.FileHeader.NumberOfSections as usize);
        for i in 0..nt_header.FileHeader.NumberOfSections as usize {
            result.push(MemorySection::from_section_header(
                self.get_executable_address(),
                unsafe { &*(section_ptr.add(i)) }
            ));
        }
        result
    }

    pub fn in_executable<T>(&self, ptr: &T) -> bool {
        &raw const *ptr as usize >= self.get_executable_address() &&
        (&raw const *ptr as usize) - self.get_executable_address() < self.get_executable_size()
    }

    pub fn get_path(&self) -> String {
        let mut buf: MaybeUninit<[u8; 260]> = MaybeUninit::uninit();
        let path_len = unsafe { ProcessStatus::GetModuleFileNameExA(
            Some(self.process), 
            Some(self.executable.handle), 
            &mut buf.assume_init_mut()[..]) 
        };
        let path = unsafe { std::str::from_utf8_unchecked(&buf.assume_init_ref()[..path_len as usize]) };
        path.to_owned()
    }

    pub fn get_executable_name(&self) -> String {
        let full_path = std::path::PathBuf::from(self.get_path());
        full_path.file_name().unwrap().to_str().unwrap().to_owned()
    }
    pub fn as_raw(&self) -> Handle { self.process }
}

// #[derive(Debug)]
pub struct MemorySection {
    address: *const u8,
    size: usize,
    name: [u8; 8],
}

impl MemorySection {
    fn from_section_header(
        base_address: usize,
        section: &Diagnostics::Debug::IMAGE_SECTION_HEADER
    ) -> Self {
        Self {
            name: section.Name,
            address: (base_address + section.VirtualAddress as usize) as *const u8,
            size: section.SizeOfRawData as usize
        }
    }

    pub fn get_name(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.name.as_slice()) }
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

pub fn get_thread_id() -> u64 {
    (unsafe { Threading::GetCurrentThreadId() }) as u64
}
