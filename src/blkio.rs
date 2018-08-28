/* block IO controller */
use std::path::PathBuf;
use std::io::{Read, Write};
use std::fs::File;

use {BlkIoResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};

#[derive(Debug, Clone)]
pub struct BlkIoController {
    base: PathBuf,
    path: PathBuf,
}

#[derive(Debug)]
pub struct BlkIoThrottle {
    pub io_service_bytes: String,
    pub io_service_bytes_recursive: String,
    pub io_serviced: String,
    pub io_serviced_recursive: String,
    pub read_bps_device: String,
    pub read_iops_device: String,
    pub write_bps_device: String,
    pub write_iops_device: String,
}

#[derive(Debug)]
pub struct BlkIo {
    pub io_merged: String,
    pub io_merged_recursive: String,
    pub io_queued: String,
    pub io_queued_recursive: String,
    pub io_service_bytes: String,
    pub io_service_bytes_recursive: String,
    pub io_serviced: String,
    pub io_serviced_recursive: String,
    pub io_service_time: String,
    pub io_service_time_recursive: String,
    pub io_wait_time: String,
    pub io_wait_time_recursive: String,
    pub leaf_weight: u64,
    pub leaf_weight_device: String,
    pub sectors: String,
    pub sectors_recursive: String,
    pub throttle: BlkIoThrottle,
    pub time: String,
    pub time_recursive: String,
    pub weight: u64,
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
            self.set_weight(res.weight as u64);
            self.set_leaf_weight(res.leaf_weight as u64);

            for dev in &res.weight_device {
                self.set_weight_for_device(format!("{}:{} {}",
                                dev.major, dev.minor, dev.weight));
            }

            for dev in &res.throttle_read_bps_device {
                self.throttle_read_bps_for_device(dev.major, dev.minor, dev.rate);
            }

            for dev in &res.throttle_write_bps_device {
                self.throttle_write_bps_for_device(dev.major, dev.minor, dev.rate);
            }

            for dev in &res.throttle_read_iops_device {
                self.throttle_read_iops_for_device(dev.major, dev.minor, dev.rate);
            }

            for dev in &res.throttle_write_iops_device {
                self.throttle_write_iops_for_device(dev.major, dev.minor, dev.rate);
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

fn read_string_from(mut file: File) -> Option<String> {
    let mut string = String::new();
    let _ = file.read_to_string(&mut string);
    Some(string.trim().to_string())
}

fn read_u64_from(mut file: File) -> Option<u64> {
    let mut string = String::new();
    let _ = file.read_to_string(&mut string);
    string.trim().parse().ok()
}

impl BlkIoController {
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }
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

    pub fn set_leaf_weight(self: &Self, w: u64) {
        self.open_path("blkio.leaf_weight", true).and_then(|mut file| {
            file.write_all(w.to_string().as_ref()).ok()
        });
    }

    pub fn set_leaf_weight_for_device(self: &Self, d: String) {
        self.open_path("blkio.leaf_weight_device", true).and_then(|mut file| {
            file.write_all(d.as_ref()).ok()
        });
    }

    pub fn reset_stats(self: &Self) {
        self.open_path("blkio.leaf_weight_device", true).and_then(|mut file| {
            file.write_all("1".to_string().as_ref()).ok()
        });
    }

    pub fn throttle_read_bps_for_device(self: &Self, major: u64, minor: u64, bps: u64) {
        self.open_path("blkio.throttle.read_bps_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, bps).to_string().as_ref()).ok()
        });
    }

    pub fn throttle_read_iops_for_device(self: &Self, major: u64, minor: u64, iops: u64) {
        self.open_path("blkio.throttle.read_iops_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, iops).to_string().as_ref()).ok()
        });
    }

    pub fn throttle_write_bps_for_device(self: &Self, major: u64, minor: u64, bps: u64) {
        self.open_path("blkio.throttle.write_bps_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, bps).to_string().as_ref()).ok()
        });
    }

    pub fn throttle_write_iops_for_device(self: &Self, major: u64, minor: u64, iops: u64) {
        self.open_path("blkio.throttle.write_iops_device", true).and_then(|mut file| {
            file.write_all(format!("{}:{} {}", major, minor, iops).to_string().as_ref()).ok()
        });
    }

    pub fn set_weight(self: &Self, w: u64) {
        self.open_path("blkio.leaf_weight", true).and_then(|mut file| {
            file.write_all(w.to_string().as_ref()).ok()
        });
    }

    pub fn set_weight_for_device(self: &Self, d: String) {
        self.open_path("blkio.weight_device", true).and_then(|mut file| {
            file.write_all(d.as_ref()).ok()
        });
    }
}
