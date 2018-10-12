#[macro_use]
extern crate log;

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub mod blkio;
pub mod cgroup;
pub mod cpu;
pub mod cpuacct;
pub mod cpuset;
pub mod devices;
pub mod error;
pub mod freezer;
pub mod hierarchies;
pub mod hugetlb;
pub mod memory;
pub mod net_cls;
pub mod net_prio;
pub mod perf_event;
pub mod pid;
pub mod rdma;
pub mod cgroup_builder;

use blkio::BlkIoController;
use cpu::CpuController;
use cpuacct::CpuAcctController;
use cpuset::CpuSetController;
use devices::DevicesController;
use error::*;
use freezer::FreezerController;
use hugetlb::HugeTlbController;
use memory::MemController;
use net_cls::NetClsController;
use net_prio::NetPrioController;
use perf_event::PerfEventController;
use pid::PidController;
use rdma::RdmaController;

pub use cgroup::Cgroup;

/// Contains all the subsystems that are available in this crate.
#[derive(Debug)]
pub enum Subsystem {
    /// Controller for the `Pid` subsystem, see `PidController` for more information.
    Pid(PidController),
    /// Controller for the `Mem` subsystem, see `MemController` for more information.
    Mem(MemController),
    /// Controller for the `CpuSet subsystem, see `CpuSetController` for more information.
    CpuSet(CpuSetController),
    /// Controller for the `CpuAcct` subsystem, see `CpuAcctController` for more information.
    CpuAcct(CpuAcctController),
    /// Controller for the `Cpu` subsystem, see `CpuController` for more information.
    Cpu(CpuController),
    /// Controller for the `Devices` subsystem, see `DevicesController` for more information.
    Devices(DevicesController),
    /// Controller for the `Freezer` subsystem, see `FreezerController` for more information.
    Freezer(FreezerController),
    /// Controller for the `NetCls` subsystem, see `NetClsController` for more information.
    NetCls(NetClsController),
    /// Controller for the `BlkIo` subsystem, see `BlkIoController` for more information.
    BlkIo(BlkIoController),
    /// Controller for the `PerfEvent` subsystem, see `PerfEventController` for more information.
    PerfEvent(PerfEventController),
    /// Controller for the `NetPrio` subsystem, see `NetPrioController` for more information.
    NetPrio(NetPrioController),
    /// Controller for the `HugeTlb` subsystem, see `HugeTlbController` for more information.
    HugeTlb(HugeTlbController),
    /// Controller for the `Rdma` subsystem, see `RdmaController` for more information.
    Rdma(RdmaController),
}

