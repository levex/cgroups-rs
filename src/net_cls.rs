//! This module contains the implementation of the `net_cls` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/net_cls.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/net_cls.txt)
use std::path::PathBuf;
use std::io::{Read, Write};
use std::fs::File;

use {NetworkResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};

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
    fn control_type(self: &Self) -> Controllers { Controllers::NetCls }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let res: &NetworkResources = &res.network;

        if res.update_values {
            self.set_class(res.class_id);
        }
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
                },
            }
        }
    }
}

fn read_u64_from(mut file: File) -> Option<u64> {
    let mut string = String::new();
    let _ = file.read_to_string(&mut string);
    string.trim().parse().ok()
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
    pub fn set_class(self: &Self, class: u64) {
        self.open_path("net_cls.classid", true).and_then(|mut file| {
            let s = format!("{:#08X}", class);
            file.write_all(s.as_ref()).ok()
        });
    }

    /// Get the network class id of the outgoing packets of the control group's tasks.
    pub fn get_class(self: &Self) -> u64 {
        self.open_path("net_cls.classid", false).and_then(|file| {
            read_u64_from(file)
        }).unwrap_or(0u64)
    }
}
