use crate::core::android::fs::fserrors;
use crate::core::rudroid::Emulator;

impl<D> Emulator<D> {
    pub fn sys_sched_getscheduler(&mut self) {
        // sys_sched_getscheduler(pid_t pid);
        let pid = self.get_arg(0) as u32;
        let pid_s = pid as i32;

        if pid_s < 0 {
            self.set_return_val(fserrors::EINVAL as u64);
            return;
        }

        let res = unsafe { libc::sched_getscheduler(pid_s) };
        self.set_return_val(res as u64);
        self.debug_print(format!("sys_sched_getscheduler: {:} = {:}", pid, res));
    }
}