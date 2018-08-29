//! This module contains the implementation of the `freezer` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/freezer-subsystem.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/freezer-subsystem.txt)
use std::path::PathBuf;
use std::io::{Read, Write};

use {Controllers, Controller, Resources, ControllIdentifier, Subsystem};

/// A controller that allows controlling the `freezer` subsystem of a Cgroup.
///
/// In essence, this subsystem allows the user to freeze and thaw (== "un-freeze") the processes in
/// the control group. This is done _transparently_ so that neither the parent, nor the children of
/// the processes can observe the freeze.
///
/// Note that if the control group is currently in the `Frozen` or `Freezing` state, then no
/// processes can be added to it.
#[derive(Debug, Clone)]
pub struct FreezerController{
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
    fn control_type(self: &Self) -> Controllers { Controllers::Freezer }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, _res: &Resources) {
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
                },
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
    pub fn freeze(self: &Self) {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("FROZEN".to_string().as_ref()).ok()
        });
    }

    /// Thaws, that is, unfreezes the processes in the control group.
    pub fn thaw(self: &Self) {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("THAWED".to_string().as_ref()).ok()
        });
    }

    /// Retrieve the state of processes in the control group.
    pub fn state(self: &Self) -> FreezerState {
        self.open_path("freezer.state", false).and_then(|mut file| {
            let mut s = String::new();
            let _ = file.read_to_string(&mut s);
            match s.as_ref() {
                "FROZEN" => Some(FreezerState::Frozen),
                "THAWED" => Some(FreezerState::Thawed),
                "FREEZING" => Some(FreezerState::Freezing),
                _ => None,
            }
        }).unwrap_or(FreezerState::Thawed)
    }
}
