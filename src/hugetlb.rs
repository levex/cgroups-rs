// Copyright (c) 2018 Levente Kurusa
//
// SPDX-License-Identifier: Apache-2.0 or MIT
//

//! This module contains the implementation of the `hugetlb` cgroup subsystem.
//!
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/hugetlb.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/hugetlb.txt)
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::error::*;
use crate::error::ErrorKind::*;

use crate::{
    ControllIdentifier, ControllerInternal, Controllers, HugePageResources, Resources,
    Subsystem,
};

/// A controller that allows controlling the `hugetlb` subsystem of a Cgroup.
///
/// In essence, using this controller it is possible to limit the use of hugepages in the tasks of
/// the control group.
#[derive(Debug, Clone)]
pub struct HugeTlbController {
    base: PathBuf,
    path: PathBuf,
}

impl ControllerInternal for HugeTlbController {
    fn control_type(&self) -> Controllers {
        Controllers::HugeTlb
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
        let res: &HugePageResources = &res.hugepages;

        if res.update_values {
            for i in &res.limits {
                let _ = self.set_limit_in_bytes(&i.size, i.limit);
                if self.limit_in_bytes(&i.size)? != i.limit {
                    return Err(Error::new(Other));
                }
            }
        }
        Ok(())
    }
}

impl ControllIdentifier for HugeTlbController {
    fn controller_type() -> Controllers {
        Controllers::HugeTlb
    }
}

impl<'a> From<&'a Subsystem> for &'a HugeTlbController {
    fn from(sub: &'a Subsystem) -> &'a HugeTlbController {
        unsafe {
            match sub {
                Subsystem::HugeTlb(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                }
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

impl HugeTlbController {
    /// Constructs a new `HugeTlbController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Whether the system supports `hugetlb_size` hugepages.
    pub fn size_supported(&self, _hugetlb_size: &str) -> bool {
        // TODO
        true
    }

    /// Check how many times has the limit of `hugetlb_size` hugepages been hit.
    pub fn failcnt(&self, hugetlb_size: &str) -> Result<u64> {
        self.open_path(&format!("hugetlb.{}.failcnt", hugetlb_size), false)
            .and_then(read_u64_from)
    }

    /// Get the limit (in bytes) of how much memory can be backed by hugepages of a certain size
    /// (`hugetlb_size`).
    pub fn limit_in_bytes(&self, hugetlb_size: &str) -> Result<u64> {
        self.open_path(&format!("hugetlb.{}.limit_in_bytes", hugetlb_size), false)
            .and_then(read_u64_from)
    }

    /// Get the current usage of memory that is backed by hugepages of a certain size
    /// (`hugetlb_size`).
    pub fn usage_in_bytes(&self, hugetlb_size: &str) -> Result<u64> {
        self.open_path(&format!("hugetlb.{}.usage_in_bytes", hugetlb_size), false)
            .and_then(read_u64_from)
    }

    /// Get the maximum observed usage of memory that is backed by hugepages of a certain size
    /// (`hugetlb_size`).
    pub fn max_usage_in_bytes(&self, hugetlb_size: &str) -> Result<u64> {
        self.open_path(
            &format!("hugetlb.{}.max_usage_in_bytes", hugetlb_size),
            false,
        ).and_then(read_u64_from)
    }

    /// Set the limit (in bytes) of how much memory can be backed by hugepages of a certain size
    /// (`hugetlb_size`).
    pub fn set_limit_in_bytes(&self, hugetlb_size: &str, limit: u64) -> Result<()> {
        self.open_path(&format!("hugetlb.{}.limit_in_bytes", hugetlb_size), true)
            .and_then(|mut file| {
                file.write_all(limit.to_string().as_ref())
                    .map_err(|e| Error::with_cause(WriteFailed, e))
            })
    }
}
