//! This module contains the implementation of the `freezer` cgroup subsystem.
//!
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/freezer-subsystem.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/freezer-subsystem.txt)
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::error::ErrorKind::*;
use crate::error::*;

use crate::{ControllIdentifier, ControllerInternal, Controllers, Resources, Subsystem};

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

impl ControllerInternal for FreezerController {
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

    fn apply(&self, _res: &Resources) -> Result<()> {
        Ok(())
    }
}

impl ControllIdentifier for FreezerController {
    fn controller_type() -> Controllers {
        Controllers::Freezer
    }
}

impl_from_subsystem_for_controller!(Subsystem::Freezer, FreezerController);

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
    pub fn freeze(&self) -> Result<()> {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("FROZEN".to_string().as_ref())
                .map_err(|e| Error::with_cause(WriteFailed, e))
        })
    }

    /// Thaws, that is, unfreezes the processes in the control group.
    pub fn thaw(&self) -> Result<()> {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("THAWED".to_string().as_ref())
                .map_err(|e| Error::with_cause(WriteFailed, e))
        })
    }

    /// Retrieve the state of processes in the control group.
    pub fn state(&self) -> Result<FreezerState> {
        self.open_path("freezer.state", false).and_then(|mut file| {
            let mut s = String::new();
            let res = file.read_to_string(&mut s);
            match res {
                Ok(_) => match s.as_ref() {
                    "FROZEN" => Ok(FreezerState::Frozen),
                    "THAWED" => Ok(FreezerState::Thawed),
                    "FREEZING" => Ok(FreezerState::Freezing),
                    _ => Err(Error::new(ParseError)),
                },
                Err(e) => Err(Error::with_cause(ReadFailed, e)),
            }
        })
    }
}
