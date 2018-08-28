use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

pub mod hierarchies;
pub mod pid;
pub mod memory;
pub mod cpuset;
pub mod cpuacct;
pub mod cpu;
pub mod devices;
pub mod cgroup;
pub mod freezer;
pub mod net_cls;
pub mod blkio;
pub mod perf_event;
pub mod net_prio;
pub mod hugetlb;
pub mod rdma;

use pid::PidController;
use memory::MemController;
use cpuset::CpuSetController;
use cpuacct::CpuAcctController;
use cpu::CpuController;
use freezer::FreezerController;
use devices::DevicesController;
use net_cls::NetClsController;
use blkio::BlkIoController;
use perf_event::PerfEventController;
use net_prio::NetPrioController;
use hugetlb::HugeTlbController;
use rdma::RdmaController;

#[derive(Debug)]
pub enum Subsystem {
    Pid(PidController),
    Mem(MemController),
    CpuSet(CpuSetController),
    CpuAcct(CpuAcctController),
    Cpu(CpuController),
    Devices(DevicesController),
    Freezer(FreezerController),
    NetCls(NetClsController),
    BlkIo(BlkIoController),
    PerfEvent(PerfEventController),
    NetPrio(NetPrioController),
    HugeTlb(HugeTlbController),
    Rdma(RdmaController),
}

#[derive(Eq, PartialEq, Debug)]
pub enum Controllers {
    Pids,
    Mem,
    CpuSet,
    CpuAcct,
    Cpu,
    Devices,
    Freezer,
    NetCls,
    BlkIo,
    PerfEvent,
    NetPrio,
    HugeTlb,
    Rdma,
}

impl Controllers {
    pub fn to_string(self: &Self) -> String {
        match self {
            Controllers::Pids => return "pids".to_string(),
            Controllers::Mem => return "memory".to_string(),
            Controllers::CpuSet => return "cpuset".to_string(),
            Controllers::CpuAcct => return "cpuacct".to_string(),
            Controllers::Cpu => return "cpu".to_string(),
            Controllers::Devices => return "devices".to_string(),
            Controllers::Freezer => return "freezer".to_string(),
            Controllers::NetCls => return "net_cls".to_string(),
            Controllers::BlkIo => return "blkio".to_string(),
            Controllers::PerfEvent => return "perf_event".to_string(),
            Controllers::NetPrio => return "net_prio".to_string(),
            Controllers::HugeTlb => return "hugetlb".to_string(),
            Controllers::Rdma => return "rdma".to_string(),
        }
    }
}

pub trait Controller {
    /* actual API */
    fn apply(self: &Self, res: &Resources);

    /* meta stuff */
    fn control_type(self: &Self) -> Controllers;
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf;
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf;
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf;

    fn verify_path(self: &Self) -> bool {
        self.get_path().starts_with(self.get_base())
    }

    fn create(self: &Self) {
        if self.verify_path() {
            match ::std::fs::create_dir(self.get_path()) {
                Ok(_) => (),
                Err(e) => println!("error create_dir {:?}", e),
            }
        }
    }

    fn exists(self: &Self) -> bool {
        self.get_path().exists()
    }

    fn delete(self: &Self) {
        if self.get_path().exists() {
            let _ = ::std::fs::remove_dir(self.get_path());
        }
    }

    fn open_path(self: &Self, p: &str, w: bool) -> Option<File> {
        let mut path = self.get_path().clone();
        path.push(p);

        if !self.verify_path() {
            return None;
        }

        if w {
            match File::create(&path) {
                Err(_) => return None,
                Ok(file) => return Some(file),
            }
        } else {
            match File::open(&path) {
                Err(_) => return None,
                Ok(file) => return Some(file),
            }
        }
    }

    fn add_task(self: &Self, pid: &CgroupPid) {
        self.open_path("tasks", true).and_then(|mut file| {
            file.write_all(pid.pid.to_string().as_ref()).ok()
        });
    }
}

pub trait ControllIdentifier {
    fn controller_type() -> Controllers;
}

