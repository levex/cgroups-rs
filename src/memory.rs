//! This module contains the implementation of the `memory` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/memory.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/memory.txt)
use std::path::PathBuf;
use std::io::{Write, Read};
use std::fs::File;

use {Resources, MemoryResources, Controller, Controllers, Subsystem, ControllIdentifier};

/// A controller that allows controlling the `memory` subsystem of a Cgroup.
///
/// In essence, using the memory controller, the user can gather statistics about the memory usage
/// of the tasks in the control group. Additonally, one can also set powerful limits on their
/// memory usage.
#[derive(Debug, Clone)]
pub struct MemController{
    base: PathBuf,
    path: PathBuf,
}

/// Contains statistics about the current usage of memory and swap (together, not seperately) by
/// the control group's tasks.
#[derive(Debug)]
pub struct MemSwap {
    /// How many times the limit has been hit.
    pub fail_cnt: u64,
    /// Memory and swap usage limit in bytes.
    pub limit_in_bytes: u64,
    /// Current usage of memory and swap in bytes.
    pub usage_in_bytes: u64,
    /// The maximum observed usage of memory and swap in bytes.
    pub max_usage_in_bytes: u64,
}

/// State of and statistics gathered by the kernel about the memory usage of the control group's
/// tasks.
#[derive(Debug)]
pub struct Memory {
    /// How many times the limit has been hit.
    pub fail_cnt: u64,
    /// The limit in bytes of the memory usage of the control group's tasks.
    pub limit_in_bytes: u64,
    /// The current usage of memory by the control group's tasks.
    pub usage_in_bytes: u64,
    /// The maximum observed usage of memory by the control group's tasks.
    pub max_usage_in_bytes: u64,
    /// Whether moving charges at immigrate is allowed.
    pub move_charge_at_immigrate: u64,
    /* TODO: parse this */
    /// Contains various statistics about the NUMA locality of the control group's tasks.
    ///
    /// The format of this field (as lifted from the kernel sources):
    /// ```text
    /// total=<total pages> N0=<node 0 pages> N1=<node 1 pages> ...
    /// file=<total file pages> N0=<node 0 pages> N1=<node 1 pages> ...
    /// anon=<total anon pages> N0=<node 0 pages> N1=<node 1 pages> ...
    /// unevictable=<total anon pages> N0=<node 0 pages> N1=<node 1 pages> ...
    /// hierarchical_<counter>=<counter pages> N0=<node 0 pages> N1=<node 1 pages> ...
    /// ```
    pub numa_stat: String,
    /// If this equals "1", then the OOM killer is enabled for this control group (this is the
    /// default setting).
    pub oom_control: String,
    /// Allows setting a limit to memory usage which is enforced when the system (note, _not_ the
    /// control group) detects memory pressure.
    pub soft_limit_in_bytes: u64,
    /* TODO: parse this */
    /// Contains a wide array of statistics about the memory usage of the tasks in the control
    /// group.
    pub stat: String,
    /// Set the tendency of the kernel to swap out parts of the address space consumed by the
    /// control group's tasks.
    ///
    /// Note that setting this to zero does *not* prevent swapping, use `mlock(2)` for that
    /// purpose.
    pub swappiness: u64,
    /// If set, then under OOM conditions, the kernel will try to reclaim memory from the children
    /// of the offending process too. By default, this is not allowed.
    pub use_hierarchy: u64,
}

/// The current state of and gathered statistics about the kernel's memory usage for TCP-related
/// data structures.
#[derive(Debug)]
pub struct Tcp {
    /// How many times the limit has been hit.
    pub fail_cnt: u64,
    /// The limit in bytes of the memory usage of the kernel's TCP buffers by control group's
    /// tasks.
    pub limit_in_bytes: u64,
    /// The current memory used by the kernel's TCP buffers related to these tasks.
    pub usage_in_bytes: u64,
    /// The observed maximum usage of memory by the kernel's TCP buffers (that originated from
    /// these tasks).
    pub max_usage_in_bytes: u64,
}

/// Gathered statistics and the current state of limitation of the kernel's memory usage. Note that
/// this is per-cgroup, so the kernel can of course use more memory, but it will fail operations by
/// these tasks if it would think that the limits here would be violated. It's important to note
/// that interrupts in particular might not be able to enforce these limits.
#[derive(Debug)]
pub struct Kmem {
    /// How many times the limit has been hit.
    pub fail_cnt: u64,
    /// The limit in bytes of the kernel memory used by the control group's tasks.
    pub limit_in_bytes: u64,
    /// The current usage of kernel memory used by the control group's tasks, in bytes.
    pub usage_in_bytes: u64,
    /// The maximum observed usage of kernel memory used by the control group's tasks, in bytes.
    pub max_usage_in_bytes: u64,
    /// Contains information about the memory usage of the kernel's caches, per control group.
    pub slabinfo: String,
}

impl Controller for MemController {
    fn control_type(self: &Self) -> Controllers { Controllers::Mem }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let memres: &MemoryResources = &res.memory;

        if memres.update_values {
            self.set_limit(memres.memory_hard_limit);
            self.set_soft_limit(memres.memory_soft_limit);
            self.set_kmem_limit(memres.kernel_memory_limit);
            self.set_memswap_limit(memres.memory_swap_limit);
            self.set_tcp_limit(memres.kernel_tcp_memory_limit);
            self.set_swappiness(memres.swappiness);
        }
    }
}

