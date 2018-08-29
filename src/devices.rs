//! This module contains the implementation of the `devices` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/devices.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/devices.txt)
use std::path::PathBuf;
use std::io::{Read, Write};

use {DeviceResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};

/// A controller that allows controlling the `devices` subsystem of a Cgroup.
///
/// In essence, using the devices controller, it is possible to allow or disallow sets of devices to
/// be used by the control group's tasks.
#[derive(Debug, Clone)]
pub struct DevicesController{
    base: PathBuf,
    path: PathBuf,
}

impl Controller for DevicesController {
    fn control_type(self: &Self) -> Controllers { Controllers::Devices }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let res: &DeviceResources = &res.devices;

        if res.update_values {
            for i in &res.devices {
                let wstr = format!("{} {}:{} {}",
                                   i.devtype, i.major, i.minor, i.access);
                if i.allow {
                    self.allow_device(&wstr);
                } else {
                    self.deny_device(&wstr);
                }
            }
        }
    }
}

impl ControllIdentifier for DevicesController {
    fn controller_type() -> Controllers {
        Controllers::Devices
    }
}

impl<'a> From<&'a Subsystem> for &'a DevicesController {
    fn from(sub: &'a Subsystem) -> &'a DevicesController {
        unsafe {
            match sub {
                Subsystem::Devices(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
            }
        }
    }
}

impl DevicesController {
    /// Constructs a new `DevicesController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Allow a (possibly, set of) device(s) to be used by the tasks in the control group.
    ///
    /// The format of `dev` is rather simple:
    /// `$type $major:$minor $rwm`
    /// where `$rwm` is a combination of the characters `r`, `w`, `m`, each standing for read,
    /// write, mknod permissions.
    ///
    /// Note that `dev` can be "regex"-like: both `$major` and `$minor` can be `*` which implies
    /// that their value does not matter.
    pub fn allow_device(self: &Self, dev: &String) {
        self.open_path("devices.allow", true).and_then(|mut file| {
            file.write_all(dev.as_ref()).ok()
        });
    }

    /// Deny the control group's tasks access to the devices covered by `dev`.
    ///
    /// The format of `dev` is rather simple:
    /// `$type $major:$minor $rwm`
    /// where `$rwm` is a combination of the characters `r`, `w`, `m`, each standing for read,
    /// write, mknod permissions.
    ///
    /// Note that `dev` can be "regex"-like: both `$major` and `$minor` can be `*` which implies
    /// that their value does not matter.
    pub fn deny_device(self: &Self, dev: &String) {
        self.open_path("devices.deny", true).and_then(|mut file| {
            file.write_all(dev.as_ref()).ok()
        });
    }

    /// Get the current list of allowed devices.
    pub fn allowed_devices(self: &Self) -> String {
        self.open_path("devices.list", false).and_then(|mut file| {
            let mut s = String::new();
            let _ = file.read_to_string(&mut s);
            Some(s)
        }).unwrap_or("".to_string())
    }
}
