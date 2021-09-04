use libc::c_void;
use xmas_elf::header;
use std::collections::HashMap;

use super::rudroid::Emulator;
use super::unicorn::unicorn_const::Protection;
use super::unicorn::arch::arm64::RegisterARM64;
use byteorder::{ByteOrder, BigEndian, LittleEndian};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MapInfo {
    pub memory_start    : u64,
    pub memory_end      : u64,
    pub memory_perms    : Protection,
    pub description     : String,
}

impl Clone for MapInfo {
    fn clone(&self) -> Self {
        MapInfo {
            memory_start    : self.memory_start,
            memory_end      : self.memory_end,
            memory_perms    : self.memory_perms,
            description     : self.description.clone(),
        }
    }    
}

impl std::fmt::Display for MapInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "memory_start {:x} memory_end {:x} memory_perms : {}  description: {}", self.memory_start, self.memory_end, self.memory_perms, self.description).unwrap();
        Ok(())
    }
}

impl<D> Emulator<D> {
    pub fn mmu_map(&mut self, address: u64, size: usize, perms: Protection, description: &str, host_ptr: *mut c_void) -> bool {

        if self.is_mapped(address, size as u64) {
            return true;
        }

        if host_ptr.is_null() {
            // let _ = self.mem_map(address, size, perms).expect("mmu_map failed!");
            let _ = self.mem_map(address, size, perms).unwrap();
        }
        else {
            // Map an existing memory region in the emulator at the specified address.
            //
            // This function is marked unsafe because it is the responsibility of the caller to
            // ensure that `size` matches the size of the passed buffer, an invalid `size` value will
            // likely cause a crash in unicorn.
            //
            // `address` must be aligned to 4kb or this will return `Error::ARG`.
            //
            // `size` must be a multiple of 4kb or this will return `Error::ARG`.
            //
            // `ptr` is a pointer to the provided memory region that will be used by the emulator.                
            let _ = unsafe { self.mem_map_ptr(address, size, perms, host_ptr).expect("mem_map_ptr failed !"); };
        }

        let desc = match description.len() {
            0 => {
                String::from("[mapped]")
            },
            _ => {
                String::from(description)
            }
        };

        let map_info = MapInfo {
            memory_start    : address,
            memory_end      : address.checked_add(size as u64).unwrap(),
            memory_perms    : perms,
            description     : desc,
        };

        self.add_mapinfo(map_info);        

        true
    }

    pub fn add_mapinfo(&mut self, map_info: MapInfo) {
        // self.map_infos.push(map_info);
        self.map_infos.insert(map_info.memory_start, map_info);
    }

    pub fn mmu_unmap(&mut self, address: u64, size: usize) {
        let removed = self.map_infos.remove_entry(&address);
        self.mem_unmap(address, size);
    }

    pub fn is_mapped(&mut self, address: u64, size: u64) -> bool {
        let regions = self.mem_regions().unwrap();
        if regions.len() <= 1 {
            return false;
        } 

        for region in self.mem_regions() {
            let val = (region[0].begin >= address) & ((address + size - 1) <= region[1].begin);
            match val {
                true => {
                    return true;
                },
                _ => {
                    
                }
            }
        };

        false
    }    

    pub fn mmu_mem_set(&mut self, addr: u64, value: char, size: usize) {
        let data = vec![value; size].iter().map(|c| *c as u8).collect::<Vec<_>>();
        self.mem_write(addr, &data).expect("mmu_mem_set failed");
    }

    pub fn get_mapped(&self, address: u64, len: u64) -> Option<&MapInfo> {
        for (addr, map_info) in self.map_infos.iter() {
            if address > *addr && address + len < map_info.memory_end {
                return Some(&map_info);
            }
        }
        None
    }

    pub fn update_mapped_perms(&mut self, address: u64, len: u64, perms: Protection) -> Option<&MapInfo> {
        for (addr, map_info) in self.map_infos.iter_mut() {
            if address > *addr && address + len < map_info.memory_end {
                // return Some(&map_info);
                map_info.memory_perms = perms;
            }
        }
        None
    }

    pub fn get_mapped_with_desc(&self, desc: String) -> HashMap<u64, MapInfo> {
        let mut mappings: HashMap<u64, MapInfo> = HashMap::new();

        for (addr, map_info) in self.map_infos.iter() {
            if map_info.description == desc  {
                mappings.insert(*addr, map_info.clone());
            }
        }

        mappings
    }

    pub fn display_mapped(&self) {
        let mut v: Vec<_> = Vec::new();
        for (addr, map_info) in self.map_infos.iter() {
            v.push((addr, map_info));
        }
        v.sort_by(|x,y| x.0.cmp(&y.0));
        
        for (addr, map_info) in v {
            println!("{}", map_info);
        }
    }

    pub fn read(&self, address: u64, size: usize) -> Vec<u8> {
        self.mem_read_as_vec(address, size).unwrap()
    }

