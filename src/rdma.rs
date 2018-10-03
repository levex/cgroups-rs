//! This module contains the implementation of the `rdma` cgroup subsystem.
//!
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/rdma.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/rdma.txt)
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use {CgroupError, ControllIdentifier, Controller, Controllers, Resources, Subsystem};

/// A controller that allows controlling the `rdma` subsystem of a Cgroup.
///
/// In essence, using this controller one can limit the RDMA/IB specific resources that the tasks
/// in the control group can use.
#[derive(Debug, Clone)]
pub struct RdmaController {
    base: PathBuf,
    path: PathBuf,
}

impl Controller for RdmaController {
    fn control_type(&self) -> Controllers {
        Controllers::Rdma
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

impl ControllIdentifier for RdmaController {
    fn controller_type() -> Controllers {
        Controllers::Rdma
    }
}

impl<'a> From<&'a Subsystem> for &'a RdmaController {
    fn from(sub: &'a Subsystem) -> &'a RdmaController {
        unsafe {
            match sub {
                Subsystem::Rdma(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                }
            }
        }
    }
}

fn read_string_from(mut file: File) -> Result<String, CgroupError> {
    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => Ok(string.trim().to_string()),
        Err(e) => Err(CgroupError::ReadError(e)),
    }
}

impl RdmaController {
    /// Constructs a new `RdmaController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Returns the current usage of RDMA/IB specific resources.
    pub fn current(&self) -> Result<String, CgroupError> {
        self.open_path("rdma.current", false)
            .and_then(read_string_from)
    }

    /// Set a maximum usage for each RDMA/IB resource.
    pub fn set_max(&self, max: &String) -> Result<(), CgroupError> {
        self.open_path("rdma.max", true).and_then(|mut file| {
            file.write_all(max.as_ref())
                .map_err(CgroupError::WriteError)
        })
    }
}
