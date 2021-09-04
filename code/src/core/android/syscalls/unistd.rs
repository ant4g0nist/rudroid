use std::process;
use crate::core::{uid, gid};
use crate::core::rudroid::Emulator;

impl<D> Emulator<D> {
    pub fn sys_getpid(&mut self) {
        let pid = 1337;
        self.set_return_val(pid as u64);
    }

    pub fn sys_set_tid_address(&mut self) {
        let tidptr       = self.get_arg(0);

        let pid          = process::id();
        let pid_buf  = self.pack_32(pid);

        self.mem_write(tidptr, &pid_buf);
        self.set_return_val(pid as u64);        
    }

    pub fn sys_faccessat(&mut self) {
        // faccessat(int dfd, const char __user *filename, int mode);
        let dfd             = self.get_arg(0);
        let filename_ptr    = self.get_arg(1);
        let mode            = self.get_arg(2);
        let path         = self.get_string(filename_ptr);
        
        let fd = self.filesystem.open(&path, libc::O_RDONLY);

        if self.debug {
            self.debug_print(format!("sys_faccessat {:x} {} {} = {}", dfd, path, mode, fd));
        }
        
        self.set_return_val(fd as u64);
    }

    pub fn sys_readlinkat(&mut self) {
        // sys_readlinkat(int dfd, const char __user *path, char __user *buf, int bufsiz);
        let dfd      = self.get_arg(0);
        let path_ptr = self.get_arg(1);
        let buf      = self.get_arg(2);
        let buf_size = self.get_arg(3);

        let path = self.get_string(path_ptr);
        let fullpath = format!("{}/{}", self.rootfs, path);

        if path == "/proc/self/exe" {
            // todo!();
            let mut path = self.elf_path.clone();
            let mut size    = self.elf_path.len() as u64;

            let mut path_buf = Vec::new();

            path_buf.extend_from_slice(&path.as_bytes());
            path_buf.push(0);

            self.debug_print(format!("readlink at {} = {}", path, (self.elf_path.len())));

            self.mem_write(buf, &path_buf);
            self.set_return_val((path_buf.len()-1) as u64);
        }
        
        else if path.contains("/proc/self/fd") {
            self.mem_write(buf, &path.as_bytes());
            self.set_return_val((path.len()-1) as u64);
        }

        else {
            let mut data = vec![0u8; buf_size as usize];
            self.set_return_val(0);
        }
    }

    pub fn sys_read(&mut self) {
        // sys_read(unsigned int fd, char __user *buf, size_t count);
        let fd = self.get_arg(0);
        let buf = self.get_arg(1);
        let count = self.get_arg(2);

        let mut data = vec![0u8; count as usize];
        let res = self.filesystem.read(fd as i32, &mut data);
        
        match res.ok() {
            Some(v) => {
                if self.debug == true {
                    println!("sys_read {} {:x} {:x} = {:x}",fd, buf, count, v);
                }
                self.mem_write(buf, &data);
                self.set_return_val(v as u64);
            },
            None => {
                // println!("sys_read fail {} {:x} {:x} = {:x}",fd, buf, count, -1);
                self.set_return_val(0xffff_ffff);
            }
        }
    }

    pub fn sys_pread64(&mut self) {
        // sys_pread64(unsigned int fd, char __user *buf, size_t count, loff_t pos);
        let fd = self.get_arg(0);
        let read_buf = self.get_arg(1);
        let read_count = self.get_arg(2);
        let read_pos = self.get_arg(3);

        let mut data = vec![0u8; read_count as usize];
        if read_pos != 0 {
            let res = self.filesystem.pread(fd as i32, &mut data, read_pos).unwrap();
            // println!("sys_pread {} {:x} {:x} {} = {:x}", fd, read_buf, read_count, read_pos, res);
            self.set_return_val(res as u64);
        }

        else {
            self.sys_read();
        }
    }

    pub fn sys_getuid(&mut self) {
        self.set_return_val(uid as u64);
    }

    pub fn sys_write(&mut self) {
        // sys_write(unsigned int fd, const char __user *buf, size_t count);
        let fd = self.get_arg(0);
        let buf_ptr = self.get_arg(1);
        let count  = self.get_arg(2);

        let mut data = self.mem_read_as_vec(buf_ptr, count as usize).unwrap();
        let res = self.filesystem.write(fd as i32, &mut data);
        
        match res.ok() {
            Some(v) => {
                self.set_return_val(v as u64);
            },
            None => {
                self.set_return_val(0xffff_ffff);
            }
        }
    }
    
}