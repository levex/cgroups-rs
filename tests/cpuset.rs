use cgroups::cpuset::CpuSetController;
use cgroups::error::ErrorKind;
use cgroups::{Cgroup, CpuResources, Hierarchy, Resources};

#[test]
fn test_cpuset_memory_pressure_root_cg() {
    let h = cgroups::hierarchies::auto();
    let h = Box::new(&*h);
    let cg = Cgroup::new(h, String::from("test_cpuset_memory_pressure_root_cg"));
    {
        let cpuset: &CpuSetController = cg.controller_of().unwrap();

        // This is not a root control group, so it should fail via InvalidOperation.
        let res = cpuset.set_enable_memory_pressure(true);
        assert_eq!(res.unwrap_err().kind(), &ErrorKind::InvalidOperation);
    }
    cg.delete();
}


#[test]
fn test_cpuset_set_cpus() {
    let h = cgroups::hierarchies::auto();
    let h = Box::new(&*h);
    let cg = Cgroup::new(h, String::from("test_cpuset_set_cpus"));
    {
        let cpuset: &CpuSetController = cg.controller_of().unwrap();

        let set = cpuset.cpuset();
        assert_eq!(0, set.cpus.len());

        // 0
        let r = cpuset.set_cpus("0");
        assert_eq!(true, r.is_ok());

        let set = cpuset.cpuset();
        assert_eq!(1, set.cpus.len());
        assert_eq!((0,0), set.cpus[0]);


        // 0-1
        // FIXME need two cores
        let r = cpuset.set_cpus("0-1");
        assert_eq!(true, r.is_ok());

        let set = cpuset.cpuset();
        assert_eq!(1, set.cpus.len());
        assert_eq!((0,1), set.cpus[0]);



    }
    cg.delete();
}