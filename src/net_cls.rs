//! This module contains the implementation of the `net_cls` cgroup subsystem.
//!
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/net_cls.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/net_cls.txt)
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use CgroupError::*;
use {
    CgroupError, ControllIdentifier, Controller, Controllers, NetworkResources, Resources,
    Subsystem,
};

/// A controller that allows controlling the `net_cls` subsystem of a Cgroup.
///
/// In esssence, using the `net_cls` controller, one can attach a custom class to the network
/// packets emitted by the control group's tasks. This can then later be used in iptables to have
/// custom firewall rules, QoS, etc.
#[derive(Debug, Clone)]
pub struct NetClsController {
    base: PathBuf,
    path: PathBuf,
}

impl Controller for NetClsController {
    fn control_type(&self) -> Controllers {
        Controllers::NetCls
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

    fn apply(&self, res: &Resources) -> Result<(), CgroupError> {
        /* get the resources that apply to this controller */
        let res: &NetworkResources = &res.network;

        if res.update_values {
            let _ = self.set_class(res.class_id);
            if self.get_class() != Ok(res.class_id) {
                return Err(CgroupError::Unknown);
            }
        }
        return Ok(());
    }
}

impl ControllIdentifier for NetClsController {
    fn controller_type() -> Controllers {
        Controllers::NetCls
    }
}

impl<'a> From<&'a Subsystem> for &'a NetClsController {
    fn from(sub: &'a Subsystem) -> &'a NetClsController {
        unsafe {
            match sub {
                Subsystem::NetCls(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                }
            }
        }
    }
}

fn read_u64_from(mut file: File) -> Result<u64, CgroupError> {
    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => string.trim().parse().map_err(|_| ParseError),
        Err(e) => Err(CgroupError::ReadError(e)),
    }
}

impl NetClsController {
    /// Constructs a new `NetClsController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Set the network class id of the outgoing packets of the control group's tasks.
    pub fn set_class(&self, class: u64) -> Result<(), CgroupError> {
        self.open_path("net_cls.classid", true)
            .and_then(|mut file| {
                let s = format!("{:#08X}", class);
                file.write_all(s.as_ref()).map_err(CgroupError::WriteError)
            })
    }

    /// Get the network class id of the outgoing packets of the control group's tasks.
    pub fn get_class(&self) -> Result<u64, CgroupError> {
        self.open_path("net_cls.classid", false)
            .and_then(|file| read_u64_from(file))
    }
}
