use std::path::Path;
use crate::core::{uid, gid};
use crate::core::rudroid::Emulator;
use nix::sys::stat::{fstatat, fstat};
use nix::sys::statfs::{Statfs, fstatfs};

impl<D> Emulator<D> {
    pub fn sys_fstatat(&mut self) {
        // sys_fstatat64(int dfd, const char __user *filename, struct stat64 __user *statbuf, int flag);
        let dirfd         = self.get_arg(0);
        let filename_ptr = self.get_arg(1); 
        let statbuf      = self.get_arg(2);
        let flag         = self.get_arg(3);

        let filename = self.get_string(filename_ptr);
        
        let fullpath = match self.filesystem.is_driver_io(&filename) {
            true => {
                filename
            },
            false => {
                format!("{}/{}", self.rootfs, filename)
            }
        };

        let fullpath: &str = &fullpath;
        if Path::new(fullpath).exists() {
            let result = fstatat(dirfd as i32, fullpath, unsafe { std::mem::transmute(flag as u32) }).unwrap();
            let mut fsstatbuf  = Vec::new();
    
            fsstatbuf.extend_from_slice(&self.pack_64(result.st_dev as u64));
            fsstatbuf.extend_from_slice(&self.pack_64(result.st_ino));

            fsstatbuf.extend_from_slice(&self.pack_32(result.st_mode as u32));
            fsstatbuf.extend_from_slice(&self.pack_32(result.st_nlink as u32));
            fsstatbuf.extend_from_slice(&self.pack_32(1000));
            fsstatbuf.extend_from_slice(&self.pack_32(1000));

            fsstatbuf.extend_from_slice(&self.pack_64(result.st_rdev as u64));
            fsstatbuf.extend_from_slice(&self.pack_64(0));
            fsstatbuf.extend_from_slice(&self.pack_64(result.st_size as u64));

            fsstatbuf.extend_from_slice(&self.pack_32(result.st_blksize as u32));
            fsstatbuf.extend_from_slice(&self.pack_32(0));
    
            fsstatbuf.extend_from_slice(&self.pack_64(result.st_blocks as u64));
            fsstatbuf.extend_from_slice(&self.pack_64(result.st_atime as u64));
            fsstatbuf.extend_from_slice(&self.pack_64(0));
            
            fsstatbuf.extend_from_slice(&self.pack_64(result.st_mtime as u64));
            fsstatbuf.extend_from_slice(&self.pack_64(0));
    
            fsstatbuf.extend_from_slice(&self.pack_64(result.st_ctime as u64));
            fsstatbuf.extend_from_slice(&self.pack_64(0));  

            self.mem_write(statbuf, &fsstatbuf);
            self.set_return_val(0);

            // println!("sys_fstatat dirfd: {} filename {}: statbuf {:x}", dirfd, fullpath, statbuf);
            if self.debug {
                self.debug_print(format!("sys_fstatat dirfd: {} filename {}: statbuf {}", dirfd, fullpath, statbuf));
            }
        }
        else {
            self.set_return_val(0xffff_ffff);
        }
    }

    pub fn sys_fstat(&mut self) {
        // sys_fstat(unsigned int fd, struct __old_kernel_stat __user *statbuf);
        let fd = self.get_arg(0);
        let statbuf = self.get_arg(1);

        let fstat_info = fstat(fd as i32).unwrap();
        let mut fstatbuf = Vec::new();
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_dev as u64));
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_ino as u64));
        fstatbuf.extend_from_slice(&self.pack_32(fstat_info.st_mode as u32));
        fstatbuf.extend_from_slice(&self.pack_32(fstat_info.st_nlink as u32));

        fstatbuf.extend_from_slice(&self.pack_32(uid));
        fstatbuf.extend_from_slice(&self.pack_32(gid));
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_rdev as u64));
        fstatbuf.extend_from_slice(&self.pack_64(0));
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_size  as u64));
        fstatbuf.extend_from_slice(&self.pack_32(fstat_info.st_blksize as u32));
        fstatbuf.extend_from_slice(&self.pack_32(0));
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_blocks  as u64));
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_atime  as u64));
        fstatbuf.extend_from_slice(&self.pack_64(0));
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_mtime as u64));
        fstatbuf.extend_from_slice(&self.pack_64(0));
        fstatbuf.extend_from_slice(&self.pack_64(fstat_info.st_ctime as u64));
        fstatbuf.extend_from_slice(&self.pack_64(0));

        self.mem_write(statbuf, &fstatbuf);

        self.set_return_val(0);
    }

    pub fn sys_fstatfs(&mut self) {
        // sys_fstatfs(unsigned int fd, struct statfs __user *buf);
        let fd = self.get_arg(0) as i32;
        let fstatbuf = self.get_arg(1);
        
        let mut statbuf = Vec::new();

        statbuf.extend_from_slice(&self.pack_64(0xef53));
        statbuf.extend_from_slice(&self.pack_64(0x1000));
        statbuf.extend_from_slice(&self.pack_64(0x3235af));
        statbuf.extend_from_slice(&self.pack_64(0x2b5763));
        statbuf.extend_from_slice(&self.pack_64(0x2b5763));
        statbuf.extend_from_slice(&self.pack_64(0xcccb0));
        statbuf.extend_from_slice(&self.pack_64(0xcbd2e));
        statbuf.extend_from_slice(&self.pack_64(0xd3609fe8));
        statbuf.extend_from_slice(&self.pack_64(0x4970d6b));
        statbuf.extend_from_slice(&self.pack_64(0xff));
        statbuf.extend_from_slice(&self.pack_64(0x1000));
        statbuf.extend_from_slice(&self.pack_64(0x426));
        self.mem_write(fstatbuf, &statbuf);
        self.set_return_val(0);        
    }

}