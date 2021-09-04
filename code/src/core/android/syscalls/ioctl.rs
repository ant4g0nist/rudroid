use libc::ioctl;
use crate::core::rudroid::Emulator;

impl<D> Emulator<D> {
    pub fn sys_ioctl(&mut self) {
        // sys_ioctl(unsigned int fd, unsigned int cmd, unsigned long arg);
        let fd = self.get_arg(0);
        let cmd = self.get_arg(1);
        let arg = self.get_arg(2);

        unsafe  {
            let res = ioctl(fd as i32, cmd, arg);
            // println!("res: {}", res);
        }
        self.set_return_val(0xffff_ffff)
    }
}