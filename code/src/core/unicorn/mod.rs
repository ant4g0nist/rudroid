pub mod arch;
pub mod ffi;
pub mod unicorn_const;

use libc::c_void;
use xmas_elf::header;

use crate::utilities;
use super::rudroid::Emulator;
use arch::arm64;
use unicorn_const::{Arch, uc_error, MemRegion, Protection, HookType, MemType, Query};

#[derive(Debug)]
pub struct Context {
    context: ffi::uc_context,
}

impl Context {
    pub fn new() -> Self {
        Context { context: 0 }
    }
    pub fn is_initialized(&self) -> bool {
        self.context != 0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::uc_free(self.context) };
    }
}

impl<D> Emulator<D> {
    
    /// Return the architecture of the current emulator.
    pub fn get_arch(&self) -> Arch {
        self.arch
    }

    /// Returns a vector with the memory regions that are mapped in the emulator.
    pub fn mem_regions(&self) -> Result<Vec<MemRegion>, uc_error> {
        let mut nb_regions: u32 = 0;
        let mut p_regions: *const MemRegion = std::ptr::null_mut();
        let err = unsafe { ffi::uc_mem_regions(self.uc, &mut p_regions, &mut nb_regions) };
        if err == uc_error::OK {
            let mut regions = Vec::new();
            for i in 0..nb_regions {
                regions.push(unsafe { std::mem::transmute_copy(&*p_regions.offset(i as isize)) });
            }
            unsafe { libc::free(p_regions as _) };
            Ok(regions)
        } else {
            Err(err)
        }
    }

    /// Read a range of bytes from memory at the specified address.
    pub fn mem_read(&self, address: u64, buf: &mut [u8]) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_mem_read(self.uc, address, buf.as_mut_ptr(), buf.len()) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Return a range of bytes from memory at the specified address as vector.
    pub fn mem_read_as_vec(&self, address: u64, size: usize) -> Result<Vec<u8>, uc_error> {
        let mut buf = vec![0; size];
        let err = unsafe { ffi::uc_mem_read(self.uc, address, buf.as_mut_ptr(), size) };
        if err == uc_error::OK {
            Ok(buf)
        } else {
            Err(err)
        }
    }

    pub fn mem_write(&mut self, address: u64, bytes: &[u8]) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_mem_write(self.uc, address, bytes.as_ptr(), bytes.len()) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Map an existing memory region in the emulator at the specified address.
    ///
    /// This function is marked unsafe because it is the responsibility of the caller to
    /// ensure that `size` matches the size of the passed buffer, an invalid `size` value will
    /// likely cause a crash in unicorn.
    ///
    /// `address` must be aligned to 4kb or this will return `Error::ARG`.
    ///
    /// `size` must be a multiple of 4kb or this will return `Error::ARG`.
    ///
    /// `ptr` is a pointer to the provided memory region that will be used by the emulator.
    pub fn mem_map_ptr(&mut self, 
            address: u64, 
            size: usize, 
            perms: Protection, 
            ptr: *mut c_void
    ) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_mem_map_ptr(self.uc, address, size, perms.bits(), ptr) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Map a memory region in the emulator at the specified address.
    ///
    /// `address` must be aligned to 4kb or this will return `Error::ARG`.
    /// `size` must be a multiple of 4kb or this will return `Error::ARG`.
    pub fn mem_map(&mut self, 
            address: u64, 
            size: libc::size_t, 
            perms: Protection
    ) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_mem_map(self.uc, address, size, perms.bits()) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Unmap a memory region.
    ///
    /// `address` must be aligned to 4kb or this will return `Error::ARG`.
    /// `size` must be a multiple of 4kb or this will return `Error::ARG`.
    pub fn mem_unmap(&mut self, 
            address: u64, 
            size: libc::size_t
    ) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_mem_unmap(self.uc, address, size) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Set the memory permissions for an existing memory region.
    ///
    /// `address` must be aligned to 4kb or this will return `Error::ARG`.
    /// `size` must be a multiple of 4kb or this will return `Error::ARG`.
    pub fn mem_protect(&mut self, 
            address: u64, 
            size: libc::size_t, 
            perms: Protection
    ) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_mem_protect(self.uc, address, size, perms.bits()) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Write an unsigned value from a register.
    pub fn reg_write<T: Into<i32>>(&mut self, regid: T, value: u64) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_reg_write(self.uc, regid.into(), &value as *const _ as _) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Write variable sized values into registers.
    /// 
    /// The user has to make sure that the buffer length matches the register size.
    /// This adds support for registers >64 bit (GDTR/IDTR, XMM, YMM, ZMM (x86); Q, V (arm64)).
    pub fn reg_write_long<T: Into<i32>>(&self, regid: T, value: Box<[u8]>) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_reg_write(self.uc, regid.into(), value.as_ptr() as _) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Read an unsigned value from a register.
    /// 
    /// Not to be used with registers larger than 64 bit.
    pub fn reg_read<T: Into<i32>>(&self, regid: T) -> Result<u64, uc_error> {
        let mut value: u64 = 0;
        let err = unsafe { ffi::uc_reg_read(self.uc, regid.into(), &mut value as *mut u64 as _) };
        if err == uc_error::OK {
            Ok(value)
        } else {
            Err(err)
        }
    }

