/* cpuset controller */
use std::path::PathBuf;
use std::io::{Read, Write};
use std::fs::File;

use {CpuResources, Resources, Controller, ControllIdentifier, Subsystem, Controllers};

#[derive(Debug, Clone)]
pub struct CpuSetController {
    base: PathBuf,
    path: PathBuf,
}

pub struct CpuSet {
    pub cpu_exclusive: bool,
    pub cpus: String,
    pub effective_cpus: String,
    pub effective_mems: String,
    pub mem_exclusive: bool,
    pub mem_hardwall: bool,
    pub memory_migrate: bool,
    pub memory_pressure: u64,
    pub memory_pressure_enabled: Option<bool>,
    pub memory_spread_page: bool, 
    pub memory_spread_slab: bool, 
    pub mems: String,
    pub sched_load_balance: bool,
    pub sched_relax_domain_level: u64,

}

impl Controller for CpuSetController {
    fn control_type(self: &Self) -> Controllers { Controllers::CpuSet }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let res: &CpuResources = &res.cpu;

        if res.update_values {
            /* apply pid_max */
            self.set_cpus(&res.cpus);
            self.set_mems(&res.mems);
        }
    }
}

impl ControllIdentifier for CpuSetController {
    fn controller_type() -> Controllers {
        Controllers::CpuSet
    }
}

impl<'a> From<&'a Subsystem> for &'a CpuSetController {
    fn from(sub: &'a Subsystem) -> &'a CpuSetController {
        unsafe {
            match sub {
                Subsystem::CpuSet(c) => c,
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

impl CpuSetController {
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    pub fn cpuset(self: &Self) -> CpuSet {
        CpuSet {
            cpu_exclusive: {
                self.open_path("cpuset.cpu_exclusive", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1).unwrap_or(false)
            },
            cpus: {
                self.open_path("cpuset.cpus", false).and_then(|mut file| {
                    let mut string = String::new();
                    let _ = file.read_to_string(&mut string);
                    Some(string.trim().to_string())
                }).unwrap()
            },
            effective_cpus: {
                self.open_path("cpuset.effective_cpus", false).and_then(|mut file| {
                    let mut string = String::new();
                    let _ = file.read_to_string(&mut string);
                    Some(string.trim().to_string())
                }).unwrap()
            },
            effective_mems: {
                self.open_path("cpuset.effective_mems", false).and_then(|mut file| {
                    let mut string = String::new();
                    let _ = file.read_to_string(&mut string);
                    Some(string.trim().to_string())
                }).unwrap()
            },
            mem_exclusive: {
                self.open_path("cpuset.mem_exclusive", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1).unwrap_or(false)
            },
            mem_hardwall: {
                self.open_path("cpuset.mem_hardwall", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1).unwrap_or(false)
            },
            memory_migrate: {
                self.open_path("cpuset.memory_migrate", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1).unwrap_or(false)
            },
            memory_pressure: {
                self.open_path("cpuset.memory_pressure", false).and_then(|file| {
                    read_u64_from(file)
                }).unwrap_or(0)
            },
            memory_pressure_enabled: {
                self.open_path("cpuset.memory_pressure_enabled", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1)
            },
            memory_spread_page: {
                self.open_path("cpuset.memory_spread_page", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1).unwrap_or(false)
            },
            memory_spread_slab: {
                self.open_path("cpuset.memory_spread_slab", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1).unwrap_or(false)
            },
            mems: {
                self.open_path("cpuset.mems", false).and_then(|mut file| {
                    let mut string = String::new();
                    let _ = file.read_to_string(&mut string);
                    Some(string.trim().to_string())
                }).unwrap()
            },
            sched_load_balance: {
                self.open_path("cpuset.sched_load_balance", false).and_then(|file| {
                    read_u64_from(file)
                }).map(|x| x == 1).unwrap_or(false)
            },
            sched_relax_domain_level: {
                self.open_path("cpuset.sched_relax_domain_level", false).and_then(|file| {
                    read_u64_from(file)
                }).unwrap_or(0)
            },
        }
    }

    pub fn set_cpu_exclusive(self: &Self, b: bool) {
        self.open_path("cpuset.cpu_exclusive", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }

    pub fn set_mem_exclusive(self: &Self, b: bool) {
        self.open_path("cpuset.mem_exclusive", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }

    pub fn set_cpus(self: &Self, cpus: &String) {
        self.open_path("cpuset.cpus", true).and_then(|mut file| {
            file.write_all(cpus.as_ref()).ok()
        });
    }

    pub fn set_mems(self: &Self, mems: &String) {
        self.open_path("cpuset.mems", true).and_then(|mut file| {
            file.write_all(mems.as_ref()).ok()
        });
    }

    pub fn set_hardwall(self: &Self, b: bool) {
        self.open_path("cpuset.mem_hardwall", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }

    pub fn set_load_balancing(self: &Self, b: bool) {
        self.open_path("cpuset.sched_load_balance", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }

    pub fn set_rebalance_relax_domain_level(self: &Self, i: i64) {
        self.open_path("cpuset.sched_relax_domain_level", true).and_then(|mut file| {
            file.write_all(i.to_string().as_ref()).ok()
        });
    }

    pub fn set_memory_migration(self: &Self, b: bool) {
        self.open_path("cpuset.memory_migrate", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }

    pub fn set_memory_spread_page(self: &Self, b: bool) {
        self.open_path("cpuset.memory_spread_page", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }

    pub fn set_memory_spread_slab(self: &Self, b: bool) {
        self.open_path("cpuset.memory_spread_slab", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }

    pub fn set_enable_memory_pressure(self: &Self, b: bool) {
        /* XXX: this file should only be present in the root cpuset cg */
        self.open_path("cpuset.memory_pressure_enabled", true).and_then(|mut file| {
            if b {
                file.write_all(b"1").ok()
            } else {
                file.write_all(b"0").ok()
            }
        });
    }
}
