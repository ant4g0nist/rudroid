
use xmas_elf::header;
use xmas_elf::ElfFile;
use std::collections::HashMap;

use super::mmu;
use super::android::fs;
use super::unicorn::ffi;

use super::unicorn::unicorn_const::{Arch, Mode, uc_error};

// #[derive(Debug)]
pub struct Emulator<D>  {
    pub debug               : bool,

    pub rootfs              : String,
    pub elf_path            : String,

    pub machine             : header::Machine,
    pub endian              : header::Data,
    pub arch                : Arch,

    pub uc                  : ffi::uc_handle,
    pub uc_type             : D,

    pub filesystem          : fs::FsScheme,

    // mmu stuff
    pub load_address        : u64,
    pub mmap_address        : u64,
    pub new_stack           : u64,
    pub interp_address      : u64,
    pub entry_point         : u64,
    pub elf_entry           : u64,
    pub brk_address         : u64,

    //elf arguments
    pub args                : Vec<String>,
    pub env                 : Vec<String>,

    pub map_infos           : HashMap<u64, mmu::MapInfo>,

    //hook
    pub code_hooks          : HashMap<*mut libc::c_void, Box<ffi::CodeHook<D>>>,
    pub mem_hooks           : HashMap<*mut libc::c_void, Box<ffi::MemHook<D>>>,
    pub intr_hooks          : HashMap<*mut libc::c_void, Box<ffi::InterruptHook<D>>>,
    pub insn_in_hooks       : HashMap<*mut libc::c_void, Box<ffi::InstructionInHook<D>>>,
    pub insn_out_hooks      : HashMap<*mut libc::c_void, Box<ffi::InstructionOutHook<D>>>,
    pub insn_sys_hooks      : HashMap<*mut libc::c_void, Box<ffi::InstructionSysHook<D>>>,

    // syscalls stuff
    pub sigmap              : HashMap<u64, Vec<u8>>,

    _pin                    : std::marker::PhantomPinned,
}


impl<D> Emulator<D>
{
    pub fn new(elf_path: &str, rootfs: &str, elf: &mut ElfFile, endian: header::Data, args: Vec<String>, env: Vec<String>, data: D, debug: bool) -> Result<Emulator<D>, uc_error> {
        let mut machine = elf.header.pt2.machine().as_machine();
        let (arch, mode) = match machine {
            header::Machine::AArch64 => {
                (Arch::ARM64, Mode::LITTLE_ENDIAN)
            },
            _ => {
                panic!("Not implemented yet!")
            }
        };

        let mut handle = std::ptr::null_mut();
        let err = unsafe { ffi::uc_open(arch, mode, &mut handle) };
        let mut emu = Emulator {
            debug           : debug,
            rootfs          : String::from(rootfs),

            elf_path        : String::from(elf_path),
            args            : args,
            env             : env,
            
            uc              : handle,
            uc_type         : data,
            
            arch            : arch,
            machine         : machine,
            endian          : endian,

            map_infos       : HashMap::new(),
            entry_point     : 0,
            elf_entry       : 0,
            brk_address     : 0,
            mmap_address    : 0,
            interp_address  : 0,
            new_stack       : 0,
            load_address    : 0,

            //hooks
            code_hooks      : HashMap::new(),
            mem_hooks       : HashMap::new(),
            intr_hooks      : HashMap::new(),
            insn_in_hooks   : HashMap::new(),
            insn_out_hooks  : HashMap::new(),
            insn_sys_hooks  : HashMap::new(),

            _pin            : std::marker::PhantomPinned,

            filesystem      : fs::FsScheme::new(String::from(rootfs)),
            sigmap          : HashMap::new(),
        };
        
        emu.load(elf);
        emu.display_mapped();

        if err == uc_error::OK {
            Ok(emu)
        } else {
            Err(err)
        }
    }
}