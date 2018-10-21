//! This module allows the user to create a control group using the Builder pattern.
use error::*;

use {pid, BlkIoDeviceResource, BlkIoDeviceThrottleResource,  Cgroup, DeviceResource, Hierarchy, HugePageResource, NetworkPriority, Resources};

// let cgroup: Cgroup = CgroupBuilder::new("hello", V1)
//      .memory()
//          .kernel_memory_limit(1024 * 1024)
//          .memory_hard_limit(1024 * 1024)
//          .done()
//      .cpu()
//          .shares(100)
//          .done()
//      .devices()
//          .device(1000, 10, DeviceType::Block, true,
//                      vec![Read, Write, MkNod]
//          .device(6, 1, DeviceType::Char, false, vec![])
//          .done()
//      .network()
//          .class_id(1337)
//          .priority("eth0", 100)
//          .priority("wl0", 200)
//          .done()
//      .hugepages()
//          .limit("2M", 0)
//          .limit("4M", 4 * 1024 * 1024 * 100)
//          .limit("2G", 2 * 1024 * 1024 * 1024)
//      .blkio()
//          .weight(123)
//          .leaf_weight(99)
//          .weight_device(6, 1, 100, 55)
//          .weight_device(6, 1, 100, 55)
//          .throttle_iops()
//              .read(6, 1, 10)
//              .write(11, 1, 100)
//          .throttle_bps()
//              .read(6, 1, 10)
//              .write(11, 1, 100)
//          .done()
//      .build();
//


macro_rules! gen_setter {
    ($res:ident, $name:ident, $ty:ty) => {
        pub fn $name(mut self, $name: $ty) -> Self {
            self.cgroup.resources.$res.update_values = true;
            self.cgroup.resources.$res.$name = $name;
            self
        }
    }
}

/// A control group builder instance:
///
/// # Example
/// Bla bla. TODO.
pub struct CgroupBuilder<'a> {
    name: String,
    hierarchy: &'a Hierarchy,
    /// Internal, unsupported field: use the associated builders instead.
    resources: Resources, // XXX: this should not be public.
}

impl<'a> CgroupBuilder<'a> {
    /// Start building a control group with the supplied hierarchy and name pair.
    ///
    /// Note that this does not actually create the control group until `build()` is called.
    pub fn new(name: &'a str, hierarchy: &'a Hierarchy) -> CgroupBuilder<'a> {
        CgroupBuilder {
            name: name.to_owned(),
            hierarchy: hierarchy,
            resources: Resources::default(),
        }
    }

    pub fn memory(self) -> MemoryResourceBuilder<'a> {
        MemoryResourceBuilder {
            cgroup: self,
        }
    }

    pub fn pid(self) -> PidResourceBuilder<'a> {
        PidResourceBuilder {
            cgroup: self,
        }
    }

    pub fn cpu(self) -> CpuResourceBuilder<'a> {
        CpuResourceBuilder {
            cgroup: self,
        }
    }

    pub fn devices(self) -> DeviceResourceBuilder<'a> {
        DeviceResourceBuilder {
            cgroup: self,
        }
    }

    pub fn network(self) -> NetworkResourceBuilder<'a> {
        NetworkResourceBuilder {
            cgroup: self,
        }
    }

    pub fn hugepages(self) -> HugepagesResourceBuilder<'a> {
        HugepagesResourceBuilder {
            cgroup: self,
        }
    }

    pub fn blkio(self) -> BlkIoResourcesBuilder<'a> {
        BlkIoResourcesBuilder {
            cgroup: self,
            throttling_iops: false,
        }
    }

    // Finalize the control group, consuming the builder and creating the control group.
    pub fn build(self) -> Cgroup<'a> {
        let cg = Cgroup::new(self.hierarchy, self.name);
        cg.apply(&self.resources);
        cg
    }
}

pub struct MemoryResourceBuilder<'a> {
    cgroup: CgroupBuilder<'a>,
}

impl<'a> MemoryResourceBuilder<'a> {

    gen_setter!(memory, kernel_memory_limit, u64);
    gen_setter!(memory, memory_hard_limit, u64);
    gen_setter!(memory, memory_soft_limit, u64);
    gen_setter!(memory, kernel_tcp_memory_limit, u64);
    gen_setter!(memory, memory_swap_limit, u64);
    gen_setter!(memory, swappiness, u64);

    pub fn done(self) -> CgroupBuilder<'a> {
        self.cgroup
    }
}

pub struct PidResourceBuilder<'a> {
    cgroup: CgroupBuilder<'a>,
}

impl<'a> PidResourceBuilder<'a> {

    gen_setter!(pid, maximum_number_of_processes, pid::PidMax);

