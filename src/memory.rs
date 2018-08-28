/* Memory controller */
use std::path::PathBuf;
use std::io::{Write, Read};
use std::fs::File;

use {Resources, MemoryResources, Controller, Controllers, Subsystem, ControllIdentifier};

#[derive(Debug, Clone)]
pub struct MemController{
    base: PathBuf,
    path: PathBuf,
}

#[derive(Debug)]
pub struct MemSwap {
    pub fail_cnt: u64,
    pub limit_in_bytes: u64,
    pub usage_in_bytes: u64,
    pub max_usage_in_bytes: u64,
}

#[derive(Debug)]
pub struct Memory {
    pub fail_cnt: u64,
    pub limit_in_bytes: u64,
    pub usage_in_bytes: u64,
    pub max_usage_in_bytes: u64,
    pub move_charge_at_immigrate: u64,
    /* TODO: parse this */
    pub numa_stat: String,
    /* TODO: parse this */
    pub oom_control: String,
    pub soft_limit_in_bytes: u64,
    /* TODO: parse this */
    pub stat: String,
    pub swappiness: u64,
    pub use_hierarchy: u64,
}

#[derive(Debug)]
pub struct Tcp {
    pub fail_cnt: u64,
    pub limit_in_bytes: u64,
    pub usage_in_bytes: u64,
    pub max_usage_in_bytes: u64,
}

#[derive(Debug)]
pub struct Kmem {
    pub fail_cnt: u64,
    pub limit_in_bytes: u64,
    pub usage_in_bytes: u64,
    pub max_usage_in_bytes: u64,
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
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

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

    pub fn set_limit(self: &Self, limit: u64) {
        self.open_path("memory.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

    pub fn set_kmem_limit(self: &Self, limit: u64) {
        self.open_path("memory.kmem.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

    pub fn set_memswap_limit(self: &Self, limit: u64) {
        self.open_path("memory.memsw.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

    pub fn set_tcp_limit(self: &Self, limit: u64) {
        self.open_path("memory.kmem.tcp.limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

    pub fn set_soft_limit(self: &Self, limit: u64) {
        self.open_path("memory.soft_limit_in_bytes", true).and_then(|mut file| {
            file.write_all(limit.to_string().as_ref()).ok()
        });
    }

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