    /// Read 128, 256 or 512 bit register value into heap allocated byte array.
    /// 
    /// This adds safe support for registers >64 bit (GDTR/IDTR, XMM, YMM, ZMM (x86); Q, V (arm64)).
    pub fn reg_read_long<T: Into<i32>>(&self, regid: T) -> Result<Box<[u8]>, uc_error> {
        let err: uc_error;
        let boxed: Box<[u8]>;
        let mut value: Vec<u8>;
        let curr_reg_id = regid.into();
        let curr_arch = self.get_arch();

        // if curr_arch == Arch::X86 {
        //     if curr_reg_id >= x86::RegisterX86::XMM0 as i32 && curr_reg_id <= x86::RegisterX86::XMM31 as i32 {
        //         value = vec![0; 16 as usize];                
        //     } else if curr_reg_id >= x86::RegisterX86::YMM0 as i32 && curr_reg_id <= x86::RegisterX86::YMM31 as i32 {
        //         value = vec![0; 32 as usize];
        //     } else if curr_reg_id >= x86::RegisterX86::ZMM0 as i32 && curr_reg_id <= x86::RegisterX86::ZMM31 as i32 {
        //         value = vec![0; 64 as usize];
        //     } else if curr_reg_id == x86::RegisterX86::GDTR as i32 ||
        //               curr_reg_id == x86::RegisterX86::IDTR as i32 {
        //         value = vec![0; 10 as usize]; // 64 bit base address in IA-32e mode
        //     } else {
        //         return Err(uc_error::ARG)
        //     }
        // } else 
        if curr_arch == Arch::ARM64 {
            if (curr_reg_id >= arm64::RegisterARM64::Q0 as i32 && curr_reg_id <= arm64::RegisterARM64::Q31 as i32) ||
               (curr_reg_id >= arm64::RegisterARM64::V0 as i32 && curr_reg_id <= arm64::RegisterARM64::V31 as i32) {
                value = vec![0; 16 as usize];
            } else {
                return Err(uc_error::ARG)
            }
        } else {
            return Err(uc_error::ARCH)
        }
        
        err = unsafe { ffi::uc_reg_read(self.uc, curr_reg_id, value.as_mut_ptr() as _) };

        if err == uc_error::OK {
            boxed = value.into_boxed_slice();
            Ok(boxed)
        } else {
            Err(err)
        }
    }

    /// Read a signed 32-bit value from a register.
    pub fn reg_read_i32<T: Into<i32>>(&self, regid: T) -> Result<i32, uc_error> {
        let mut value: i32 = 0;
        let err = unsafe { ffi::uc_reg_read(self.uc, regid.into(), &mut value as *mut i32 as _) };
        if err == uc_error::OK {
            Ok(value)
        } else {
            Err(err)
        }
    }

