use std::{
    path,
    fs::File,
    fs::OpenOptions,
    io::prelude::*,
    io::{self, Write, Read},
    os::unix::io::FromRawFd,
    os::unix::io::AsRawFd,
    os::unix::io::RawFd,
    os::unix::io::IntoRawFd,
    os::unix::fs::OpenOptionsExt,
    os::unix::fs,
};

use std::collections::HashMap;
pub(crate) mod fserrors;

pub(crate) const MAX_FDS: i32 = 1024;

pub const O_RDONLY  :i32   = 0;
pub const O_WRONLY  :i32   = 1;
pub const O_RDWR    :i32   = 2;
pub const O_APPEND  :i32   = 8;
pub const O_CREAT   :i32   = 512;

pub const O_ACCMODE :i32   = O_CREAT|O_RDWR|O_WRONLY|O_RDONLY;

#[derive(Clone, Debug)]
pub struct oFile {
    pub path    : String,
    // pub file    : File,
    pub fd      : RawFd,
    pub flags   : i32,
    pub seek    : usize,
    pub shared  : bool,
}


#[derive(Debug)]
pub struct FsScheme
{
    pub rootfs        : String,
    pub open_files    : HashMap<RawFd, oFile>
}

impl oFile {
    pub fn new(path: &str, fd: RawFd, flags: i32) -> oFile {
        oFile {
            path   : String::from(path),
            // file  : file,
            fd     : fd,
            flags  : flags,
            seek   : 0,
            shared : false,
        }
    }

    pub fn set_shared(&mut self, shared: bool) {
        self.shared = shared;
    }
}

impl FsScheme {
    pub fn new(rootfs: String) -> FsScheme {
        FsScheme {
            rootfs      : rootfs.clone(),
            open_files  : HashMap::new(),
        }
    }

    pub fn check_for_traversal(&self, path: &str) {
        //! FIX ME. i'm just checking for ../ == ParentDir
        let path = path::Path::new(path);

        for component in path.components().into_iter() {
           match component {
            path::Component::ParentDir => {
                panic!("trying to do dir traversal??/");
            },
            _ => {
                }
           }
        }
    }

    pub fn open(&mut self, path: &str, flags: i32) -> RawFd {

        self.check_for_traversal(path);

        let full_path = self.change_path_if_special(path);

        let file = OpenOptions::new().custom_flags(flags as i32)
                            .create(flags & libc::O_ACCMODE == libc::O_CREAT)
                            .read((flags & libc::O_ACCMODE == libc::O_RDONLY) || (flags & libc::O_ACCMODE == libc::O_RDWR))
                            .write((flags & libc::O_ACCMODE == libc::O_WRONLY) || (flags & libc::O_ACCMODE == libc::O_RDWR))
                            .open(&full_path);

        match file.ok() {
            Some(v) => {
                let fd = v.into_raw_fd();
                let ofile = oFile::new(path, fd, flags);
                self.open_files.insert(fd, ofile);
                fd
            },
            None => {
                -1
            }
        }
    }

    pub fn openat(&mut self, dirfd: RawFd, path: &str, flags: i32, mode: i32) -> RawFd {
        if dirfd >0 && dirfd < 256 {
            unsafe {
                self.check_for_traversal(path);
                let fd = libc::openat(dirfd, path.as_ptr() as *mut libc::c_char, flags);
                let ofile = oFile::new(path, fd, flags);
                self.open_files.insert(fd, ofile);
    
                fd
            }
        }
        else {
            self.open(&path, flags)
        }
    }

    pub fn write(&mut self, fd: RawFd, buffer: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let res = libc::write(fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len());
            Ok(res as usize)
        }
    }

    pub fn read(&self, fd: RawFd, buffer: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let res = libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len());
            Ok(res as usize)
        }
    }

    pub fn pread(&self, fd: RawFd, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        unsafe 
         {
            let orig_off = libc::lseek(fd, 0, libc::SEEK_CUR);
            libc::lseek(fd, offset as i64, libc::SEEK_SET);
            let ret = self.read(fd, buf).unwrap();
            libc::lseek(fd, orig_off as i64, libc::SEEK_SET);
            Ok(ret)
        }
    }

    pub fn close(&mut self, fd: i32) {
        unsafe {
            libc::close(fd);
        }
        let file = self.open_files.get(&fd);
        match file {
            Some(v) => {
                self.open_files.remove(&fd);
            },  
            None => { }
        }
    }

    pub fn get_local_file(&self, fd: RawFd) -> Option<&oFile> {
        for (f, file) in self.open_files.iter() {
            if *f == fd {
                return Some(file);
            }
        }
        None        
    }

    pub fn set_ofile_shared(&mut self, path: &str, shared: bool) {
        for (f, file) in self.open_files.iter_mut() {
            if file.path == path {
                file.set_shared(shared);
            }
        }
    }

    pub fn change_path_if_special(&self, path: &str) -> String {
        if self.is_driver_io(path) {
            return String::from(path);
        }

        let res = match path {
            "/sys/fs/selinux/null" => {
                String::from("/dev/null")
            },
            _ => {
                // let res = format!("{}/{}", self.rootfs, path);
                let mut res = String::new();
                res.push_str(&self.rootfs);
                res.push('/');
                res.push_str(path);
                res
            }
        };
        // println!("{}", res);
        res
    }

    pub fn is_driver_io(&self, path: &str) -> bool {
        match path {
            "/dev/urandom" | "/dev/random"| "/dev/srandom" | "/proc/self/exe" | "/dev/null" => {
                true
            },
            path if path.contains("/proc/self/fd") => {
                true
            },
            _ => {
                false
            }
        }
    }

    pub fn get_path(&mut self, fd: RawFd) -> Result<String, fserrors::Error> {
        for (f, file) in self.open_files.iter() {
            if fd == *f {
                return Ok(file.path.clone());
            }
        }
        Err(fserrors::Error::new(fserrors::ENOENT))
    }
}