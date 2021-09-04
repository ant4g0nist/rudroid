use std::{io::Read, fs::File};
use crate::core::rudroid::Emulator;

impl<D> Emulator<D> {
    pub fn sys_getrandom(&mut self) {
        // sys_getrandom(char __user *buf, size_t count, unsigned int flags);
        let buf_ptr = self.get_arg(0);
        let count   = self.get_arg(1);
        let flags   = self.get_arg(2);

        let mut f = File::open("/dev/urandom").unwrap();
        let mut buf = vec![0; count as usize];
        f.read_exact(&mut buf).unwrap();

        if self.debug {
            self.debug_print(format!("sys_getrandom {:x} {} {}", buf_ptr, count, flags));
        }
        
        let res = self.mem_write(buf_ptr, &buf);
        match res.ok() {
            Some(v) => {
                self.set_return_val(count);
            },
            None => {
                self.set_return_val(0xffff_ffff);
            }
        };
    }
}