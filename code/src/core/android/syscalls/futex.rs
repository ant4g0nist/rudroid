use std::thread;
use crate::core::rudroid::Emulator;

const FUTEX_WAIT            : u64 = 0;
const FUTEX_WAKE            : u64 = 1;
const FUTEX_FD              : u64 = 2;
const FUTEX_REQUEUE         : u64 = 3;
const FUTEX_CMP_REQUEUE     : u64 = 4;
const FUTEX_WAKE_OP         : u64 = 5;
const FUTEX_LOCK_PI         : u64 = 6;
const FUTEX_UNLOCK_PI       : u64 = 7;
const FUTEX_TRYLOCK_PI      : u64 = 8;
const FUTEX_WAIT_BITSET     : u64 = 9;
const FUTEX_WAKE_BITSET     : u64 = 10;
const FUTEX_WAIT_REQUEUE_PI : u64 = 11;
const FUTEX_CMP_REQUEUE_PI  : u64 = 12;
const FUTEX_PRIVATE_FLAG    : u64 = 128;


impl<D> Emulator<D> {
    pub fn sys_futex(&mut self) {
        // sys_futex(u32 __user *uaddr, int op, u32 val,
        //     struct timespec __user *utime, u32 __user *uaddr2,
        //     u32 val3);
        
        let uaddr   = self.get_arg(0);
        let op      = self.get_arg(1);
        let val     = self.get_arg(2);
        let mut old_val = self.mem_read_as_vec(uaddr, 4).unwrap();
        let mut old_val     = self.unpack(&old_val);
        
        println!("sys_futex: {:x} {} {} op & 0x7f: {}", uaddr, op, val, op & 0x7f);

        match op & 0x7f {
            FUTEX_WAIT => {
                println!("sys_futex: futex_wait {:x} {} {} op & 0x7f: {}", uaddr, op, val, op & 0x7f);
                if old_val == val {
                    if self.debug {
                        println!("futex old = {:x} val = {:x}", old_val, val);
                    }
                    self.emu_stop();
                    self.set_return_val(0);
                }
                
                thread::yield_now();
                let timeout = self.get_arg(3);
                let mytype = val & 0xc000;
                let shared = val & 0x2000;
                // uaddr.setInt(0, mytype | shared);
                let res = mytype | shared;
                self.mem_write(uaddr, &self.pack(res));
                self.set_return_val(0);
            },

            FUTEX_WAKE => {
                self.set_return_val(0);
            },
            _ => {

            }
        }        
    }
}