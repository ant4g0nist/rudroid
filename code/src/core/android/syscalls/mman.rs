use xmas_elf::program;

use crate::utilities;
use crate::core::mmu::MapInfo;
use crate::core::rudroid::Emulator;
use crate::core::android::fs::MAX_FDS;
use crate::core::unicorn::unicorn_const::Protection;

const MAP_FIXED      : u64 = 0x10;
const MAP_ANONYMOUS  : u64 = 0x20;
const MREMAP_MAYMOVE : u64 = 0x1;

impl<D> Emulator<D> {
    pub fn sys_mmap(&mut self) {
        let addr    = self.get_arg(0);
        let len     = self.get_arg(1);
        let prot    = self.get_arg(2);
        let flags   = self.get_arg(3);
        let fd  : i32    = self.get_arg(4) as i32;
        let off     = self.get_arg(5) ;

        let aligned_len = self.align_len(len);

        let mut mmap_base = addr;
        let mut need_map : bool = true;

        if addr == 0 {
            mmap_base         = self.mmap_address;
            self.mmap_address = mmap_base + aligned_len;
        }
        
        else {
            need_map = false;
        }
        
        let is_fixed = (flags & MAP_FIXED) != 0;
        if self.debug {
            self.debug_print(format!("mmap_base 0x{:x} length 0x{:x} fixed: {} = ({:x}, {:x})", addr, len, is_fixed, mmap_base, aligned_len as usize));
        }

        if need_map {
            self.mmu_map(mmap_base, aligned_len as usize, Protection::ALL, "[syscall_mmap]", self.null_mut());
        }

        if (( flags & MAP_ANONYMOUS) == 0 ) && fd < MAX_FDS && fd > 0 {
            let mut data = vec![0u8; len as usize];
            self.filesystem.pread(fd, &mut data, off).unwrap();

            let mem_info: &str = &self.filesystem.get_path(fd).unwrap();

            let map_info = MapInfo {
                memory_start    : mmap_base,
                memory_end      : mmap_base+((len+0x1000-1)/0x1000) * 0x1000,
                memory_perms    : Protection::ALL,
                description     : String::from(mem_info),
            };

            self.add_mapinfo(map_info);
            self.write(mmap_base, &data);
        }

        self.set_return_val(mmap_base);
    }

    pub fn sys_mprotect(&mut self) {
        let start  = self.get_arg(0);
        let len    = self.get_arg(1); 
        let prot   = self.get_arg(2);

        if self.debug {
            self.debug_print(format!("mprotect(0x{:x}, 0x{:x}, {})", start, len, prot));
        }
        self.set_return_val(0);
    }

    pub fn sys_munmap(&mut self) {
        let address = self.get_arg(0);
        let len = self.get_arg(1);

        if self.debug {
            self.debug_print(format!("sys_munmap(0x{:x}, 0x{:x})",address, len));
        }
        
        let aligned = ((len + 0x1000 - 1) / 0x1000) * 0x1000;
        self.mmu_unmap(address, aligned as usize);
        // self.munmap(address, len, aligned);
        self.set_return_val(0);
    }

