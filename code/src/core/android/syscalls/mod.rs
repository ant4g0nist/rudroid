mod syscalls;
mod unistd;
mod mman;
mod futex;
mod sched;
mod signal;
mod prctl;
mod random;
mod fnctl;
mod stat;
mod ioctl;

use crate::{core::{rudroid::Emulator, unicorn::arch::arm64::RegisterARM64}, utilities};

pub fn get_syscall(uc: &mut Emulator<i64>) -> syscalls::Syscalls {
    // syscall_num = UC_ARM64_REG_X8
    let syscall = uc.reg_read(RegisterARM64::X8 as i32).unwrap();
    unsafe { ::std::mem::transmute(syscall) }
}

pub fn hook_syscall(uc: &mut Emulator<i64>, intno: u32) {
    let pc = uc.reg_read(RegisterARM64::PC as i32).unwrap();
    let syscall = get_syscall(uc);
    uc.syscall(syscall);
}

impl<D> Emulator<D> {
    pub fn syscall(&mut self, syscall: syscalls::Syscalls) {
        if self.debug {
            utilities::draw_line();
            self.debug_print(format!("got syscall: {:?}", syscall));
        }
        
        match syscall {
            
            syscalls::Syscalls::__NR3264_mmap =>
            {
                self.sys_mmap();
            },

            syscalls::Syscalls::__NR_getpid =>
            {
                self.sys_getpid();
            },
            
            syscalls::Syscalls::__NR_set_tid_address => {
                self.sys_set_tid_address();
            },
            
            syscalls::Syscalls::__NR_faccessat => {
                self.sys_faccessat();
            },
            
            syscalls::Syscalls::__NR_futex => {
                self.sys_futex();
            },

            syscalls::Syscalls::__NR_sched_getscheduler => {
                self.sys_sched_getscheduler();
            },
            
            syscalls::Syscalls::__NR_mprotect => {
                self.sys_mprotect();
            },
            syscalls::Syscalls::__NR_sigaltstack => {
                self.sys_sigaltstack();
            },

            //prctl
            syscalls::Syscalls::__NR_prctl => {
                self.prctl();
            },

            //random
            syscalls::Syscalls::__NR_getrandom => {
                self.sys_getrandom();
            },
            
            syscalls::Syscalls::__NR_openat => {
                self.sys_openat();
            },
            syscalls::Syscalls::__NR3264_fcntl => {
                self.sys_fcntl()
            },
            syscalls::Syscalls::__NR_close => {
                self.sys_close();
            },

            syscalls::Syscalls::__NR3264_fstatat => {
                self.sys_fstatat();
            },

            syscalls::Syscalls::__NR_readlinkat => {
                self.sys_readlinkat();
            },

            syscalls::Syscalls::__NR_rt_sigaction => {
                self.sys_rt_sigaction();
            },

            syscalls::Syscalls::__NR3264_fstat => {
                self.sys_fstat();
            },

            syscalls::Syscalls::__NR_read => {
                self.sys_read();
            },
            syscalls::Syscalls::__NR_munmap => {
                self.sys_munmap();
            },

            syscalls::Syscalls::__NR3264_fstatfs => {
                self.sys_fstatfs();
            },

            syscalls::Syscalls::__NR_pread64 => {
                self.sys_pread64()
            },

            syscalls::Syscalls::__NR_rt_sigprocmask => {
                self.sys_rt_sigprocmask();
            },

            syscalls::Syscalls::__NR_clock_gettime => {
                self.empty_syscall_return();
            },
            syscalls::Syscalls::__NR_madvise => {
                self.empty_syscall_return();
            },

            syscalls::Syscalls::__NR_getuid => {
                self.sys_getuid()
            },
            syscalls::Syscalls::__NR_write => {
                self.sys_write()
            },

            syscalls::Syscalls::__NR_ioctl => {
                self.sys_ioctl();
            },

            syscalls::Syscalls::__NR_exit_group => {
                self.sys_exit_group();
            },

            _ => {
                panic!("Syscall {:?} not implemented yet!", syscall);
            }
        }; 
    }

    pub fn empty_syscall_return(&mut self) {
        self.reg_write(RegisterARM64::X0 as i32, 0).unwrap();
    }

    pub fn get_arg(&mut self, num: i32) -> u64 {
        // 'x0', 'x1', 'x2', 'x3', 'x4', 'x5', 'x6', 'x7'
        match num {
            0 => {
                self.reg_read(RegisterARM64::X0 as i32).unwrap()
            },
            1 => {
                self.reg_read(RegisterARM64::X1 as i32).unwrap()
            },
            2 => {
                self.reg_read(RegisterARM64::X2 as i32).unwrap()
            },
            3 => {
                self.reg_read(RegisterARM64::X3 as i32).unwrap()
            },
            4 => {
                self.reg_read(RegisterARM64::X4 as i32).unwrap()
            },
            5 => {
                self.reg_read(RegisterARM64::X5 as i32).unwrap()
            },
            6 => {
                self.reg_read(RegisterARM64::X6 as i32).unwrap()
            },
            7 => {
                self.reg_read(RegisterARM64::X7 as i32).unwrap()
            },
            _ => {
                panic!("i do not support any more arguments :/");
            }
        }
    }

    pub fn set_return_val(&mut self, value: u64) {
        self.reg_write(RegisterARM64::X0 as i32, value).unwrap();
    }

    pub fn get_string(&mut self, addr: u64) -> String {
        let mut addr = addr;
        let mut string = String::new();

        loop {
            let c = self.mem_read_as_vec(addr, 1).unwrap();
            let c = char::from(c[0]);
            match c {
                '\x20'..='\x7e' => {
                    string.push(c)
                },
                _ => {
                    break;
                }
            };
            addr+=1;
        }
        string
    }    
}