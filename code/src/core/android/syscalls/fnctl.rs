use crate::core::rudroid::Emulator;

impl<D> Emulator<D> {
    pub fn sys_openat(&mut self) {
        // sys_openat(int dfd, const char __user *filename, int flags, umode_t mode);
        let dfd = self.get_arg(0);
        let filename_ptr = self.get_arg(1);
        let flags = self.get_arg(2);
        let mode = self.get_arg(3);

        let filename = self.get_string(filename_ptr);
        let fullpath = self.filesystem.change_path_if_special(&filename);
        let path: &str    = &fullpath;
        
        let fd = self.filesystem.openat(dfd as i32, &filename, flags as i32, mode as i32);
        
        // let fd = self.filesystem.open(&filename, flags as i32);
        self.set_return_val(fd as u64);

        if self.debug {
            self.debug_print(format!("sys_openat in: {:x} {} {} {} = {}", dfd, filename, flags, mode, fd));
        }
    }

    pub fn sys_fcntl(&mut self) {
        // sys_fcntl(unsigned int fd, unsigned int cmd, unsigned long arg);
        let fd = self.get_arg(0);
        let cmd = self.get_arg(1);
        let arg = self.get_arg(2);
        let res = unsafe {libc::fcntl(fd as i32, cmd as i32)};

        if self.debug {
            self.debug_print(format!("sys_fcntl in: {:x} {:x} {} = {}", fd, cmd, arg, res));
        }

        self.set_return_val(res as u64);   
    }

    pub fn sys_close(&mut self) {
        let fd = self.get_arg(0);
        self.filesystem.close(fd as i32);

        if self.debug {
            self.debug_print(format!("sys_close in: {:x} = {}", fd, 0));
        }

        self.set_return_val(0);
    }    
}