//! Some simple tests covering the builder pattern for control groups.
use cgroups::*;
use cgroups::cpu::*;
use cgroups::devices::*;
use cgroups::pid::*;
use cgroups::memory::*;
use cgroups::net_cls::*;
use cgroups::hugetlb::*;
use cgroups::blkio::*;
use cgroups::cgroup_builder::*;

#[test]
pub fn test_cpu_res_build() {
    let v1 = crate::hierarchies::V1::new();
    let cg: Cgroup = CgroupBuilder::new("test_cpu_res_build", &v1)
        .cpu()
            .shares(85)
            .done()
        .build();

    {
        let cpu: &CpuController = cg.controller_of().unwrap();
        assert!(cpu.shares().is_ok());
        assert_eq!(cpu.shares().unwrap(), 85);
    }

    cg.delete();
}

#[test]
pub fn test_memory_res_build() {
    let v1 = crate::hierarchies::V1::new();
    let cg: Cgroup = CgroupBuilder::new("test_memory_res_build", &v1)
        .memory()
            .kernel_memory_limit(128 * 1024 * 1024)
            .swappiness(70)
            .memory_hard_limit(1024 * 1024 * 1024)
            .done()
        .build();

    {
        let c: &MemController = cg.controller_of().unwrap();
        assert_eq!(c.kmem_stat().limit_in_bytes, 128 * 1024 * 1024);
        assert_eq!(c.memory_stat().swappiness, 70);
        assert_eq!(c.memory_stat().limit_in_bytes, 1024 * 1024 * 1024);
    }

    cg.delete();
}

#[test]
pub fn test_pid_res_build() {
    let v1 = crate::hierarchies::V1::new();
    let cg: Cgroup = CgroupBuilder::new("test_pid_res_build", &v1)
        .pid()
            .maximum_number_of_processes(PidMax::Value(123))
            .done()
        .build();

    {
        let c: &PidController = cg.controller_of().unwrap();
        assert!(c.get_pid_max().is_ok());
        assert_eq!(c.get_pid_max().unwrap(), PidMax::Value(123));
    }

    cg.delete();
}

#[test]
#[ignore] // ignore this test for now, not sure why my kernel doesn't like it
pub fn test_devices_res_build() {
    let v1 = crate::hierarchies::V1::new();
    let cg: Cgroup = CgroupBuilder::new("test_devices_res_build", &v1)
        .devices()
            .device(1, 6, DeviceType::Char, true,
                    vec![DevicePermissions::Read])
            .done()
        .build();

    {
        let c: &DevicesController = cg.controller_of().unwrap();
        assert!(c.allowed_devices().is_ok());
        assert_eq!(c.allowed_devices().unwrap(), vec![
                   DeviceResource {
                       allow: true,
                       devtype: DeviceType::Char,
                       major: 1,
                       minor: 6,
                       access: vec![DevicePermissions::Read],
                   }
        ]);
    }
    cg.delete();
}

#[test]
pub fn test_network_res_build() {
    let v1 = crate::hierarchies::V1::new();
    let cg: Cgroup = CgroupBuilder::new("test_network_res_build", &v1)
        .network()
            .class_id(1337)
            .done()
        .build();

    {
        let c: &NetClsController = cg.controller_of().unwrap();
        assert!(c.get_class().is_ok());
        assert_eq!(c.get_class().unwrap(), 1337);
    }
    cg.delete();
}

#[test]
pub fn test_hugepages_res_build() {
    let v1 = crate::hierarchies::V1::new();
    let cg: Cgroup = CgroupBuilder::new("test_hugepages_res_build", &v1)
        .hugepages()
            .limit("2MB".to_string(), 4 * 2 * 1024 * 1024)
            .done()
        .build();

    {
        let c: &HugeTlbController = cg.controller_of().unwrap();
        assert!(c.limit_in_bytes(&"2MB".to_string()).is_ok());
        assert_eq!(c.limit_in_bytes(&"2MB".to_string()).unwrap(), 4 * 2 * 1024 * 1024);
    }
    cg.delete();
}

#[test]
pub fn test_blkio_res_build() {
    let v1 = crate::hierarchies::V1::new();
    let cg: Cgroup = CgroupBuilder::new("test_blkio_res_build", &v1)
        .blkio()
            .weight(100)
            .done()
        .build();

    {
        let c: &BlkIoController = cg.controller_of().unwrap();
        assert_eq!(c.blkio().weight, 100);
    }
    cg.delete();
}