#[doc(hidden)]
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
    pub fn to_string(&self) -> String {
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

mod sealed {
    use super::*;

    pub trait ControllerInternal {
        fn apply(&self, res: &Resources) -> Result<()>;

        // meta stuff
        fn control_type(&self) -> Controllers;
        fn get_path(&self) -> &PathBuf;
        fn get_path_mut(&mut self) -> &mut PathBuf;
        fn get_base(&self) -> &PathBuf;

        fn verify_path(&self) -> Result<()> {
            if self.get_path().starts_with(self.get_base()) {
                Ok(())
            } else {
                Err(Error::new(ErrorKind::InvalidPath))
            }
        }

        fn open_path(&self, p: &str, w: bool) -> Result<File> {
            let mut path = self.get_path().clone();
            path.push(p);

            self.verify_path()?;

            if w {
                match File::create(&path) {
                    Err(e) => return Err(Error::with_cause(ErrorKind::WriteFailed, e)),
                    Ok(file) => return Ok(file),
                }
            } else {
                match File::open(&path) {
                    Err(e) => return Err(Error::with_cause(ErrorKind::ReadFailed, e)),
                    Ok(file) => return Ok(file),
                }
            }
        }

        #[doc(hidden)]
        fn path_exists(&self, p: &str) -> bool {
            if let Err(_) = self.verify_path() {
                return false;
            }

            std::path::Path::new(p).exists()
        }

    }
}

pub(crate) use sealed::ControllerInternal;

/// A Controller is a subsystem attached to the control group.
///
/// Implementors are able to control certain aspects of a control group.
pub trait Controller {
    #[doc(hidden)]
    fn control_type(&self) -> Controllers;

    /// The file system path to the controller.
    fn path(&self) -> &Path;

    /// Apply a set of resources to the Controller, invoking its internal functions to pass the
    /// kernel the information.
    fn apply(&self, res: &Resources) -> Result<()>;

    /// Create this controller
    fn create(&self);

    /// Does this controller already exist?
    fn exists(&self) -> bool;

    /// Delete the controller.
    fn delete(&self);

    /// Attach a task to this controller.
    fn add_task(&self, pid: &CgroupPid) -> Result<()>;

    /// Get the list of tasks that this controller has.
    fn tasks(&self) -> Vec<CgroupPid>;
}

impl<T> Controller for T where T: ControllerInternal {
    fn control_type(&self) -> Controllers {
        ControllerInternal::control_type(self)
    }

    fn path(&self) -> &Path {
        self.get_path()
    }

    /// Apply a set of resources to the Controller, invoking its internal functions to pass the
    /// kernel the information.
    fn apply(&self, res: &Resources) -> Result<()> {
        ControllerInternal::apply(self, res)
    }

    /// Create this controller
    fn create(&self) {
        self.verify_path().expect("path should be valid");

        match ::std::fs::create_dir(self.get_path()) {
            Ok(_) => (),
            Err(e) => warn!("error create_dir {:?}", e),
        }
    }

    /// Does this controller already exist?
    fn exists(&self) -> bool {
        self.get_path().exists()
    }

    /// Delete the controller.
    fn delete(&self) {
        if self.get_path().exists() {
            let _ = ::std::fs::remove_dir(self.get_path());
        }
    }

    /// Attach a task to this controller.
    fn add_task(&self, pid: &CgroupPid) -> Result<()> {
        self.open_path("tasks", true).and_then(|mut file| {
            file.write_all(pid.pid.to_string().as_ref())
                .map_err(|e| Error::with_cause(ErrorKind::WriteFailed, e))
        })
    }

    /// Get the list of tasks that this controller has.
    fn tasks(&self) -> Vec<CgroupPid> {
        self.open_path("tasks", false)
            .and_then(|file| {
                let bf = BufReader::new(file);
                let mut v = Vec::new();
                for line in bf.lines() {
                    if let Ok(line) = line {
                        let n = line.trim().parse().unwrap_or(0u64);
                        v.push(n);
                    }
                }
                Ok(v.into_iter().map(CgroupPid::from).collect())
            }).unwrap_or(vec![])
    }
}

#[doc(hidden)]
pub trait ControllIdentifier {
    fn controller_type() -> Controllers;
}

/// Control group hierarchy (right now, only V1 is supported, but in the future Unified will be
/// implemented as well).
pub trait Hierarchy {
    /// Returns what subsystems are supported by the hierarchy.
    fn subsystems(&self) -> Vec<Subsystem>;

    /// Returns the root directory of the hierarchy.
    fn root(&self) -> PathBuf;

    /// Return a handle to the root control group in the hierarchy.
    fn root_control_group(&self) -> Cgroup;

    /// Checks whether a certain subsystem is supported in the hierarchy.
    ///
    /// This is an internal function and should not be used.
    #[doc(hidden)]
    fn check_support(&self, sub: Controllers) -> bool;
}

/// Resource limits for the memory subsystem.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct MemoryResources {
    /// Whether values should be applied to the controller.
    pub update_values: bool,
    /// How much memory (in bytes) can the kernel consume.
    pub kernel_memory_limit: u64,
    /// Upper limit of memory usage of the control group's tasks.
    pub memory_hard_limit: u64,
    /// How much memory the tasks in the control group can use when the system is under memory
    /// pressure.
    pub memory_soft_limit: u64,
    /// How much of the kernel's memory (in bytes) can be used for TCP-related buffers.
    pub kernel_tcp_memory_limit: u64,
    /// How much memory and swap together can the tasks in the control group use.
    pub memory_swap_limit: u64,
    /// Controls the tendency of the kernel to swap out parts of the address space of the tasks to
    /// disk. Lower value implies less likely.
    ///
    /// Note, however, that a value of zero does not mean the process is never swapped out. Use the
    /// traditional `mlock(2)` system call for that purpose.
    pub swappiness: u64,
}

/// Resources limits on the number of processes.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PidResources {
    /// Whether values should be applied to the controller.
    pub update_values: bool,
    /// The maximum number of processes that can exist in the control group.
    ///
    /// Note that attaching processes to the control group will still succeed _even_ if the limit
    /// would be violated, however forks/clones inside the control group will have with `EAGAIN` if
    /// they would violate the limit set here.
    pub maximum_number_of_processes: pid::PidMax,
}

/// Resources limits about how the tasks can use the CPU.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct CpuResources {
    /// Whether values should be applied to the controller.
    pub update_values: bool,
    // cpuset
    /// A comma-separated list of CPU IDs where the task in the control group can run. Dashes
    /// between numbers indicate ranges.
    pub cpus: String,
    /// Same syntax as the `cpus` field of this structure, but applies to memory nodes instead of
    /// processors.
    pub mems: String,
    // cpu
    /// Weight of how much of the total CPU time should this control group get. Note that this is
    /// hierarchical, so this is weighted against the siblings of this control group.
    pub shares: u64,
    /// In one `period`, how much can the tasks run in nanoseconds.
    pub quota: i64,
    /// Period of time in nanoseconds.
    pub period: u64,
    /// This is currently a no-operation.
    pub realtime_runtime: i64,
    /// This is currently a no-operation.
    pub realtime_period: u64,
}