    pub fn done(self) -> CgroupBuilder<'a> {
        self.cgroup
    }
}

pub struct CpuResourceBuilder<'a> {
    cgroup: CgroupBuilder<'a>,
}

impl<'a> CpuResourceBuilder<'a> {

    gen_setter!(cpu, cpus, String);
    gen_setter!(cpu, mems, String);
    gen_setter!(cpu, shares, u64);
    gen_setter!(cpu, quota, i64);
    gen_setter!(cpu, period, u64);
    gen_setter!(cpu, realtime_runtime, i64);
    gen_setter!(cpu, realtime_period, u64);

    pub fn done(self) -> CgroupBuilder<'a> {
        self.cgroup
    }
}

pub struct DeviceResourceBuilder<'a> {
    cgroup: CgroupBuilder<'a>,
}

impl<'a> DeviceResourceBuilder<'a> {

    pub fn device(mut self,
                  major: i64,
                  minor: i64,
                  devtype: ::devices::DeviceType,
                  allow: bool,
                  access: Vec<::devices::DevicePermissions>)
            -> DeviceResourceBuilder<'a> {
        self.cgroup.resources.devices.update_values = true;
        self.cgroup.resources.devices.devices.push(DeviceResource {
            major,
            minor,
            devtype,
            allow,
            access
        });
        self
    }

    pub fn done(self) -> CgroupBuilder<'a> {
        self.cgroup
    }
}

pub struct NetworkResourceBuilder<'a> {
    cgroup: CgroupBuilder<'a>,
}

impl<'a> NetworkResourceBuilder<'a> {

    gen_setter!(network, class_id, u64);

    pub fn priority(mut self, name: String, priority: u64)
        -> NetworkResourceBuilder<'a> {
        self.cgroup.resources.network.update_values = true;
        self.cgroup.resources.network.priorities.push(NetworkPriority {
            name,
            priority,
        });
        self
    }

    pub fn done(self) -> CgroupBuilder<'a> {
        self.cgroup
    }
}

pub struct HugepagesResourceBuilder<'a> {
    cgroup: CgroupBuilder<'a>,
}

impl<'a> HugepagesResourceBuilder<'a> {

    pub fn limit(mut self, size: String, limit: u64)
        -> HugepagesResourceBuilder<'a> {
        self.cgroup.resources.hugepages.update_values = true;
        self.cgroup.resources.hugepages.limits.push(HugePageResource {
            size,
            limit,
        });
        self
    }

    pub fn done(self) -> CgroupBuilder<'a> {
        self.cgroup
    }
}

pub struct BlkIoResourcesBuilder<'a> {
    cgroup: CgroupBuilder<'a>,
    throttling_iops: bool,
}

impl<'a> BlkIoResourcesBuilder<'a> {

    gen_setter!(blkio, weight, u16);
    gen_setter!(blkio, leaf_weight, u16);

    pub fn weight_device(mut self,
                         major: u64,
                         minor: u64,
                         weight: u16,
                         leaf_weight: u16)
        -> BlkIoResourcesBuilder<'a> {
        self.cgroup.resources.blkio.update_values = true;
        self.cgroup.resources.blkio.weight_device.push(BlkIoDeviceResource {
            major,
            minor,
            weight,
            leaf_weight,
        });
        self
    }

    pub fn throttle_iops(mut self) -> BlkIoResourcesBuilder<'a> {
        self.throttling_iops = true;
        self
    }

    pub fn throttle_bps(mut self) -> BlkIoResourcesBuilder<'a> {
        self.throttling_iops = false;
        self
    }

    pub fn read(mut self, major: u64, minor: u64, rate: u64)
        -> BlkIoResourcesBuilder<'a> {
        self.cgroup.resources.blkio.update_values = true;
        let throttle = BlkIoDeviceThrottleResource {
            major,
            minor,
            rate,
        };
        if self.throttling_iops {
            self.cgroup.resources.blkio.throttle_read_iops_device.push(throttle);
        } else {
            self.cgroup.resources.blkio.throttle_read_bps_device.push(throttle);
        }
        self
    }

    pub fn write(mut self, major: u64, minor: u64, rate: u64)
        -> BlkIoResourcesBuilder<'a> {
        self.cgroup.resources.blkio.update_values = true;
        let throttle = BlkIoDeviceThrottleResource {
            major,
            minor,
            rate,
        };
        if self.throttling_iops {
            self.cgroup.resources.blkio.throttle_write_iops_device.push(throttle);
        } else {
            self.cgroup.resources.blkio.throttle_write_bps_device.push(throttle);
        }
        self
    }

    pub fn done(self) -> CgroupBuilder<'a> {
        self.cgroup
    }
}
