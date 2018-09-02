//! This module contains the implementation of the `blkio` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/blkio-controller.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/blkio-controller.txt)
use std::path::PathBuf;
use std::io::{Read, Write};
use std::fs::File;

use {CgroupError, BlkIoResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};
use CgroupError::*;

/// A controller that allows controlling the `blkio` subsystem of a Cgroup.
///
/// In essence, using the `blkio` controller one can limit and throttle the tasks' usage of block
/// devices in the control group.
#[derive(Debug, Clone)]
pub struct BlkIoController {
    base: PathBuf,
    path: PathBuf,
}

/// Current state and statistics about how throttled are the block devices when accessed from the
/// controller's control group.
#[derive(Debug)]
pub struct BlkIoThrottle {
    /// Total amount of bytes transferred to and from the block devices.
    pub io_service_bytes: String,
    /// Same as `io_service_bytes`, but contains all descendant control groups.
    pub io_service_bytes_recursive: String,
    /// The number of I/O operations performed on the devices as seen by the throttling policy.
    pub io_serviced: String,
    /// Same as `io_serviced`, but contains all descendant control groups.
    pub io_serviced_recursive: String,
    /// The upper limit of bytes per second rate of read operation on the block devices by the
    /// control group's tasks.
    pub read_bps_device: String,
    /// The upper limit of I/O operation per second, when said operation is a read operation.
    pub read_iops_device: String,
    /// The upper limit of bytes per second rate of write operation on the block devices by the
    /// control group's tasks.
    pub write_bps_device: String,
    /// The upper limit of I/O operation per second, when said operation is a write operation.
    pub write_iops_device: String,
}

/// Statistics and state of the block devices.
#[derive(Debug)]
pub struct BlkIo {
    /// The number of BIOS requests merged into I/O requests by the control group's tasks.
    pub io_merged: String,
    /// Same as `io_merged`, but contains all descendant control groups.
    pub io_merged_recursive: String,
    /// The number of requests queued for I/O operations by the tasks of the control group.
    pub io_queued: String,
    /// Same as `io_queued`, but contains all descendant control groups.
    pub io_queued_recursive: String,
    /// The number of bytes transferred from and to the block device (as seen by the CFQ I/O
    /// scheduler).
    pub io_service_bytes: String,
    /// Same as `io_service_bytes`, but contains all descendant control groups.
    pub io_service_bytes_recursive: String,
    /// The number of I/O operations (as seen by the CFQ I/O scheduler) between the devices and the
    /// control group's tasks.
    pub io_serviced: String,
    /// Same as `io_serviced`, but contains all descendant control groups.
    pub io_serviced_recursive: String,
    /// The total time spent between dispatch and request completion for I/O requests (as seen by
    /// the CFQ I/O scheduler) by the control group's tasks.
    pub io_service_time: String,
    /// Same as `io_service_time`, but contains all descendant control groups.
    pub io_service_time_recursive: String,
    /// Total amount of time spent waiting for a free slot in the CFQ I/O scheduler's queue.
    pub io_wait_time: String,
    /// Same as `io_wait_time`, but contains all descendant control groups.
    pub io_wait_time_recursive: String,
    /// How much weight do the control group's tasks have when competing against the descendant
    /// control group's tasks.
    pub leaf_weight: u64,
    /// Same as `leaf_weight`, but per-block-device.
    pub leaf_weight_device: String,
    /// Total number of sectors transferred between the block devices and the control group's
    /// tasks.
    pub sectors: String,
    /// Same as `sectors`, but contains all descendant control groups.
    pub sectors_recursive: String,
    /// Similar statistics, but as seen by the throttle policy.
    pub throttle: BlkIoThrottle,
    /// The time the control group had access to the I/O devices.
    pub time: String,
    /// Same as `time`, but contains all descendant control groups.
    pub time_recursive: String,
    /// The weight of this control group.
    pub weight: u64,
    /// Same as `weight`, but per-block-device.
    pub weight_device: String,
}

impl Controller for BlkIoController {
    fn control_type(self: &Self) -> Controllers { Controllers::BlkIo }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let res: &BlkIoResources = &res.blkio;

        if res.update_values {
            let _ = self.set_weight(res.weight as u64);
            let _ = self.set_leaf_weight(res.leaf_weight as u64);

            for dev in &res.weight_device {
                let _ = self.set_weight_for_device(format!("{}:{} {}",
                                dev.major, dev.minor, dev.weight));
            }

            for dev in &res.throttle_read_bps_device {
                let _ = self.throttle_read_bps_for_device(dev.major, dev.minor, dev.rate);
            }

            for dev in &res.throttle_write_bps_device {
                let _ = self.throttle_write_bps_for_device(dev.major, dev.minor, dev.rate);
            }

            for dev in &res.throttle_read_iops_device {
                let _ = self.throttle_read_iops_for_device(dev.major, dev.minor, dev.rate);
            }

            for dev in &res.throttle_write_iops_device {
                let _ = self.throttle_write_iops_for_device(dev.major, dev.minor, dev.rate);
            }
        }
    }
}

impl ControllIdentifier for BlkIoController {
    fn controller_type() -> Controllers {
        Controllers::BlkIo
    }
}

