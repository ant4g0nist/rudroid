extern crate byteorder;
extern crate capstone;
extern crate keystone;
extern crate nix;
extern crate xmas_elf;

mod utilities;
mod core;

use std::env;
use xmas_elf::ElfFile;

use crate::utilities::context_title;

fn parse_args() -> env::Args {
    //! Parse Command line arguments
    let mut args = env::args();

    if args.len() != 3 {
        panic!("Please provide an ELF library and rootfs folder");
    }
    args
}

fn main()
{
    utilities::context_title(Some("Hello, world!"));
    let mut args = parse_args();
    let mut elf_filename = args.nth(1).unwrap();
    let rootfs       = args.next().unwrap();
    
    let mut elf_data    = std::fs::read(&mut elf_filename).unwrap();
    let mut elf: ElfFile        = ElfFile::new(&mut elf_data).unwrap();

    //our hello world program takes no arguments or environment variables
    let program_args: Vec<String>   = vec![];
    let program_env: Vec<String>    = Vec::new();
    
    let endian =  elf.header.pt1.data();
    let mut emu = core::rudroid::Emulator::new( &elf_filename, &rootfs, &mut elf, endian, program_args, program_env, 0, true).expect("Emulator initialisation failed");
    
    //set up hooks
    core::hooks::add_hooks(&mut emu);

    //run linker to load dependencies of ELF and then run the main from ELF
    emu.run_linker();
    emu.run_elf();

    context_title(Some("Emulator creted"))
}