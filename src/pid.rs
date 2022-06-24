//! This module contains the implementation of the `pids` cgroup subsystem.
//!
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroups-v1/pids.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/pids.txt)
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::error::*;
use crate::error::ErrorKind::*;

use crate::{
    ControllIdentifier, ControllerInternal, Controllers, PidResources, Resources, Subsystem,
};

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

impl ControllerInternal for PidController {
    fn control_type(&self) -> Controllers {
        Controllers::Pids
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
        let pidres: &PidResources = &res.pid;

        if pidres.update_values {
            // apply pid_max
            let _ = self.set_pid_max(pidres.maximum_number_of_processes);

            // now, verify
            if self.get_pid_max()? == pidres.maximum_number_of_processes {
                return Ok(());
            } else {
                return Err(Error::new(Other));
            }
        }

        Ok(())
    }
}

// impl<'a> ControllIdentifier for &'a PidController {
//     fn controller_type() -> Controllers {
//         Controllers::Pids
//     }
// }

impl ControllIdentifier for PidController {
    fn controller_type() -> Controllers {
        Controllers::Pids
    }
}

impl<'a> From<&'a Subsystem> for &'a PidController {
    fn from(sub: &'a Subsystem) -> &'a PidController {
        
            match sub {
                Subsystem::Pid(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    unsafe { ::std::mem::uninitialized() }
                }
            }
        
    }
}

fn read_u64_from(mut file: File) -> Result<u64> {
    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => string.trim().parse().map_err(|e| Error::with_cause(ParseError, e)),
        Err(e) => Err(Error::with_cause(ReadFailed, e)),
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
    pub fn get_pid_events(&self) -> Result<u64> {
        self.open_path("pids.events", false).and_then(|mut file| {
            let mut string = String::new();
            match file.read_to_string(&mut string) {
                Ok(_) => match string.split_whitespace().nth(1) {
                    Some(elem) => match elem.parse() {
                        Ok(val) => Ok(val),
                        Err(e) => Err(Error::with_cause(ParseError, e)),
                    },
                    None => Err(Error::new(ParseError)),
                },
                Err(e) => Err(Error::with_cause(ReadFailed, e)),
            }
        })
    }

    /// The number of processes currently.
    pub fn get_pid_current(&self) -> Result<u64> {
        self.open_path("pids.current", false)
            .and_then(read_u64_from)
    }

    /// The maximum number of processes that can exist at one time in the control group.
    pub fn get_pid_max(&self) -> Result<PidMax> {
        self.open_path("pids.max", false).and_then(|mut file| {
            let mut string = String::new();
            let res = file.read_to_string(&mut string);
            match res {
                Ok(_) => if string.trim() == "max" {
                    Ok(PidMax::Max)
                } else {
                    match string.trim().parse() {
                        Ok(val) => Ok(PidMax::Value(val)),
                        Err(e) => Err(Error::with_cause(ParseError, e)),
                    }
                },
                Err(e) => Err(Error::with_cause(ReadFailed, e)),
            }
        })
    }

    /// Set the maximum number of processes that can exist in this control group.
    ///
    /// Note that if `get_pid_current()` returns a higher number than what you
    /// are about to set (`max_pid`), then no processess will be killed. Additonally, attaching
    /// extra processes to a control group disregards the limit.
    pub fn set_pid_max(&self, max_pid: PidMax) -> Result<()> {
        self.open_path("pids.max", true).and_then(|mut file| {
            let string_to_write = match max_pid {
                PidMax::Max => "max".to_string(),
                PidMax::Value(num) => num.to_string(),
            };
            match file.write_all(string_to_write.as_ref()) {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::with_cause(WriteFailed, e)),
            }
        })
    }
}
