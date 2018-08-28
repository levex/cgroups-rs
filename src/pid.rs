//! This module contains the implementation of the `pids` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroups-v1/pids.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/pids.txt)
use std::path::PathBuf;
use std::io::{Write, Read};

use {Resources, PidResources, Controller, ControllIdentifier, Subsystem, Controllers};

/// A controller that allows controlling the `pids` subsystem of a Cgroup.
#[derive(Debug, Clone)]
pub struct PidController {
    base: PathBuf,
    path: PathBuf,
}

/// The values found in the `pids.max` file in a Cgroup's `pids` subsystem.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum PidMax {
    /// This value is returned when the text found `pids.max` is `"max"`.
    Max,
    /// When the value in `pids.max` is a numerical value, they are returned via this enum field.
    Value(i64),
}

impl Default for PidMax {
    /// By default, (as per the kernel) `pids.max` should contain `"max"`.
    fn default() -> Self {
        PidMax::Max
    }
}

impl Controller for PidController {
    fn control_type(self: &Self) -> Controllers { Controllers::Pids }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let pidres: &PidResources = &res.pid;

        if pidres.update_values {
            /* apply pid_max */
            self.set_pid_max(pidres.maximum_number_of_processes);
        }
    }
}

/*impl<'a> ControllIdentifier for &'a PidController {
    fn controller_type() -> Controllers {
        Controllers::Pids
    }
}*/

impl ControllIdentifier for PidController {
    fn controller_type() -> Controllers {
        Controllers::Pids
    }
}

impl<'a> From<&'a Subsystem> for &'a PidController {
    fn from(sub: &'a Subsystem) -> &'a PidController {
        unsafe {
            match sub {
                Subsystem::Pid(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
            }
        }
    }
}

impl PidController {
    /// Constructors a new `PidController` instance, with `oroot` serving as the controller's root
    /// directory.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// The number of times `fork` failed because the limit was hit.
    pub fn get_pid_events(self: &Self) -> i64 {
        self.open_path("pids.events", false).and_then(|mut file| {
            let mut string = String::new();
            let _ = file.read_to_string(&mut string);
            Some(string.split_whitespace().nth(1).unwrap().parse().unwrap_or(0))
        }).unwrap()
    }

    /// The number of processes currently.
    pub fn get_pid_current(self: &Self) -> i64 {
        self.open_path("pids.current", false).and_then(|mut file| {
            let mut string = String::new();
            let _ = file.read_to_string(&mut string);
            Some(string.trim().parse().unwrap_or(0))
        }).unwrap()
    }

    /// The maximum number of processes that can exist at one time in the control group.
    pub fn get_pid_max(self: &Self) -> Option<PidMax> {
        self.open_path("pids.max", false).and_then(|mut file| {
            let mut string = String::new();
            let _ = file.read_to_string(&mut string);
            if string.trim() == "max" {
                Some(PidMax::Max)
            } else {
                Some(PidMax::Value(string.trim().parse().unwrap_or(0)))
            }
        })
    }

    /// Set the maximum number of processes that can exist in this control group.
    ///
    /// Note that if `get_pid_current()` returns a higher number than what you
    /// are about to set (`max_pid`), then no processess will be killed. Additonally, attaching
    /// extra processes to a control group disregards the limit.
    pub fn set_pid_max(self: &Self, max_pid: PidMax) {
        self.open_path("pids.max", true).and_then(|mut file| {
            let string_to_write = match max_pid {
                PidMax::Max => "max".to_string(),
                PidMax::Value(num) => num.to_string(),
            };
            match file.write_all(string_to_write.as_ref()) {
                Ok(_) => (),
                Err(e) => println!("error {:?}", e),
            }
            Some(0i64)
        });
    }
}
