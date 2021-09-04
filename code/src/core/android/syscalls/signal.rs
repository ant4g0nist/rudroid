use crate::core::rudroid::Emulator;

impl<D> Emulator<D> {
    pub fn sys_sigaltstack(&mut self) {
        // sys_sigaltstack(const struct sigaltstack __user *uss, struct sigaltstack __user *uoss);
        let ss      = self.get_arg(0);
        let old_ss  = self.get_arg(1);
        self.set_return_val(0);
    }

    pub fn sys_rt_sigaction(&mut self) {
        // sys_rt_sigaction(int, const struct sigaction __user *, struct sigaction __user *, size_t);
        let sig_num = self.get_arg(0);
        let sigaction = self.get_arg(1);        //pointer
        let oldaction =   self.get_arg(2);      //pointer
        
        // sigaction(signum, sigaction, oldaction);
        self.sigaction(sig_num, sigaction, oldaction);
        self.set_return_val(0);
    }   

    pub fn sigaction(&mut self, signum: u64, act: u64, oldact : u64) {
        let mut sig_num = signum;
        let mut prefix = "Unknown";

        self.debug_print(format!("sigaction signum={}, act={:x}, oldact={:x} prefix={}", sig_num, act, oldact, prefix));

        if oldact != 0 {
            let last_act    = self.sigmap.get(&sig_num);
            
            let data = match last_act {
                Some(v) => {
                    last_act.unwrap().clone()
                },
                None => {
                    let res = vec![0u8; 20];
                    res.clone()
                }
            };

            self.write(oldact, &data);
        }

        if act != 0 {
            let mut data = Vec::new();
            for i in 0..5 {
                let addr =  act + 4*i;
                let val = self.mem_read_as_vec(addr, 4).unwrap();
                // let val = self.unpack_32(&val);
                data.extend_from_slice(&val);
            }
            self.sigmap.insert(signum, data);
        }
    }

    pub fn sys_rt_sigprocmask(&mut self) {
        // sys_rt_sigprocmask(int how, sigset_t __user *set, sigset_t __user *oset, size_t sigsetsize);
        let how         = self.get_arg(0); 
        let set         = self.get_arg(1);
        let oset        = self.get_arg(2);
        let sigsetsize  = self.get_arg(3);

        self.set_return_val(0);
    }

    pub fn sys_exit_group(&mut self) {
        // sys_exit_group(int error_code)
        let error_code = self.get_arg(0);
        self.debug_print(format!("sys_exit_group code: {}", error_code));
        self.emu_stop();
        std::process::exit(1);
    }    

}