/* cpuacct controller */
use std::path::PathBuf;
use std::io::{Read, Write};
use std::fs::File;

use {Controllers, Resources, Subsystem, ControllIdentifier, Controller};

#[derive(Debug, Clone)]
pub struct CpuAcctController {
    base: PathBuf,
    path: PathBuf,
}

pub struct CpuAcct {
    pub stat: String,
    pub usage: u64,
    pub usage_all: String,
    pub usage_percpu: String,
    pub usage_percpu_sys: String,
    pub usage_percpu_user: String,
    pub usage_sys: u64,
    pub usage_user: u64,
}

impl Controller for CpuAcctController {
    fn control_type(self: &Self) -> Controllers { Controllers::CpuAcct }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, _res: &Resources) {
    }
}

impl ControllIdentifier for CpuAcctController {
    fn controller_type() -> Controllers {
        Controllers::CpuAcct
    }
}

impl<'a> From<&'a Subsystem> for &'a CpuAcctController {
    fn from(sub: &'a Subsystem) -> &'a CpuAcctController {
        unsafe {
            match sub {
                Subsystem::CpuAcct(c) => c,
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

impl CpuAcctController {
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }
    pub fn cpuacct(self: &Self) -> CpuAcct {
        CpuAcct {
            stat: self.open_path("cpuacct.stat", false)
                    .and_then(|mut file| {
                        let mut string = String::new();
                        let _ = file.read_to_string(&mut string);
                        Some(string.trim().to_string())
                    }).unwrap_or("".to_string()),
            usage: self.open_path("cpuacct.usage", false)
                    .and_then(|file| read_u64_from(file))
                    .unwrap_or(0),
            usage_all: self.open_path("cpuacct.usage_all", false)
                    .and_then(|mut file| {
                        let mut string = String::new();
                        let _ = file.read_to_string(&mut string);
                        Some(string.trim().to_string())
                    }).unwrap_or("".to_string()),
            usage_percpu: self.open_path("cpuacct.usage_percpu", false)
                    .and_then(|mut file| {
                        let mut string = String::new();
                        let _ = file.read_to_string(&mut string);
                        Some(string.trim().to_string())
                    }).unwrap_or("".to_string()),
            usage_percpu_sys: self.open_path("cpuacct.usage_percpu_sys", false)
                    .and_then(|mut file| {
                        let mut string = String::new();
                        let _ = file.read_to_string(&mut string);
                        Some(string.trim().to_string())
                    }).unwrap_or("".to_string()),
            usage_percpu_user: self.open_path("cpuacct.usage_percpu_user", false)
                    .and_then(|mut file| {
                        let mut string = String::new();
                        let _ = file.read_to_string(&mut string);
                        Some(string.trim().to_string())
                    }).unwrap_or("".to_string()),
            usage_sys: self.open_path("cpuacct.usage_sys", false)
                    .and_then(|file| read_u64_from(file))
                    .unwrap_or(0),
            usage_user: self.open_path("cpuacct.usage_user", false)
                    .and_then(|file| read_u64_from(file))
                    .unwrap_or(0),
        }
    }
    pub fn reset(self: &Self) {
        self.open_path("cpuacct.usage", true).and_then(|mut file| {
            file.write_all(b"0").ok()
        });
    }
}