/// A device resource that can be allowed or denied access to.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DeviceResource {
    /// If true, access to the device is allowed, otherwise it's denied.
    pub allow: bool,
    /// `'c'` for character device, `'b'` for block device; or `'a'` for all devices.
    pub devtype: ::devices::DeviceType,
    /// The major number of the device.
    pub major: i64,
    /// The minor number of the device.
    pub minor: i64,
    /// Sequence of `'r'`, `'w'` or `'m'`, each denoting read, write or mknod permissions.
    pub access: Vec<::devices::DevicePermissions>,
}

/// Limit the usage of devices for the control group's tasks.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DeviceResources {
    /// Whether values should be applied to the controller.
    pub update_values: bool,
    /// For each device in the list, the limits in the structure are applied.
    pub devices: Vec<DeviceResource>,
}

/// Assigned priority for a network device.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct NetworkPriority {
    /// The name (as visible in `ifconfig`) of the interface.
    pub name: String,
    /// Assigned priority.
    pub priority: u64,
}

/// Collections of limits and tags that can be imposed on packets emitted by the tasks in the
/// control group.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct NetworkResources {
    /// Whether values should be applied to the controller.
    pub update_values: bool,
    /// The networking class identifier to attach to the packets.
    ///
    /// This can then later be used in iptables and such to have special rules.
    pub class_id: u64,
    /// Priority of the egress traffic for each interface.
    pub priorities: Vec<NetworkPriority>,
}

/// A hugepage type and its consumption limit for the control group.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct HugePageResource {
    /// The size of the hugepage, i.e. `2MB`, `1GB`, etc.
    pub size: String,
    /// The amount of bytes (of memory consumed by the tasks) that are allowed to be backed by
    /// hugepages.
    pub limit: u64,
}

/// Provides the ability to set consumption limit on each type of hugepages.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct HugePageResources {
    /// Whether values should be applied to the controller.
    pub update_values: bool,
    /// Set a limit of consumption for each hugepages type.
    pub limits: Vec<HugePageResource>,
}

/// Weight for a particular block device.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BlkIoDeviceResource {
    /// The major number of the device.
    pub major: u64,
    /// The minor number of the device.
    pub minor: u64,
    /// The weight of the device against the descendant nodes.
    pub weight: u16,
    /// The weight of the device against the sibling nodes.
    pub leaf_weight: u16,
}

/// Provides the ability to throttle a device (both byte/sec, and IO op/s)
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BlkIoDeviceThrottleResource {
    /// The major number of the device.
    pub major: u64,
    /// The minor number of the device.
    pub minor: u64,
    /// The rate.
    pub rate: u64,
}

/// General block I/O resource limits.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BlkIoResources {
    /// Whether values should be applied to the controller.
    pub update_values: bool,
    /// The weight of the control group against descendant nodes.
    pub weight: u16,
    /// The weight of the control group against sibling nodes.
    pub leaf_weight: u16,
    /// For each device, a separate weight (both normal and leaf) can be provided.
    pub weight_device: Vec<BlkIoDeviceResource>,
    /// Throttled read bytes/second can be provided for each device.
    pub throttle_read_bps_device: Vec<BlkIoDeviceThrottleResource>,
    /// Throttled read IO operations per second can be provided for each device.
    pub throttle_read_iops_device: Vec<BlkIoDeviceThrottleResource>,
    /// Throttled written bytes/second can be provided for each device.
    pub throttle_write_bps_device: Vec<BlkIoDeviceThrottleResource>,
    /// Throttled write IO operations per second can be provided for each device.
    pub throttle_write_iops_device: Vec<BlkIoDeviceThrottleResource>,
}

/// The resource limits and constraints that will be set on the control group.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Resources {
    /// Memory usage related limits.
    pub memory: MemoryResources,
    /// Process identifier related limits.
    pub pid: PidResources,
    /// CPU related limits.
    pub cpu: CpuResources,
    /// Device related limits.
    pub devices: DeviceResources,
    /// Network related tags and limits.
    pub network: NetworkResources,
    /// Hugepages consumption related limits.
    pub hugepages: HugePageResources,
    /// Block device I/O related limits.
    pub blkio: BlkIoResources,
}

/// A structure representing a `pid`. Currently implementations exist for `u64` and
/// `std::process::Child`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CgroupPid {
    /// The process identifier
    pub pid: u64,
}

impl From<u64> for CgroupPid {
    fn from(u: u64) -> CgroupPid {
        CgroupPid { pid: u }
    }
}

impl<'a> From<&'a std::process::Child> for CgroupPid {
    fn from(u: &std::process::Child) -> CgroupPid {
        CgroupPid { pid: u.id() as u64 }
    }
}

impl Subsystem {
    fn enter(self, path: &String) -> Self {
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

    fn to_controller(&self) -> &dyn Controller {
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