pub trait Hierarchy {
    fn subsystems(self: &Self) -> Vec<Subsystem>;
    fn can_create_cgroup(self: &Self) -> bool;
    fn root(self: &Self) -> PathBuf;
    fn check_support(self: &Self, sub: Controllers) -> bool;
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct MemoryResources {
    pub update_values: bool,
    pub kernel_memory_limit: u64,
    pub memory_hard_limit: u64,
    pub memory_soft_limit: u64,
    pub kernel_tcp_memory_limit: u64,
    pub memory_swap_limit: u64,
    pub swappiness: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PidResources {
    pub update_values: bool,
    pub maximum_number_of_processes: pid::PidMax,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct CpuResources {
    pub update_values: bool,
    /* cpuset */
    pub cpus: String,
    pub mems: String,
    /* cpu */
    pub shares: u64,
    pub quota: i64,
    pub period: u64,
    pub realtime_runtime: i64,
    pub realtime_period: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DeviceResource {
    pub allow: bool,
    pub devtype: String,
    pub major: u64,
    pub minor: u64,
    pub access: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DeviceResources {
    pub update_values: bool,
    pub devices: Vec<DeviceResource>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct NetworkPriority {
    pub name: String,
    pub priority: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct NetworkResources {
    pub update_values: bool,
    pub class_id: u64,
    pub priorities: Vec<NetworkPriority>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct HugePageResource {
    pub size: String,
    pub limit: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct HugePageResources {
    pub update_values: bool,
    pub limits: Vec<HugePageResource>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BlkIoDeviceResource {
    pub major: u64,
    pub minor: u64,
    pub weight: u16,
    pub leaf_weight: u16,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BlkIoDeviceThrottleResource {
    pub major: u64,
    pub minor: u64,
    pub rate: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BlkIoResources {
    pub update_values: bool,
    pub weight: u16,
    pub leaf_weight: u16,
    pub weight_device: Vec<BlkIoDeviceResource>,
    pub throttle_read_bps_device: Vec<BlkIoDeviceThrottleResource>,
    pub throttle_read_iops_device: Vec<BlkIoDeviceThrottleResource>,
    pub throttle_write_bps_device: Vec<BlkIoDeviceThrottleResource>,
    pub throttle_write_iops_device: Vec<BlkIoDeviceThrottleResource>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Resources {
    pub memory: MemoryResources,
    pub pid: PidResources,
    pub cpu: CpuResources,
    pub devices: DeviceResources,
    pub network: NetworkResources,
    pub hugepages: HugePageResources,
    pub blkio: BlkIoResources,
}

pub struct CgroupPid {
    pub pid: u64,
}

impl From<u64> for CgroupPid {
    fn from(u: u64) -> CgroupPid {
        CgroupPid {
            pid: u,
        }
    }
}


impl Subsystem {
    fn enter(self: Self, path: &String) -> Self {
        match self {
            Subsystem::Pid(cont) => Subsystem::Pid({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::Mem(cont) => Subsystem::Mem({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::CpuSet(cont) => Subsystem::CpuSet({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::CpuAcct(cont) => Subsystem::CpuAcct({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::Cpu(cont) => Subsystem::Cpu({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::Devices(cont) => Subsystem::Devices({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::Freezer(cont) => Subsystem::Freezer({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::NetCls(cont) => Subsystem::NetCls({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::BlkIo(cont) => Subsystem::BlkIo({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::PerfEvent(cont) => Subsystem::PerfEvent({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::NetPrio(cont) => Subsystem::NetPrio({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::HugeTlb(cont) => Subsystem::HugeTlb({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
            Subsystem::Rdma(cont) => Subsystem::Rdma({
                let mut c = cont.clone();
                c.get_path_mut().push(path);
                c
            }),
        }
    }

    fn to_controller(self: &Self) -> &dyn Controller {
        match self {
            Subsystem::Pid(cont) => cont,
            Subsystem::Mem(cont) => cont,
            Subsystem::CpuSet(cont) => cont,
            Subsystem::CpuAcct(cont) => cont,
            Subsystem::Cpu(cont) => cont,
            Subsystem::Devices(cont) => cont,
            Subsystem::Freezer(cont) => cont,
            Subsystem::NetCls(cont) => cont,
            Subsystem::BlkIo(cont) => cont,
            Subsystem::PerfEvent(cont) => cont,
            Subsystem::NetPrio(cont) => cont,
            Subsystem::HugeTlb(cont) => cont,
            Subsystem::Rdma(cont) => cont,
        }
    }
}


#[cfg(test)]
mod tests {
    use {Resources, PidResources, Hierarchy, Controller, Controllers, Subsystem};
    use pid::{PidMax, PidController};
    use cgroup::Cgroup;

    #[test]
    fn create_and_delete_cgroup() {
        let hier = ::hierarchies::V1::new();
        let cg = Cgroup::new(&hier, String::from("ltest2"), 0);
        {
            let pidcontroller: &PidController = cg.controller_of().unwrap();
            pidcontroller.set_pid_max(PidMax::Value(1337));
            assert_eq!(pidcontroller.get_pid_max(), Some(PidMax::Value(1337)));
        }
        cg.delete();
    }

    #[test]
    fn test_pid_pids_current_is_zero() {
        let hier = ::hierarchies::V1::new();
        let cg = Cgroup::new(&hier, String::from("ltest3"), 0);
        {
            let pidcontroller: &PidController = cg.controller_of().unwrap();
            assert_eq!(pidcontroller.get_pid_current(), 0);
        }
        cg.delete();
    }

    #[test]
    fn test_pid_pids_events_is_zero() {
        let hier = ::hierarchies::V1::new();
        let cg = Cgroup::new(&hier, String::from("ltest4"), 0);
        {
            let pidcontroller: &PidController = cg.controller_of().unwrap();
            assert_eq!(pidcontroller.get_pid_events(), 0);
        }
        cg.delete();
    }

    #[test]
    fn test_setting_resources() {
        let hier = ::hierarchies::V1::new();
        let cg = Cgroup::new(&hier, String::from("ltest5"), 0);
        {
            let res = Resources {
                pid: PidResources {
                    update_values: true,
                    maximum_number_of_processes: PidMax::Value(512),
                },
                ..Default::default()
            };
            cg.apply(&res);

            /* verify */
            let pidcontroller: &PidController = cg.controller_of().unwrap();
            assert_eq!(pidcontroller.get_pid_max(), Some(PidMax::Value(512)));
        }
        cg.delete();
    }

}
