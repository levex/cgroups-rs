/* CPU controller */
use std::path::PathBuf;
use std::io::{Read, Write};

use {CpuResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};

#[derive(Debug, Clone)]
pub struct CpuController{
    base: PathBuf,
    path: PathBuf,
}

#[derive(Debug)]
pub struct Cpu {
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
            self.set_shares(res.shares);
            self.set_cfs_period(res.period);
            self.set_cfs_quota(res.quota as u64);
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
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }
    pub fn cpu(self: &Self) -> Cpu {
        Cpu {
            stat: self.open_path("cpu.stat", false).and_then(|mut file| {
                let mut s = String::new();
                let _ = file.read_to_string(&mut s);
                Some(s)
            }).unwrap_or("".to_string()),
        }
    }

    pub fn set_shares(self: &Self, shares: u64) {
        self.open_path("cpu.shares", true).and_then(|mut file| {
            file.write_all(shares.to_string().as_ref()).ok()
        });
    }

    pub fn set_cfs_period(self: &Self, us: u64) {
        self.open_path("cpu.cfs_period_us", true).and_then(|mut file| {
            file.write_all(us.to_string().as_ref()).ok()
        });
    }

    pub fn set_cfs_quota(self: &Self, us: u64) {
        self.open_path("cpu.cfs_quota_us", true).and_then(|mut file| {
            file.write_all(us.to_string().as_ref()).ok()
        });
    }
}
