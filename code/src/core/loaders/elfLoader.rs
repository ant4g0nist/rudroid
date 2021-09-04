use crate::core as linux;
use crate::utilities;
use xmas_elf::{header, ElfFile, program};
use super::super::rudroid::Emulator;
use super::super::unicorn::unicorn_const::Protection;
use super::super::unicorn::arch::arm64::RegisterARM64;


/* Symbolic values for the entries in the auxiliary table
   put on the initial stack */
#[repr(u64)]
enum AUX {
    AT_NULL = 0,
    AT_IGNORE = 1,
    AT_EXECFD = 2,
    AT_PHDR = 3,
    AT_PHENT = 4,
    AT_PHNUM = 5,
    AT_PAGESZ = 6,
    AT_BASE = 7,
    AT_FLAGS = 8,
    AT_ENTRY = 9,
    AT_NOTELF = 10,
    AT_UID = 11,
    AT_EUID = 12,
    AT_GID = 13,
    AT_EGID = 14,
    AT_PLATFORM = 15,
    AT_HWCAP = 16,
    AT_CLKTCK = 17,
    AT_SECURE = 23,
    AT_BASE_PLATFORM = 24,
    AT_RANDOM = 25,
    AT_HWCAP2 = 26,
    AT_EXECFN = 31,
}

use std::io::prelude::*;

impl<D> Emulator<D> {

    pub fn load(& mut self, elf: &mut ElfFile) {
        self.enable_vfp();
        
        let profile = match self.machine {
            header::Machine::AArch64 => {
                (linux::OS64::stack_address, linux::OS64::stack_size)
            },
            _ => {
                    panic!("[load_with_ld] Not implemented yet!")
            }
        };

        let mut stack_address = profile.0 as u64;
        let stack_size      = profile.1 as usize;
        
        self.mmu_map(stack_address, stack_size, Protection::READ|Protection::WRITE, "[stack]", self.null_mut());
        self.load_with_ld(stack_address.checked_add(stack_size as u64).unwrap() , 0, self.machine, elf);
        stack_address = self.new_stack;
        self.reg_write(RegisterARM64::SP as i32, stack_address).unwrap();
    }

