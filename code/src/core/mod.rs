pub mod mmu;
pub mod hooks;
pub mod android;
pub mod loaders;
pub mod rudroid;
pub mod unicorn;


const ram_size      : u64 = 0xa00000;
const entry_point   : u64 = 0x1000000;

#[repr(u64)]
pub enum OS64 {
    stack_address       = 0x4ffffffde000,
    stack_size          = 0x30000,
    load_address        = 0x555555554000,
    interp_address      = 0x7fffb7dd5000,
    mmap_address        = 0x7ffff7dd6000,
    vsyscall_address    = 0xffffffffff600000,
    vsyscall_size       = 0x1000,
}

// [KERNEL]
pub const uid : u32 = 0;
pub const gid : u32 = 0;