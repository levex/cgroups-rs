//! This module contains the implementation of the `freezer` cgroup subsystem.
//!
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/freezer-subsystem.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/freezer-subsystem.txt)
use std::io::{Read, Write};
use std::path::PathBuf;

use {CgroupError, ControllIdentifier, Controller, Controllers, Resources, Subsystem};

/// A controller that allows controlling the `freezer` subsystem of a Cgroup.
///
/// In essence, this subsystem allows the user to freeze and thaw (== "un-freeze") the processes in
/// the control group. This is done _transparently_ so that neither the parent, nor the children of
/// the processes can observe the freeze.
///
/// Note that if the control group is currently in the `Frozen` or `Freezing` state, then no
/// processes can be added to it.
#[derive(Debug, Clone)]
pub struct FreezerController {
    base: PathBuf,
    path: PathBuf,
}

/// The current state of the control group
pub enum FreezerState {
    /// The processes in the control group are _not_ frozen.
    Thawed,
    /// The processes in the control group are in the processes of being frozen.
    Freezing,
    /// The processes in the control group are frozen.
    Frozen,
}

impl Controller for FreezerController {
    fn control_type(&self) -> Controllers {
        Controllers::Freezer
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

    fn apply(&self, _res: &Resources) -> Result<(), CgroupError> {
        Ok(())
    }
}

impl ControllIdentifier for FreezerController {
    fn controller_type() -> Controllers {
        Controllers::Freezer
    }
}

impl<'a> From<&'a Subsystem> for &'a FreezerController {
    fn from(sub: &'a Subsystem) -> &'a FreezerController {
        unsafe {
            match sub {
                Subsystem::Freezer(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                }
            }
        }
    }
}

impl FreezerController {
    /// Contructs a new `FreezerController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Freezes the processes in the control group.
    pub fn freeze(&self) -> Result<(), CgroupError> {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("FROZEN".to_string().as_ref())
                .map_err(CgroupError::WriteError)
        })
    }

    /// Thaws, that is, unfreezes the processes in the control group.
    pub fn thaw(&self) -> Result<(), CgroupError> {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("THAWED".to_string().as_ref())
                .map_err(CgroupError::WriteError)
        })
    }

    /// Retrieve the state of processes in the control group.
    pub fn state(&self) -> Result<FreezerState, CgroupError> {
        self.open_path("freezer.state", false).and_then(|mut file| {
            let mut s = String::new();
            let res = file.read_to_string(&mut s);
            match res {
                Ok(_) => match s.as_ref() {
                    "FROZEN" => Ok(FreezerState::Frozen),
                    "THAWED" => Ok(FreezerState::Thawed),
                    "FREEZING" => Ok(FreezerState::Freezing),
                    _ => Err(CgroupError::ParseError),
                },
                Err(e) => Err(CgroupError::ReadError(e)),
            }
        })
    }
}