impl<'a> From<&'a Subsystem> for &'a BlkIoController {
    fn from(sub: &'a Subsystem) -> &'a BlkIoController {
        unsafe {
            match sub {
                Subsystem::BlkIo(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
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

fn read_u64_from(mut file: File) -> Result<u64, CgroupError> {
    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => string.trim().parse().map_err(|_| ParseError),
        Err(e) => Err(CgroupError::ReadError(e)),
    }
}

impl BlkIoController {
    /// Constructs a new `BlkIoController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Gathers statistics about and reports the state of the block devices used by the control
    /// group's tasks.
    pub fn blkio(self: &Self) -> BlkIo {
        BlkIo {
            io_merged: self.open_path("blkio.io_merged", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_merged_recursive: self.open_path("blkio.io_merged_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_queued: self.open_path("blkio.io_queued", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_queued_recursive: self.open_path("blkio.io_queued_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_service_bytes: self.open_path("blkio.io_service_bytes", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_service_bytes_recursive: self.open_path("blkio.io_service_bytes_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_serviced: self.open_path("blkio.io_serviced", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_serviced_recursive: self.open_path("blkio.io_serviced_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_service_time: self.open_path("blkio.io_service_time", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_service_time_recursive: self.open_path("blkio.io_service_time_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_wait_time: self.open_path("blkio.io_wait_time", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            io_wait_time_recursive: self.open_path("blkio.io_wait_time_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            leaf_weight: self.open_path("blkio.leaf_weight", false).and_then(|file| {
                read_u64_from(file)
            }).unwrap_or(0u64),
            leaf_weight_device: self.open_path("blkio.leaf_weight_device", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            sectors: self.open_path("blkio.sectors", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            sectors_recursive: self.open_path("blkio.sectors_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            throttle: BlkIoThrottle {
                io_service_bytes: self.open_path("blkio.throttle.io_service_bytes", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
                io_service_bytes_recursive: self.open_path("blkio.throttle.io_service_bytes_recursive", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
                io_serviced: self.open_path("blkio.throttle.io_serviced", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
                io_serviced_recursive: self.open_path("blkio.throttle.io_serviced_recursive", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
                read_bps_device: self.open_path("blkio.throttle.read_bps_device", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
                read_iops_device: self.open_path("blkio.throttle.read_iops_device", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
                write_bps_device: self.open_path("blkio.throttle.write_bps_device", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
                write_iops_device: self.open_path("blkio.throttle.write_iops_device", false).and_then(|file| {
                    read_string_from(file)
                }).unwrap_or("".to_string()),
            },
            time: self.open_path("blkio.time", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            time_recursive: self.open_path("blkio.time_recursive", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
            weight: self.open_path("blkio.weight", false).and_then(|file| {
                read_u64_from(file)
            }).unwrap_or(0u64),
            weight_device: self.open_path("blkio.weight_device", false).and_then(|file| {
                read_string_from(file)
            }).unwrap_or("".to_string()),
        }
    }

    /// Set the leaf weight on the control group's tasks, i.e., how are they weighted against the
    /// descendant control groups' tasks.
    pub fn set_leaf_weight(self: &Self, w: u64) -> Result<(), CgroupError> {
        self.open_path("blkio.leaf_weight", true).and_then(|mut file| {
            file.write_all(w.to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Same as `set_leaf_weight()`, but settable per each block device.
    pub fn set_leaf_weight_for_device(self: &Self, d: String) -> Result<(), CgroupError> {
        self.open_path("blkio.leaf_weight_device", true).and_then(|mut file| {
            file.write_all(d.as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Reset the statistics the kernel has gathered so far and start fresh.
    pub fn reset_stats(self: &Self) -> Result<(), CgroupError> {
        self.open_path("blkio.leaf_weight_device", true).and_then(|mut file| {
            file.write_all("1".to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Throttle the bytes per second rate of read operation affecting the block device
    /// `major:minor` to `bps`.
    pub fn throttle_read_bps_for_device(self: &Self, major: u64, minor: u64, bps: u64) -> Result<(), CgroupError> {
        self.open_path("blkio.throttle.read_bps_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, bps).to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Throttle the I/O operations per second rate of read operation affecting the block device
    /// `major:minor` to `bps`.
    pub fn throttle_read_iops_for_device(self: &Self, major: u64, minor: u64, iops: u64) -> Result<(), CgroupError> {
        self.open_path("blkio.throttle.read_iops_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, iops).to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }
    /// Throttle the bytes per second rate of write operation affecting the block device
    /// `major:minor` to `bps`.
    pub fn throttle_write_bps_for_device(self: &Self, major: u64, minor: u64, bps: u64) -> Result<(), CgroupError> {
        self.open_path("blkio.throttle.write_bps_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, bps).to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Throttle the I/O operations per second rate of write operation affecting the block device
    /// `major:minor` to `bps`.
    pub fn throttle_write_iops_for_device(self: &Self, major: u64, minor: u64, iops: u64) -> Result<(), CgroupError> {
        self.open_path("blkio.throttle.write_iops_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, iops).to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Set the weight of the control group's tasks.
    pub fn set_weight(self: &Self, w: u64) -> Result<(), CgroupError> {
        self.open_path("blkio.leaf_weight", true).and_then(|mut file| {
            file.write_all(w.to_string().as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Same as `set_weight()`, but settable per each block device.
    pub fn set_weight_for_device(self: &Self, d: String) -> Result<(), CgroupError> {
        self.open_path("blkio.weight_device", true).and_then(|mut file| {
            file.write_all(d.as_ref()).map_err(CgroupError::WriteError)
        })
    }
}
