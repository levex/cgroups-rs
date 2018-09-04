extern crate cgroups;

use cgroups::{Cgroup, CgroupError};
use cgroups::cpuset::CpuSetController;

#[test]
fn test_cpuset_memory_pressure_root_cg() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_cpuset_memory_pressure_root_cg"));
    {
        let cpuset: &CpuSetController = cg.controller_of().unwrap();

        // This is not a root control group, so it should fail via InvalidOperation.
        let res = cpuset.set_enable_memory_pressure(true);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), CgroupError::InvalidOperation);
    }
    cg.delete();
}
