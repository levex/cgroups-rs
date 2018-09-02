//! This module contains the implementation of the `cpu` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/scheduler/sched-design-CFS.txt](https://www.kernel.org/doc/Documentation/scheduler/sched-design-CFS.txt)
//!  paragraph 7 ("GROUP SCHEDULER EXTENSIONS TO CFS").
use std::path::PathBuf;
use std::io::{Read, Write};

use {CgroupError, CpuResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};

/// A controller that allows controlling the `cpu` subsystem of a Cgroup.
/// 
/// In essence, it allows gathering information about how much the tasks inside the control group
/// are using the CPU and creating rules that limit their usage. Note that this crate does not yet
/// support managing realtime tasks.
#[derive(Debug, Clone)]
pub struct CpuController{
    base: PathBuf,
    path: PathBuf,
}

/// The current state of the control group and its processes.
#[derive(Debug)]
pub struct Cpu {
    /// Reports CPU time statistics.
    ///
    /// Corresponds the `cpu.stat` file in `cpu` control group.
    pub stat: String,
}

impl Controller for CpuController {
    fn control_type(self: &Self) -> Controllers { Controllers::Cpu}
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let res: &CpuResources = &res.cpu;

        if res.update_values {
            /* apply pid_max */
            let _ = self.set_shares(res.shares);
            let _ = self.set_cfs_period(res.period);
            let _ = self.set_cfs_quota(res.quota as u64);
            /* TODO: rt properties (CONFIG_RT_GROUP_SCHED) are not yet supported */
        }
    }
}

impl ControllIdentifier for CpuController {
    fn controller_type() -> Controllers {
        Controllers::Cpu
    }
}

impl<'a> From<&'a Subsystem> for &'a CpuController {
    fn from(sub: &'a Subsystem) -> &'a CpuController {
        unsafe {
            match sub {
                Subsystem::Cpu(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
            }
        }
    }
}

impl CpuController {
    /// Contructs a new `CpuController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Returns CPU time statistics based on the processes in the control group.
    pub fn cpu(self: &Self) -> Cpu {
        Cpu {
            stat: self.open_path("cpu.stat", false).and_then(|mut file| {
                let mut s = String::new();
                let res = file.read_to_string(&mut s);
                match res {
                    Ok(_) => Ok(s),
                    Err(e) => Err(CgroupError::ReadError(e)),
                }
            }).unwrap_or("".to_string()),
        }
    }

    /// Configures the CPU bandwidth (in relative relation to other control groups and this control
    /// group's parent).
    /// 
    /// For example, setting control group `A`'s `shares` to `100`, and control group `B`'s
    /// `shares` to `200` ensures that control group `B` receives twice as much as CPU bandwidth.
    /// (Assuming both `A` and `B` are of the same parent)
    pub fn set_shares(self: &Self, shares: u64) -> Result<(), CgroupError> {
        self.open_path("cpu.shares", true).and_then(|mut file| {
            file.write_all(shares.to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Specify a period (when using the CFS scheduler) of time in microseconds for how often this
    /// control group's access to the CPU should be reallocated.
    pub fn set_cfs_period(self: &Self, us: u64) -> Result<(), CgroupError> {
        self.open_path("cpu.cfs_period_us", true).and_then(|mut file| {
            file.write_all(us.to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Specify a quota (when using the CFS scheduler) of time in microseconds for which all tasks
    /// in this control group can run during one period (see: `set_cfs_period()`).
    pub fn set_cfs_quota(self: &Self, us: u64) -> Result<(), CgroupError> {
        self.open_path("cpu.cfs_quota_us", true).and_then(|mut file| {
            file.write_all(us.to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }
}
