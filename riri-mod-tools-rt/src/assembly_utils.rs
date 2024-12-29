#[cfg(any(target_arch = "x86_64"))]
pub mod x86_64 {
    use std::arch::x86_64;
    type XmmType = x86_64::__m128;
    // type YmmType = x86_64::__m256;
    // type ZmmType = x86_64::__m512; 
    const XMM_COUNT: usize = XmmRegister::Xmm15 as usize + 1;

    unsafe fn write_stack_pointer(ofs: isize) -> String {
        if ofs == 0 {
            "[rsp]".to_owned()
        } else if ofs > 0 {
            format!("[rsp + {}]", ofs)
        } else {
            format!("[rsp - {}]", ofs)
        }
    }

    #[repr(u32)]
    #[derive(Debug)]
    pub enum XmmRegister {
        Xmm0,
        Xmm1,
        Xmm2,
        Xmm3,
        Xmm4,
        Xmm5,
        Xmm6,
        Xmm7,
        Xmm8,
        Xmm9,
        Xmm10,
        Xmm11,
        Xmm12,
        Xmm13,
        Xmm14,
        Xmm15
    }
    
    impl TryFrom<u32> for XmmRegister {
        type Error = ();
        fn try_from(value: u32) -> Result<Self, Self::Error> {
            if value <= XmmRegister::Xmm15 as u32 {
                Ok(unsafe { std::mem::transmute(value) })
            } else {
                Err(())
            }
        }
    } 
    
    /// Pushes the value of an xmm register to the stack, saving it so it can be restored with
    /// `pop_xmm_for_fasm`.
    #[no_mangle]
    pub unsafe extern "C" fn push_xmm_for_fasm(n: XmmRegister) -> *const u8 {
        format!("sub rsp, {}\n\
                movdqu dqword [rsp], xmm{}\n", 
                std::mem::size_of::<XmmType>(), n as u32
        ).leak().as_ptr()
    }
    /// Pushes all xmm registers to the stack, saving them to be restored with
    /// `pop_all_xmm_for_fasm`
    #[no_mangle]
    pub unsafe extern "C" fn push_all_xmm_for_fasm() -> *const u8 {
        let mut out = String::new();
        out.push_str(&format!("sub rsp, {}\n", std::mem::size_of::<XmmType>() * XMM_COUNT));
        for i in 0..XMM_COUNT {
            out.push_str(&format!("movdqu dqword {}, xmm{}\n", 
                write_stack_pointer((XMM_COUNT-1-i) as isize * std::mem::size_of::<XmmType>() as isize), i));
        }
        out.leak().as_ptr()
    }
    /// Pops the value of an xmm register to the stack, restoring it after being saved with
    /// `push_xmm_for_fasm`
    #[no_mangle]
    pub unsafe extern "C" fn pop_xmm_for_fasm(n: XmmRegister) -> *const u8 {
        format!("movdqu xmm{}, dqword [rsp]\n\
                add rsp, {}\n",
                n as u32, std::mem::size_of::<XmmType>()
        ).leak().as_ptr()
    }
    /// Pops all xmm registers from the stack, restoring them after being saved with
    /// `push_all_xmm_for_fasm`
    #[no_mangle]
    pub unsafe extern "C" fn pop_all_xmm_for_fasm() -> *const u8 {
        let mut out = String::new();
        out.push_str(&format!("add rsp, {}\n", std::mem::size_of::<XmmType>() * XMM_COUNT));
        for i in 0..XMM_COUNT {
            out.push_str(&format!("movdqu dqword {}, xmm{}\n", 
                write_stack_pointer(-((XMM_COUNT-i) as isize) * std::mem::size_of::<XmmType>() as isize), i));
        }
        out.leak().as_ptr()
    }
}