    fn load_with_ld(&mut self, stack_address: u64, load_address: u64, archbit: header::Machine, elf: &mut ElfFile) {
        let mut load_address = match load_address {
            0 => {
                match  archbit {
                    header::Machine::AArch64 => {
                        self.mmap_address = linux::OS64::mmap_address as u64;
                        linux::OS64::load_address as u64
                    },
                    _ => {
                        panic!("Shouldn't be here");
                    }
                }
            },
            _ => {
                panic!("Shouldn't be here");
            }
        };
        
        let mut mem_start   : u64 = 0xffff_ffff;
        let mut mem_end     : u64 = 0xffff_ffff;
        let mut mem_s       : u64 = 0;
        let mut mem_e       : u64 = 0;

        let mut interp_path : String = String::new();

        match elf.header.pt2.type_().as_type() {
            header::Type::Executable => {
                load_address = 0;
            },
            header::Type::SharedObject => {
                
            }
            _ => {
                panic!("Some error in head e_type: {:?}", header::Type::SharedObject);
            }
        }

        for header in elf.program_iter() {
            match header.get_type().unwrap() {

                program::Type::Interp => {
                    let offset      = header.offset() as usize;
                    let end_offset  = (header.offset()+header.mem_size()) as usize;
                    let data = elf.input.get(offset..end_offset).unwrap();
                    interp_path = self.null_str(std::str::from_utf8(data).unwrap());
                },

                program::Type::Load => {
                    if mem_start > header.virtual_addr() || mem_start == 0xffff_ffff {
                        mem_start = header.virtual_addr();
                    };

                    if mem_end < header.virtual_addr()+header.mem_size() || mem_end == 0xffff_ffff {
                        mem_end = header.virtual_addr()+header.mem_size();
                    }
                },
                _ => {

                }
            }
        }

        mem_start = self.uc_align_down(mem_start);
        mem_end   = self.uc_align_up(mem_end);

        for header in elf.program_iter() {
            match header.get_type().unwrap() {
                program::Type::Load => {
                    mem_s = self.uc_align_down(load_address + header.virtual_addr());
                    mem_e = self.uc_align_up(load_address + header.virtual_addr() + header.file_size());
                    let perms =  utilities::to_uc_permissions(header.flags());

                    let desc = self.elf_path.clone();
                    self.mmu_map(mem_s, (mem_e-mem_s) as usize, perms, &desc, self.null_mut());
                    
                    let data = elf.input.get(header.offset() as usize..
                                                                (header.offset()+header.file_size()) as usize).unwrap();

                    self.write(load_address+header.virtual_addr(), data);
                },
                _ => {

                }
            }
        }
        
        let loaded_mem_end = load_address + mem_end;

        if loaded_mem_end > mem_e {
            let desc = self.elf_path.clone();
            self.mmu_map( mem_e, (loaded_mem_end-mem_e) as usize, Protection::ALL, &desc, self.null_mut());
        }

        self.elf_entry = elf.header.pt2.entry_point() + load_address;
        self.debug_print(format!("elf_entry {:x}", self.elf_entry));

        self.brk_address = mem_end + load_address + 0x2000; //not sure why?? seems to be used in ql_syscall_brk

        // load interpreter if there is an interpreter
        if !interp_path.is_empty() {
            self.debug_print(format!("Trying to load interpreter: {}{}", self.rootfs, interp_path));

            let mut interp_full_path = String::new();

            interp_full_path.push_str(&self.rootfs);
            interp_full_path.push_str(&interp_path);

            let interp_data = std::fs::read(&interp_full_path).unwrap();
            let interp_elf  = ElfFile::new(interp_data.get(0..).unwrap()).unwrap();

            let mut interp_mem_size: u64 = 0;
            let mut interp_address : u64 = 0;

            for i_header in interp_elf.program_iter() {
                match i_header.get_type().unwrap() {
                    program::Type::Load => {
                        if interp_mem_size < i_header.virtual_addr() + i_header.mem_size() || interp_mem_size == 0 {
                            interp_mem_size = i_header.virtual_addr() + i_header.mem_size();
                        }
                    },
                    _ => {

                    }
                };
            }

            interp_mem_size = self.uc_align_up(interp_mem_size);

            match archbit {
                header::Machine::AArch64 => {
                    interp_address = linux::OS64::interp_address as u64;
                }
                _ => {
                    panic!("what?");
                }
            };

            self.mmu_map(interp_address, interp_mem_size as usize , Protection::ALL, &interp_path, self.null_mut());

            for i_header in interp_elf.program_iter() { 
                match i_header.get_type().unwrap() {
                    program::Type::Load => {
                        let data = interp_elf.input.get(i_header.offset()  as usize..
                                                                            (i_header.offset()+i_header.file_size()) as usize
                                                                                    ).unwrap();
                        self.write( interp_address+i_header.physical_addr(), data);
                    },
                    _ => {

                    }
                };
            }

            self.interp_address = interp_address;
            self.entry_point    = interp_elf.header.pt2.entry_point() + self.interp_address;
        }

        // setup elf table
        let mut elf_table: Vec<u8> = Vec::new();

        let mut new_stack = stack_address;

        // copy arg0 on to stack. elf_path
        new_stack = self.copy_str(new_stack, &mut self.elf_path.clone());

        elf_table.extend_from_slice(&self.pack(self.args.len() as u64 + 1)); // + 1 is for arg0 = elf path.
        elf_table.extend_from_slice(&self.pack(new_stack));
        
        let mut argc = self.args.len();

        loop {
            if argc <=0 {
                break;
            }
            argc -= 1;

            let mut arg = self.args[argc].clone();
            new_stack = self.copy_str(new_stack, &mut arg);
            elf_table.extend_from_slice(&self.pack(new_stack));
        }

        elf_table.extend_from_slice(&self.pack(0));
        
        let mut envc = self.env.len();
        loop {
            if envc <=0 {
                break;
            }
            envc -= 1;
            let mut env = self.env[envc].clone();
            new_stack = self.copy_str(new_stack, &mut env);
            elf_table.extend_from_slice(&self.pack(new_stack));
        }

        elf_table.extend_from_slice(&self.pack(0));

        new_stack = self.alignment(new_stack);

        let mut randstr   = "a".repeat(0x10);
        let mut cpustr    = String::from("aarch64");

        let mut addr1 = self.copy_str(new_stack, &mut randstr);
        new_stack = addr1;

        let mut addr2 = self.copy_str(new_stack, &mut cpustr);
        new_stack = addr2;

        new_stack = self.alignment(new_stack);

        // Set AUX
        let head = elf.header;
        
        let elf_phdr    = load_address + head.pt2.ph_offset();
        let elf_phent   = head.pt2.ph_entry_size();
        let elf_phnum   = head.pt2.ph_count();
        let elf_pagesz  = 0x1000;
        let elf_guid    = linux::uid;
        let elf_flags   = 0;
        let elf_entry   = load_address + head.pt2.entry_point();
        let randstraddr = addr1; 
        let cpustraddr  = addr2;

        let elf_hwcap: u64 = match head.pt2.machine().as_machine() {
            header::Machine::AArch64 => {
                0x078bfbfd
            },
            _ => {
                panic!("");
            }
        };
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_PHDR  as u64, elf_phdr + mem_start));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_PHENT as u64, elf_phent as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_PHNUM as u64, elf_phnum as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_PAGESZ as u64, elf_pagesz as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_BASE as u64, self.interp_address));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_FLAGS as u64, elf_flags));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_ENTRY as u64, elf_entry));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_UID as u64, elf_guid as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_EUID as u64, elf_guid as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_GID as u64, elf_guid as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_EGID as u64, elf_guid as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_HWCAP as u64, elf_hwcap as u64));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_CLKTCK as u64, 100));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_RANDOM as u64, randstraddr));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_PLATFORM as u64, cpustraddr));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_SECURE as u64, 0));
        elf_table.extend_from_slice(&self.new_aux_ent(AUX::AT_NULL as u64, 0));


        let len = 0x10 - ((new_stack - elf_table.len() as u64) & 0xf) as usize;
        let padding = std::iter::repeat('0').take(len).collect::<String>();

        elf_table.extend_from_slice(padding.as_bytes());
        
        let addr = new_stack - elf_table.len() as u64;
        self.write( addr, &elf_table);

        new_stack = new_stack - elf_table.len() as u64;

        self.new_stack = new_stack;
        self.load_address = load_address;
    }

    fn new_aux_ent(&self, key: u64, val: u64) -> Vec<u8> {
        let mut aux: Vec<u8> = Vec::new();
        aux.extend_from_slice(&self.pack(key));
        aux.extend_from_slice(&self.pack(val));
        aux
    }

    pub fn run_linker(&mut self) {
        utilities::context_title(Some("Emulating linker64"));
        let res = self.emu_start(self.entry_point, self.elf_entry, 0, 0);
        self.handle_emu_exception(res);
        utilities::context_title(Some("Emulating linker64 done"));
        // self.display_mapped();
    }
}