    /// Add a code hook.
    pub fn add_code_hook<F: 'static,>(
        &mut self,
        begin: u64,
        end: u64,
        callback: F,
    ) -> Result<ffi::uc_hook, uc_error>
    where F: FnMut(&mut Emulator<D>, u64, u32)
    {
        let mut hook_ptr = std::ptr::null_mut();
        let mut user_data = Box::new(ffi::CodeHook {
            unicorn: self,
            callback: Box::new(callback),
        });
        
        let err = unsafe {
            ffi::uc_hook_add(
                self.uc,
                &mut hook_ptr,
                HookType::CODE,
                ffi::code_hook_proxy::<D> as _,
                user_data.as_mut() as *mut _ as _,
                begin,
                end,
            )
        };
        if err == uc_error::OK {
            unsafe { self }.code_hooks.insert(hook_ptr, user_data);
            Ok(hook_ptr)
        } else {
            Err(err)
        }
    }

    /// Add a memory hook.
    pub fn add_mem_hook<F: 'static>(
        &mut self,
        hook_type: HookType,
        begin: u64,
        end: u64,
        callback: F,
    ) -> Result<ffi::uc_hook, uc_error>
    where F: FnMut(&mut Emulator<D>, MemType, u64, usize, i64)
    {
        if (hook_type as i32) < 16 || hook_type == HookType::INSN_INVALID {
            return Err(uc_error::ARG);
        }
        
        let mut hook_ptr = std::ptr::null_mut();
        let mut user_data = Box::new(ffi::MemHook {
            unicorn: self,
            callback: Box::new(callback),
        });
        
        let err = unsafe {
            ffi::uc_hook_add(
                self.uc,
                &mut hook_ptr,
                hook_type,
                ffi::mem_hook_proxy::<D> as _,
                user_data.as_mut() as *mut _ as _,
                begin,
                end,
            )
        };
        if err == uc_error::OK {
            unsafe { self}.mem_hooks.insert(hook_ptr, user_data);
            Ok(hook_ptr)
        } else {
            Err(err)
        }
    }

    /// Add an interrupt hook.
    pub fn add_intr_hook<F: 'static>(
        &mut self,
        callback: F,
    ) -> Result<ffi::uc_hook, uc_error>
    where F: FnMut(&mut Emulator<D>, u32)
    { 
        let mut hook_ptr = std::ptr::null_mut();
        let mut user_data = Box::new(ffi::InterruptHook {
            unicorn: self,
            callback: Box::new(callback),
        });
        
        let err = unsafe {
            ffi::uc_hook_add(
                self.uc,
                &mut hook_ptr,
                HookType::INTR,
                ffi::intr_hook_proxy::<D> as _,
                user_data.as_mut() as *mut _ as _,
                0,
                0,
            )
        };
        if err == uc_error::OK {
            unsafe { self }.intr_hooks.insert(hook_ptr, user_data);
            Ok(hook_ptr)
        } else {
            Err(err)
        }
    }

    /// Remove a hook.
    ///
    /// `hook` is the value returned by `add_*_hook` functions.
    pub fn remove_hook(&mut self, hook: ffi::uc_hook) -> Result<(), uc_error> {
        let handle = unsafe { self };
        let err: uc_error;
        // if handle.code_hooks.contains_key(&hook) || 
        //     handle.mem_hooks.contains_key(&hook) ||
        //     handle.intr_hooks.contains_key(&hook) ||
        //     handle.insn_in_hooks.contains_key(&hook) ||
        //     handle.insn_out_hooks.contains_key(&hook) ||
        //     handle.insn_sys_hooks.contains_key(&hook) {
            err = unsafe { ffi::uc_hook_del(handle.uc, hook) };
            // handle.mem_hooks.remove(&hook);
        // } else {
        //     err = uc_error::HOOK;
        // }

        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Allocate and return an empty Unicorn context.
    /// 
    /// To be populated via context_save.
    pub fn context_alloc(&self) -> Result<Context, uc_error> {
        let mut empty_context: ffi::uc_context = Default::default();
        let err = unsafe { ffi::uc_context_alloc(self.uc, &mut empty_context) };
        if err == uc_error::OK {
            Ok(Context { context: empty_context })
        } else {
            Err(err)
        }
    }

    /// Save current Unicorn context to previously allocated Context struct.
    pub fn context_save(&self, context: &mut Context) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_context_save(self.uc, context.context) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Allocate and return a Context struct initialized with the current CPU context.
    /// 
    /// This can be used for fast rollbacks with context_restore.
    /// In case of many non-concurrent context saves, use context_alloc and *_save 
    /// individually to avoid unnecessary allocations.
    pub fn context_init(&self) -> Result<Context, uc_error> {
        let mut new_context: ffi::uc_context = Default::default();
        let err = unsafe { ffi::uc_context_alloc(self.uc, &mut new_context) };
        if err != uc_error::OK {
            return Err(err);
        }
        let err = unsafe { ffi::uc_context_save(self.uc, new_context) };
        if err == uc_error::OK {
            Ok(Context { context: new_context })
        } else {
            unsafe { ffi::uc_free(new_context) };
            Err(err)
        }
    }

    /// Restore a previously saved Unicorn context.
    /// 
    /// Perform a quick rollback of the CPU context, including registers and some
    /// internal metadata. Contexts may not be shared across engine instances with
    /// differing arches or modes. Memory has to be restored manually, if needed.
    pub fn context_restore(&self, context: &Context) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_context_restore(self.uc, context.context) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Emulate machine code for a specified duration.
    ///
    /// `begin` is the address where to start the emulation. The emulation stops if `until`
    /// is hit. `timeout` specifies a duration in microseconds after which the emulation is
    /// stopped (infinite execution if set to 0). `count` is the maximum number of instructions
    /// to emulate (emulate all the available instructions if set to 0).
    pub fn emu_start(&mut self, 
            begin: u64, 
            until: u64, 
            timeout: u64, 
            count: usize
    ) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_emu_start(self.uc, begin, until, timeout, count as _) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Stop the emulation.
    ///
    /// This is usually called from callback function in hooks.
    /// NOTE: For now, this will stop the execution only after the current block.
    pub fn emu_stop(&mut self) -> Result<(), uc_error> {
        let err = unsafe { ffi::uc_emu_stop(self.uc) };
        if err == uc_error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Query the internal status of the engine.
    /// 
    /// supported: MODE, PAGE_SIZE, ARCH
    pub fn query(&self, query: Query) -> Result<usize, uc_error> {
        let mut result: libc::size_t = Default::default();
        let err = unsafe { ffi::uc_query(self.uc, query, &mut result) };
        if err == uc_error::OK {
            Ok(result)
        } else {
            Err(err)
        }
    }

    pub fn debug_print(&self, message: String) {
        if self.debug {
            utilities::draw_line(); 
            utilities::log(&message, utilities::DebugLevel::DEBUG);
        }
    }

    pub fn enable_vfp(&mut self) {
        let UC_ARM64_REG_CPACR_EL1 = 261;
        let cpacr_el1 = self.reg_read(UC_ARM64_REG_CPACR_EL1).unwrap();
        self.reg_write(UC_ARM64_REG_CPACR_EL1, cpacr_el1|0x300000 as u64).unwrap();
    }

    pub fn align_len(&self, len: u64) -> u64 {
        ((len+ 0x1000 - 1) / 0x1000) * 0x1000
    }

    pub fn uc_align_down(&self, address: u64) -> u64 {
        (address / 0x1000) * 0x1000
    }

    pub fn uc_align_up(&self, address: u64) -> u64 {
        let re = (address / 0x1000 + 1) * 0x1000;
        re
    }

    pub fn align_size(&self, value: u64, align: u64) -> u64 {
        ( (value -1) / align + 1 ) * align
    }

    pub fn null_mut(&self) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }


    pub fn null_str(&self, input: &str) -> String {
        let res = input.trim_matches(char::from(0));
        String::from(res)
    }

    pub fn alignment(&self, address: u64) -> u64 {
        match self.machine {
            header::Machine::AArch64 => {
                (address / 8) * 8
            },
            _ => {
                panic!("");
            }
        }
    }
}

