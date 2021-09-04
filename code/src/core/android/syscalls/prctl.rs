use crate::core::rudroid::Emulator;

const PR_SET_NAME       : u64 = 15;
const BIONIC_PR_SET_VMA : u64 = 0x53564d41;
const PR_SET_PTRACER    : u64 = 0x59616d61;

impl<D> Emulator<D> {
    pub fn prctl(&mut self) {
        // sys_prctl(int option, unsigned long arg2, unsigned long arg3, unsigned long arg4, unsigned long arg5)

        let option = self.get_arg(0);
        let arg2   = self.get_arg(1);

        self.debug_print(format!("prctl option = 0x{:x}, arg2=0x{:x}",option, arg2));
        self.set_return_val(0);
    }
}