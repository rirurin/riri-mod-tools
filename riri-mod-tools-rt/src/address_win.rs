#![allow(dead_code, unused_variables)]
use std::{
    fs::File,
    mem::MaybeUninit
};
use twox_hash::XxHash3_64;
use windows::Win32::{
    Foundation,
    System::{ ProcessStatus, Threading }
};

// Based on .NET's System.Diagnostics.Process:
// https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process

type Handle = Foundation::HANDLE;
type Module = Foundation::HMODULE;

#[derive(Debug, Clone)]
pub struct ProcessModule {
    owner: Handle,
    handle: Module,
    module_size: usize,
    hash: u64
}

impl ProcessModule {
    pub unsafe fn new(own: Handle, hndl: Module) -> windows::core::Result<Self> {
        let mut pinfo: std::mem::MaybeUninit<ProcessStatus::MODULEINFO> = std::mem::MaybeUninit::uninit();
        ProcessStatus::GetModuleInformation(
            own, hndl, pinfo.as_mut_ptr(), std::mem::size_of::<ProcessStatus::MODULEINFO>() as u32
        )?;
        let mut filename_buffer: MaybeUninit<[u8; 260]> = MaybeUninit::uninit();
        let path_len = ProcessStatus::GetModuleFileNameExA(own, hndl, filename_buffer.assume_init_mut()); 
        let exec = std::fs::read(std::str::from_utf8_unchecked(&filename_buffer.assume_init_ref()[..path_len as usize]))?;
        let hash = XxHash3_64::oneshot(exec.as_slice());
        println!("hash: 0x{:x}", hash);
        Ok(ProcessModule {
            owner: own,
            handle: hndl,
            module_size: pinfo.assume_init_ref().SizeOfImage as usize,
            hash
        })
    }
    pub fn get_base_address(&self) -> usize { self.handle.0 as usize }
    pub fn get_memory_size(&self) -> usize { self.module_size }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    process: Handle,
    executable: ProcessModule
}

unsafe impl Send for ProcessInfo { }
unsafe impl Sync for ProcessInfo { }

impl ProcessInfo {
    pub fn get_current_process() -> windows::core::Result<Self> {
        let process = unsafe { Threading::GetCurrentProcess() };
        let executable = Self::try_get_main_module(process)?;
        Ok(ProcessInfo { process, executable })
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
}

pub fn get_thread_id() -> u64 {
    (unsafe { Threading::GetCurrentThreadId() }) as u64
}