    pub fn munmap(&mut self, address: u64, len: u64, aligned: u64) {
        let removed = self.map_infos.remove_entry(&address);

        let mut nmap_info : MapInfo = MapInfo {
            memory_start    : 0,
            memory_end      : 0,
            memory_perms    : Protection::NONE,
            description     : String::new()  
        };

        match removed {
            Some(v) => {
                let addr = v.0;
                let map_info = v.1;

                let removed_size: u64 = map_info.memory_end - map_info.memory_start;

                if map_info.memory_end - map_info.memory_start != aligned {

                    if aligned >= removed_size {
                        if self.debug {
                            self.debug_print(format!("sys_munmap removed=0x{:x} aligned=0x{:x} start=0x{:x} ",removed_size, aligned, address));
                        }
                        let mut addr = address + removed_size;
                        let mut size = aligned - removed_size;
                        loop {
                            if size == 0 {
                                break;
                            }
    
                            self.map_infos.remove_entry(&addr);
                            addr += removed_size;
                            size -= removed_size;
                        }
                        // return ;
                    }

                    
                    nmap_info = MapInfo {
                        memory_start    : address,
                        memory_end      : address + aligned,
                        memory_perms    : map_info.memory_perms,
                        description     : map_info.description.clone()
                    };
                }

                if self.map_infos.is_empty() {
                    self.mmap_address = address;
                }
            },
            None => {
                for (addr, map_info) in self.map_infos.iter_mut() { 
                     if address > *addr && address < map_info.memory_end {
                        if address + aligned < map_info.memory_end {
                            let new_size = map_info.memory_end - address - aligned;
                            
                            if self.debug {
                                println!("sys_munmap aligned = 0x{:x} start=0x{:x} base=0x{:x} size={}", aligned, address, address+aligned, new_size);
                            }
                            
                            nmap_info = MapInfo {
                                memory_start    : address + aligned,
                                memory_end      : address + aligned + new_size,
                                memory_perms    : map_info.memory_perms,
                                description     : map_info.description.clone()
                            };
                        }
                     } 
                }
            }
        };

        if nmap_info.memory_start != 0 {
            // self.add_mapinfo(nmap_info);
            self.debug_print(format!("sys_munmap address: 0x{:x} len=0x{:x} ",nmap_info.memory_start, (nmap_info.memory_end - nmap_info.memory_start)));

            let desc: &str = &nmap_info.description;
            self.mmu_map(nmap_info.memory_start, (nmap_info.memory_end - nmap_info.memory_start) as usize, nmap_info.memory_perms, "munmap", self.null_mut());
        }               
    }

    pub fn sys_mremap(&mut self) {
        // sys_mremap(unsigned long addr, unsigned long old_len, unsigned long new_len, unsigned long flags, unsigned long new_addr);

        let address = self.get_arg(0);
        let old_len = self.get_arg(1);
        let new_len = self.get_arg(2);
        let flags = self.get_arg(3);
        let new_addr = self.get_arg(4);

        let aligned_old_len = ((old_len + 0x1000 - 1) / 0x1000) * 0x1000;
        let aligned_new_len = ((new_len + 0x1000 - 1) / 0x1000) * 0x1000;

        if self.debug {
            self.debug_print(format!("sys_mremap(0x{:x},0x{:x},0x{:x},0x{:x}, 0x{:x})", address, old_len, new_len, flags, new_addr));
        }

        if old_len == 0 {
            panic!("sys_mremap: old_size is zero");
        }

        if flags & MREMAP_MAYMOVE == 0 {
            panic!("sys_mremap: flags = {:x}", flags);
        }

        let mut nmap_info : MapInfo = MapInfo {
            memory_start    : 0,
            memory_end      : 0,
            memory_perms    : Protection::NONE,
            description     : String::new()  
        };

        for (addr, map_info) in self.map_infos.iter_mut() { 
            if address >= map_info.memory_start && address + old_len <= map_info.memory_end {
                nmap_info = MapInfo {
                    memory_start    : map_info.memory_start,
                    memory_end      : map_info.memory_end,
                    memory_perms    : map_info.memory_perms,
                    description     : map_info.description.clone()
                };
            }
        }

        if nmap_info.memory_start != nmap_info.memory_end && nmap_info.memory_end != 0 {
            
            let len = nmap_info.memory_end - nmap_info.memory_start;
            
            let data = self.mem_read_as_vec(nmap_info.memory_start, len as usize).unwrap();
            self.mmu_unmap(address, len as usize);
            let mmap_base = self.mmap_address;
            self.mmap_address = self.uc_align_down(mmap_base + aligned_new_len);
            self.mmu_map(mmap_base, aligned_new_len as usize, Protection::ALL, "[syscall_mmap]", self.null_mut());
            self.mem_write(mmap_base, &data);
            println!("memory_start {:x} memory_end {:x} memory_perms : {}  description: {}", nmap_info.memory_start, nmap_info.memory_end, nmap_info.memory_perms, nmap_info.description);

            self.set_return_val(mmap_base);
            return;
        }

        self.set_return_val(0xffff_ffff);
    }    

}
