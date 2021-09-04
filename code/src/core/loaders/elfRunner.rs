use super::super::unicorn::ffi;
use super::super::rudroid;
use super::super::unicorn::unicorn_const;
use super::super::unicorn::unicorn_const::*;
use crate::utilities;
use capstone::prelude::*;
use keystone::keystone_const;
use keystone::{Keystone, Arch as kArch};
use super::super::unicorn::arch::arm64::RegisterARM64;

use capstone::prelude::*;

impl<D> rudroid::Emulator<D> {
    pub fn run_elf(&mut self) {
        utilities::context_title(Some("Emulating elf"));
        let res = self.emu_start(self.elf_entry, 0, 0, 0);
        self.handle_emu_exception(res);
        utilities::context_title(Some("Emulating elf done."));
    }

    pub fn fuzz_init(&mut self) -> u64 {
        let emu_addr = self.uc_align_up(0x14141414141);
        self.mem_map(emu_addr, 0x1000, Protection::ALL).expect("Code Emulation");
    
        let code = "stp fp, lr, [sp, #-16]!\nmov fp, sp\nmov fp, sp\nblr x12\nldp fp, lr, [sp], #16\nret lr";
        let ks_arm: Keystone = Keystone::new(kArch::ARM64, keystone_const::MODE_LITTLE_ENDIAN).expect("Could not initialize Keystone engine");;
        let result = ks_arm.asm(code.to_string(), 0).expect("Could not asemble");
        self.mem_write(emu_addr, &result.bytes);
        emu_addr
    }

    pub fn call_me(&mut self, func_addr: u64, emu_addr: u64) {
        //assumes arguments are already set
        self.reg_write(RegisterARM64::X12 as i32, func_addr); // function to emulate
        self.reg_write(RegisterARM64::LR as i32, 0);   //should return on 0

        let res = self.emu_start(emu_addr, 0x14141415014, 0, 0);
        self.handle_emu_exception(res);
    }

    pub fn handle_emu_exception(&mut self, err: Result<(), unicorn_const::uc_error>) {
        // self.get_mapped();

        match err.ok() {
            Some(v) => {

            },
            None => {
                self.display_mapped();
                self.dump_context();

                match err.err().unwrap() {
                    unicorn_const::uc_error::FETCH_UNMAPPED => {
                        panic!("- [handle_emu_exception] unicorn::unicorn_const::uc_error::FETCH_UNMAPPED");
                    },
                    _ => {
                        panic!("- [handle_emu_exception] {:?}", err);
                    }
                }
            }
        }
    }

    pub fn dump_context(&mut self) {
        utilities::draw_line();
    
        let pc  = self.reg_read(RegisterARM64::PC  as i32).expect("failed to read r11"); 
        let sp  = self.reg_read(RegisterARM64::SP  as i32).expect("failed to read SP" );
        let lr  = self.reg_read(RegisterARM64::LR  as i32).expect("failed to read LR" );
        let r0  = self.reg_read(RegisterARM64::X0  as i32).expect("failed to read r0" );
        let r1  = self.reg_read(RegisterARM64::X1  as i32).expect("failed to read r1" );
        let r2  = self.reg_read(RegisterARM64::X2  as i32).expect("failed to read r2" );
        let r3  = self.reg_read(RegisterARM64::X3  as i32).expect("failed to read r3" );
        let r4  = self.reg_read(RegisterARM64::X4  as i32).expect("failed to read r4" );
        let r5  = self.reg_read(RegisterARM64::X5  as i32).expect("failed to read r5" );
        let r6  = self.reg_read(RegisterARM64::X6  as i32).expect("failed to read r6" );
        let r7  = self.reg_read(RegisterARM64::X7  as i32).expect("failed to read r7" );
        let r8  = self.reg_read(RegisterARM64::X8  as i32).expect("failed to read r8" );
        let r9  = self.reg_read(RegisterARM64::X9  as i32).expect("failed to read r9" );
        let r10 = self.reg_read(RegisterARM64::X10 as i32).expect("failed to read r10");
        let r11 = self.reg_read(RegisterARM64::X11 as i32).expect("failed to read r11");
        let r12 = self.reg_read(RegisterARM64::X12 as i32).expect("failed to read r11");
        let r13 = self.reg_read(RegisterARM64::X13 as i32).expect("failed to read r11");
        let r14 = self.reg_read(RegisterARM64::X14 as i32).expect("failed to read r11");
        let r15 = self.reg_read(RegisterARM64::X15 as i32).expect("failed to read r11");
        let r18 = self.reg_read(RegisterARM64::X18 as i32).expect("failed to read r11");
        let r19 = self.reg_read(RegisterARM64::X19 as i32).expect("failed to read r11");
        let r20 = self.reg_read(RegisterARM64::X20 as i32).expect("failed to read r11");
        let r21 = self.reg_read(RegisterARM64::X21 as i32).expect("failed to read r11");
        let r22 = self.reg_read(RegisterARM64::X22 as i32).expect("failed to read r11");
        let r23 = self.reg_read(RegisterARM64::X23 as i32).expect("failed to read r11");
        let r24 = self.reg_read(RegisterARM64::X24 as i32).expect("failed to read r11");
        let r25 = self.reg_read(RegisterARM64::X25 as i32).expect("failed to read r11");
        let r26 = self.reg_read(RegisterARM64::X26 as i32).expect("failed to read r11");
        let r27 = self.reg_read(RegisterARM64::X27 as i32).expect("failed to read r11");
        let r28 = self.reg_read(RegisterARM64::X28 as i32).expect("failed to read r11");
        
        let cpacr_el1 = self.reg_read(RegisterARM64::CPACR_EL1 as i32).expect("failed to read r11"); 
        utilities::draw_line();
    
        println!("$r0 : {:#016x}   $r1 : {:#016x}    $r2: {:#016x}    $r3: {:#016x}", r0, r1, r2, r3);
        println!("$r4 : {:#016x}   $r5 : {:#016x}    $r6: {:#016x}    $r7: {:#016x}", r4, r5, r6, r7);
        println!("$r8 : {:#016x}   $r9 : {:#016x}   $r10: {:#016x}   $r11: {:#016x}", r8, r9, r10, r11);
        println!("$r12: {:#016x}   $r13: {:#016x}   $r14: {:#016x}   $r15: {:#016x}", r12, r13, r14, r15);
        println!("$r18: {:#016x}   $r19: {:#016x}   $r20: {:#016x}   $r21: {:#016x}", r18, r19, r20, r21);
        println!("$r22: {:#016x}   $r23: {:#016x}   $r24: {:#016x}   $r25: {:#016x}", r22, r23, r24, r25);
        println!("$r26: {:#016x}   $r27: {:#016x}   $r28: {:#016x}   ", r26, r27, r28);
        println!("$sp : {:#016x}   $lr : {:#016x}    $pc: {:#016x}", sp, lr, pc);
        println!("$cpacr_el1: {:#016x} \n", cpacr_el1);
        
        utilities::draw_line();
    }

}
