//! Integration tests about the hugetlb subsystem
use cgroups::memory::MemController;
use cgroups::Cgroup;
use cgroups::Controller;

use cgroups::error::ErrorKind::*;
use cgroups::error::*;

#[test]
fn test_disable_oom_killer() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_disable_oom_killer"));
    {
        let mem_controller: &MemController = cg.controller_of().unwrap();

        // before disable
        let m = mem_controller.memory_stat();
        assert_eq!(m.oom_control.oom_kill_disable, false);

        // disable oom killer
        let r = mem_controller.disable_oom_killer();
        assert_eq!(r.is_err(), false);

        // after disable
        let m = mem_controller.memory_stat();
        assert_eq!(m.oom_control.oom_kill_disable, true);
    }
    cg.delete();
}