impl MemController {
    /// Contructs a new `MemController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Gathers overall statistics (and the current state of) about the memory usage of the control
    /// group's tasks.
    ///
    /// See the individual fields for more explanation, and as always, remember to consult the
    /// kernel Documentation and/or sources.
    pub fn memory_stat(self: &Self) -> Memory {
        Memory {
            fail_cnt: self.open_path("memory.failcnt", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            limit_in_bytes: self.open_path("memory.limit_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            usage_in_bytes: self.open_path("memory.usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            max_usage_in_bytes: self.open_path("memory.max_usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            move_charge_at_immigrate: self.open_path("memory.move_charge_at_immigrate", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            numa_stat: self.open_path("memory.numa_stat", false)
                            .and_then(|mut file| {
                                let mut string = String::new();
                                let _ = file.read_to_string(&mut string);
                                Some(string.trim().to_string())
                            }).unwrap_or("".to_string()),
            oom_control: self.open_path("memory.oom_control", false)
                            .and_then(|mut file| {
                                let mut string = String::new();
                                let _ = file.read_to_string(&mut string);
                                Some(string.trim().to_string())
                            }).unwrap_or("".to_string()),
            soft_limit_in_bytes: self.open_path("memory.soft_limit_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            stat: self.open_path("memory.stat", false)
                            .and_then(|mut file| {
                                let mut string = String::new();
                                let _ = file.read_to_string(&mut string);
                                Some(string.trim().to_string())
                            }).unwrap_or("".to_string()),
            swappiness: self.open_path("memory.swappiness", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            use_hierarchy: self.open_path("memory.use_hierarchy", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0)
        }
    }

    /// Gathers information about the kernel memory usage of the control group's tasks.
    pub fn kmem_stat(self: &Self) -> Kmem {
        Kmem {
            fail_cnt: self.open_path("memory.kmem.failcnt", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            limit_in_bytes: self.open_path("memory.kmem.limit_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            usage_in_bytes: self.open_path("memory.kmem.usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            max_usage_in_bytes: self.open_path("memory.kmem.max_usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            slabinfo: self.open_path("memory.kmem.slabinfo", false)
                            .and_then(|mut file| {
                                let mut string = String::new();
                                let _ = file.read_to_string(&mut string);
                                Some(string.trim().to_string())
                            }).unwrap_or("".to_string()),
        }
    }

    /// Gathers information about the control group's kernel memory usage where said memory is
    /// TCP-related.
    pub fn kmem_tcp_stat(self: &Self) -> Tcp {
        Tcp {
            fail_cnt: self.open_path("memory.kmem.tcp.failcnt", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            limit_in_bytes: self.open_path("memory.kmem.tcp.limit_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            usage_in_bytes: self.open_path("memory.kmem.tcp.usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            max_usage_in_bytes: self.open_path("memory.kmem.tcp.max_usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
        }
    }

    /// Gathers information about the memory usage of the control group including the swap usage
    /// (if any).
    pub fn memswap(self: &Self) -> MemSwap {
        MemSwap {
            fail_cnt: self.open_path("memory.memsw.failcnt", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            limit_in_bytes: self.open_path("memory.memsw.limit_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            usage_in_bytes: self.open_path("memory.memsw.usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
            max_usage_in_bytes: self.open_path("memory.memsw.max_usage_in_bytes", false)
                            .and_then(|file| read_u64_from(file))
                            .unwrap_or(0),
        }
    }

    /// Set the memory usage limit of the control group, in bytes.
    pub fn set_limit(self: &Self, limit: u64) {
        self.open_path("memory.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

    /// Set the kernel memory limit of the control group, in bytes.
    pub fn set_kmem_limit(self: &Self, limit: u64) {
        self.open_path("memory.kmem.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

    /// Set the memory+swap limit of the control group, in bytes.
    pub fn set_memswap_limit(self: &Self, limit: u64) {
        self.open_path("memory.memsw.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

    /// Set how much kernel memory can be used for TCP-related buffers by the control group.
    pub fn set_tcp_limit(self: &Self, limit: u64) {
        self.open_path("memory.kmem.tcp.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }


    /// Set the soft limit of the control group, in bytes.
    ///
    /// This limit is enforced when the system is nearing OOM conditions. Contrast this with the
    /// hard limit, which is _always_ enforced.
    pub fn set_soft_limit(self: &Self, limit: u64) {
        self.open_path("memory.soft_limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }


    /// Set how likely the kernel is to swap out parts of the address space used by the control
    /// group.
    ///
    /// Note that a value of zero does not imply that the process will not be swapped out.
    pub fn set_swappiness(self: &Self, swp: u64) {
        self.open_path("memory.swappiness", true).and_then(|mut file| {
            file.write_all(swp.to_string().as_ref()).ok()
        });
    }
}

impl ControllIdentifier for MemController {
    fn controller_type() -> Controllers {
        Controllers::Mem
    }
}

impl<'a> From<&'a Subsystem> for &'a MemController {
    fn from(sub: &'a Subsystem) -> &'a MemController {
        unsafe {
            match sub {
                Subsystem::Mem(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
            }
        }
    }
}

fn read_u64_from(mut file: File) -> Option<u64> {
    let mut string = String::new();
    let _ = file.read_to_string(&mut string);
    string.trim().parse().ok()
}
