//! This module contains the implementation of the `cpu` cgroup subsystem.
//!
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/scheduler/sched-design-CFS.txt](https://www.kernel.org/doc/Documentation/scheduler/sched-design-CFS.txt)
//!  paragraph 7 ("GROUP SCHEDULER EXTENSIONS TO CFS").

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::error::{Error, ErrorKind, Result};
use crate::{
    ControllIdentifier, ControllerInternal, Controllers, CpuResources, Resources, Subsystem,
};

/// A controller that allows controlling the `cpu` subsystem of a Cgroup.
///
/// In essence, it allows gathering information about how much the tasks inside the control group
/// are using the CPU and creating rules that limit their usage. Note that this crate does not yet
/// support managing realtime tasks.
#[derive(Debug, Clone)]
pub struct CpuController {
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

impl ControllerInternal for CpuController {
    fn control_type(&self) -> Controllers {
        Controllers::Cpu
    }

    fn get_path(&self) -> &PathBuf {
        &self.path
    }

    fn get_path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }

    fn get_base(&self) -> &PathBuf {
        &self.base
    }

    fn apply(&self, res: &Resources) -> Result<()> {
        // get the resources that apply to this controller
        let res: &CpuResources = &res.cpu;

        if res.update_values {
            // apply pid_max
            let _ = self.set_shares(res.shares);
            if self.shares()? != res.shares as u64 {
                return Err(Error::new(ErrorKind::ApplyFailed));
            }

            let _ = self.set_cfs_period(res.period);
            if self.cfs_period()? != res.period as u64 {
                return Err(Error::new(ErrorKind::ApplyFailed));
            }

            let _ = self.set_cfs_quota(res.quota as u64);
            if self.cfs_quota()? != res.quota as u64 {
                return Err(Error::new(ErrorKind::ApplyFailed));
            }

            // TODO: rt properties (CONFIG_RT_GROUP_SCHED) are not yet supported
        }

        Ok(())
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
                }
            }
        }
    }
}

fn read_u64_from(mut file: File) -> Result<u64> {
    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => string
            .trim()
            .parse()
            .map_err(|e| Error::with_source(ErrorKind::ParseFailed, e)),
        Err(e) => Err(Error::with_source(ErrorKind::ReadFailed, e)),
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
    pub fn cpu(&self) -> Cpu {
        Cpu {
            stat: self
                .open_path("cpu.stat", false)
                .and_then(|mut file| {
                    let mut s = String::new();
                    let res = file.read_to_string(&mut s);
                    match res {
                        Ok(_) => Ok(s),
                        Err(e) => Err(Error::with_source(ErrorKind::ReadFailed, e)),
                    }
                })
                .unwrap_or("".to_string()),
        }
    }

    /// Configures the CPU bandwidth (in relative relation to other control groups and this control
    /// group's parent).
    ///
    /// For example, setting control group `A`'s `shares` to `100`, and control group `B`'s
    /// `shares` to `200` ensures that control group `B` receives twice as much as CPU bandwidth.
    /// (Assuming both `A` and `B` are of the same parent)
    pub fn set_shares(&self, shares: u64) -> Result<()> {
        self.open_path("cpu.shares", true).and_then(|mut file| {
            file.write_all(shares.to_string().as_ref())
                .map_err(|e| Error::with_source(ErrorKind::WriteFailed, e))
        })
    }

    /// Retrieve the CPU bandwidth that this control group (relative to other control groups and
    /// this control group's parent) can use.
    pub fn shares(&self) -> Result<u64> {
        self.open_path("cpu.shares", false).and_then(read_u64_from)
    }

    /// Specify a period (when using the CFS scheduler) of time in microseconds for how often this
    /// control group's access to the CPU should be reallocated.
    pub fn set_cfs_period(&self, us: u64) -> Result<()> {
        self.open_path("cpu.cfs_period_us", true)
            .and_then(|mut file| {
                file.write_all(us.to_string().as_ref())
                    .map_err(|e| Error::with_source(ErrorKind::WriteFailed, e))
            })
    }

    /// Retrieve the period of time of how often this cgroup's access to the CPU should be
    /// reallocated in microseconds.
    pub fn cfs_period(&self) -> Result<u64> {
        self.open_path("cpu.cfs_period_us", false)
            .and_then(read_u64_from)
    }

    /// Specify a quota (when using the CFS scheduler) of time in microseconds for which all tasks
    /// in this control group can run during one period (see: `set_cfs_period()`).
    pub fn set_cfs_quota(&self, us: u64) -> Result<()> {
        self.open_path("cpu.cfs_quota_us", true)
            .and_then(|mut file| {
                file.write_all(us.to_string().as_ref())
                    .map_err(|e| Error::with_source(ErrorKind::WriteFailed, e))
            })
    }

    /// Retrieve the quota of time for which all tasks in this cgroup can run during one period, in
    /// microseconds.
    pub fn cfs_quota(&self) -> Result<u64> {
        self.open_path("cpu.cfs_quota_us", false)
            .and_then(read_u64_from)
    }
}