    pub fn write(&mut self, address: u64, data: &[u8]) {
        self.mem_write(address, data).expect("mmu.write");
    }

    pub fn copy_str(&mut self, address: u64, string: &mut str) -> u64 {
        let data = string.as_bytes();
        let address: u64 = ((address as usize)  - data.len() - 1) as u64;
        self.write(address, data);
        address
    }

    fn stack_push(&mut self, value: u64) {
        let mut sp = self.read_sp();
        sp = sp-8;
        self.write_sp(sp);

        self.write(sp, &self.pack_64(value));
    }

    fn stack_pop(&mut self) -> u64 {
        let mut sp = self.read_sp();
        sp = sp-8;
        self.write_sp(sp);

        let data = self.read(sp, 8);
        self.unpack_64(&data)
    }

    fn stack_read(&mut self, offset: u32) -> u64 {
        let mut sp = self.read_sp();
        let data = self.read(sp+ offset as u64, 8);
        self.unpack_64(&data)
    }

    fn stack_write(&mut self, offset: usize, data: &[u8]) {
        let mut sp = self.read_sp() + offset as u64;
        self.write(sp, data);
    }

    fn read_sp(&mut self) -> u64 {
        self.reg_read(RegisterARM64::SP as i32).unwrap()
    }

    fn write_sp(&mut self, value: u64) {
        self.reg_write(RegisterARM64::SP as i32, value);
    }

    fn read_pc(&mut self) -> u64 {
        self.reg_read(RegisterARM64::PC as i32).unwrap()
    }

    fn write_pc(&mut self, value: u64) {
        self.reg_write(RegisterARM64::PC as i32, value);
    }

    // unsigned pack
    pub fn pack(&self, value: u64) -> Vec<u8> { 
        match self.machine {
            header::Machine::AArch64 => {
                self.pack_64(value)
            },
            _ => {
                panic!("");
            }
        }
    }

    // unsigned unpack
    pub fn unpack(&self, value: &[u8]) -> u64 { 
        match self.machine {
            header::Machine::AArch64 => {
                if value.len() == 8 {
                    self.unpack_64(value)
                }
                else if (value.len() == 4) {
                    return self.unpack_32(value) as u64;
                }
                else {
                    panic!("need 4 or 8 byte");
                }   
            },
            _ => {
                panic!("");
            }
        }
    }

    pub fn pack_64(&self, value: u64) -> Vec<u8> {
        match self.endian {
            header::Data::BigEndian  => {
                value.to_be_bytes().to_vec()
            },
            header::Data::LittleEndian => {
                value.to_le_bytes().to_vec()
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        }
    }
    
    pub fn pack_64s(&self, value: i64, buf: &mut [u8]) {
        match self.endian {
            header::Data::BigEndian  => {
                BigEndian::write_i64(buf, value)
            },
            header::Data::LittleEndian => {
                // LittleEndian::write_i64(buf, value)
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        };
    }
    
    pub fn unpack_64(&self, value: &[u8]) -> u64 {
        match self.endian {
            header::Data::BigEndian  => {
                BigEndian::read_u64(value)
            },
            header::Data::LittleEndian => {
                LittleEndian::read_u64(value)
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        }
    }
    
    pub fn unpack_64s(&self, value: &[u8]) -> i64 {
        match self.endian {
            header::Data::BigEndian  => {
                BigEndian::read_i64(value)
            },
            header::Data::LittleEndian => {
                LittleEndian::read_i64(value)
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        }
    }
    
    pub fn pack_32(&self, value: u32) -> Vec<u8> {        
        match self.endian {
            header::Data::BigEndian  => {
                value.to_be_bytes().to_vec()
            },
            header::Data::LittleEndian => {
                value.to_le_bytes().to_vec()
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        }
    }
    
    pub fn pack_32s(&self, value: i32, buf: &mut [u8]) {
        match self.endian {
            header::Data::BigEndian  => {
                BigEndian::write_i32(buf, value)
            },
            header::Data::LittleEndian => {
                LittleEndian::write_i32(buf, value)
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        };
    }

    pub fn unpack_32(&self, value: &[u8]) -> u32 {        
        match self.endian {
            header::Data::BigEndian  => {
                BigEndian::read_u32(value)
            },
            header::Data::LittleEndian => {
                LittleEndian::read_u32(value)
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        }
    }
    
    pub fn unpack_32s(&self, value: &[u8]) -> i32 {
        match self.endian {
            header::Data::BigEndian  => {
                BigEndian::read_i32(value)
            },
            header::Data::LittleEndian => {
                LittleEndian::read_i32(value)
            },
            _ => {
                panic!("what kiinda endian is this")
            }
        }
    }

    pub fn get_pointer_at(&mut self, addr: u64) -> u64 {
        match self.machine {
            header::Machine::AArch64 => {
                let mem = self.mem_read_as_vec(addr, 8).unwrap();
                self.unpack(&mem)
            },
            _ => {
                panic!("get_pointer_at 0x{:x} failed", addr);
            }
        }
    }
}