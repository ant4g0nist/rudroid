use capstone::prelude::*;

use super::android;
use super::rudroid;
use super::unicorn::arch::arm64;
use crate::utilities;
use super::unicorn::unicorn_const;

pub fn add_hooks(emu: &mut rudroid::Emulator<i64>) {
    // hook syscalls: https://github.com/unicorn-engine/unicorn/issues/1137
    emu.add_intr_hook(android::syscalls::hook_syscall).unwrap();    
    
    emu.add_mem_hook(unicorn_const::HookType::MEM_FETCH_UNMAPPED, 1, 0, callback_mem_error).unwrap();
    emu.add_mem_hook(unicorn_const::HookType::MEM_READ_UNMAPPED, 1, 0, callback_mem_error).unwrap();
}

// hooks
pub fn callback(uc: &mut rudroid::Emulator<i64>, address: u64, size: u32) {
    // dump_context(uc, address, size as usize);
    println!("addr: {:x}", address);
}

pub fn callback_mem_error(uc: &mut rudroid::Emulator<i64>, memtype: unicorn_const::MemType, address: u64, size: usize, value: i64) {
    println!("callback_mem_error {:x}", address);
    dump_context(uc, address, size);
}

pub fn callback_mem_rw(uc: &mut rudroid::Emulator<i64>, memtype: unicorn_const::MemType, address: u64, size: usize, value: i64) {
    println!("callback_mem_rw {:x}", address);
    dump_context(uc, address, size);
}

pub fn dump_context(uc: &mut rudroid::Emulator<i64>, addr: u64, size: usize) {
    utilities::draw_line();

    let pc  = uc.reg_read(arm64::RegisterARM64::PC  as i32).expect("failed to read PC"); 
    let sp  = uc.reg_read(arm64::RegisterARM64::SP  as i32).expect("failed to read SP" );
    let lr  = uc.reg_read(arm64::RegisterARM64::LR  as i32).expect("failed to read LR" );
    let r0  = uc.reg_read(arm64::RegisterARM64::X0  as i32).expect("failed to read x0" );
    let r1  = uc.reg_read(arm64::RegisterARM64::X1  as i32).expect("failed to read x1" );
    let r2  = uc.reg_read(arm64::RegisterARM64::X2  as i32).expect("failed to read x2" );
    let r3  = uc.reg_read(arm64::RegisterARM64::X3  as i32).expect("failed to read x3" );
    let r4  = uc.reg_read(arm64::RegisterARM64::X4  as i32).expect("failed to read x4" );
    let r5  = uc.reg_read(arm64::RegisterARM64::X5  as i32).expect("failed to read x5" );
    let r6  = uc.reg_read(arm64::RegisterARM64::X6  as i32).expect("failed to read x6" );
    let r7  = uc.reg_read(arm64::RegisterARM64::X7  as i32).expect("failed to read x7" );
    let r8  = uc.reg_read(arm64::RegisterARM64::X8  as i32).expect("failed to read x8" );
    let r9  = uc.reg_read(arm64::RegisterARM64::X9  as i32).expect("failed to read x9" );
    let r10 = uc.reg_read(arm64::RegisterARM64::X10 as i32).expect("failed to read x10");
    let r11 = uc.reg_read(arm64::RegisterARM64::X11 as i32).expect("failed to read x11");
    let r12 = uc.reg_read(arm64::RegisterARM64::X12 as i32).expect("failed to read x12");
    let r13 = uc.reg_read(arm64::RegisterARM64::X13 as i32).expect("failed to read x13");
    let r14 = uc.reg_read(arm64::RegisterARM64::X14 as i32).expect("failed to read x14");
    let r15 = uc.reg_read(arm64::RegisterARM64::X15 as i32).expect("failed to read x15");
    let r18 = uc.reg_read(arm64::RegisterARM64::X18 as i32).expect("failed to read x18");
    let r19 = uc.reg_read(arm64::RegisterARM64::X19 as i32).expect("failed to read x19");
    let r20 = uc.reg_read(arm64::RegisterARM64::X20 as i32).expect("failed to read x20");
    let r21 = uc.reg_read(arm64::RegisterARM64::X21 as i32).expect("failed to read x21");
    let r22 = uc.reg_read(arm64::RegisterARM64::X22 as i32).expect("failed to read x22");
    let r23 = uc.reg_read(arm64::RegisterARM64::X23 as i32).expect("failed to read x23");
    let r24 = uc.reg_read(arm64::RegisterARM64::X24 as i32).expect("failed to read x24");
    let r25 = uc.reg_read(arm64::RegisterARM64::X25 as i32).expect("failed to read x25");
    let r26 = uc.reg_read(arm64::RegisterARM64::X26 as i32).expect("failed to read x26");
    let r27 = uc.reg_read(arm64::RegisterARM64::X27 as i32).expect("failed to read x27");
    let r28 = uc.reg_read(arm64::RegisterARM64::X28 as i32).expect("failed to read x28");
    
    let cpacr_el1 = uc.reg_read(arm64::RegisterARM64::CPACR_EL1 as i32).expect("failed to read CPACR_EL1"); 
    utilities::draw_line();

    println!("$x0 : {:#016x}   $x1 : {:#016x}    $x2: {:#016x}    $x3: {:#016x}", r0, r1, r2, r3);
    println!("$x4 : {:#016x}   $x5 : {:#016x}    $x6: {:#016x}    $x7: {:#016x}", r4, r5, r6, r7);
    println!("$x8 : {:#016x}   $x9 : {:#016x}   $x10: {:#016x}   $x11: {:#016x}", r8, r9, r10, r11);
    println!("$x12: {:#016x}   $x13: {:#016x}   $x14: {:#016x}   $x15: {:#016x}", r12, r13, r14, r15);
    println!("$x18: {:#016x}   $x19: {:#016x}   $x20: {:#016x}   $x21: {:#016x}", r18, r19, r20, r21);
    println!("$x22: {:#016x}   $x23: {:#016x}   $x24: {:#016x}   $x25: {:#016x}", r22, r23, r24, r25);
    println!("$x26: {:#016x}   $x27: {:#016x}   $x28: {:#016x}   ", r26, r27, r28);
    println!("$sp : {:#016x}   $lr : {:#016x}    $pc: {:#016x}", sp, lr, pc);
    println!("$cpacr_el1: {:#016x} \n", cpacr_el1);

    let mut buf = vec![0; size];
    if pc != 0 {
        uc.mem_read(pc, &mut buf).expect("failed to read opcode from memory");
        let cs_arm: Capstone = Capstone::new()
            .arm64()
            .mode(arch::arm64::ArchMode::Arm)
            .detail(true)
            .build().expect("failed to create capstone for ARM");

        let ins = cs_arm.disasm_all(&buf, size as u64).unwrap();
        println!("$pc: {:#016x}", pc);
        println!("{}", ins);
    }

    // let stack = uc.mem_read_as_vec(sp, 0x60).unwrap();
    // utilities::context_title(Some("STACK"));
    // print!("{}",utilities::pretty_hex(&stack, sp));
    
    utilities::draw_line();
}
