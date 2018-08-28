/* RDMA controller */
use std::path::PathBuf;
use std::io::{Write, Read};
use std::fs::File;

use {Controllers, Controller, Resources, ControllIdentifier, Subsystem};

#[derive(Debug, Clone)]
pub struct RdmaController {
    base: PathBuf,
    path: PathBuf,
}

impl Controller for RdmaController {
    fn control_type(self: &Self) -> Controllers { Controllers::Rdma }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, _res: &Resources) {
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

impl RdmaController {
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }
    pub fn current(self: &Self) -> String {
        self.open_path("rdma.current", false)
            .and_then(read_string_from)
            .unwrap_or("".to_string())
    }

    pub fn set_max(self: &Self, max: &String) {
        self.open_path("rdma.max", true)
            .and_then(|mut file| {
                file.write_all(max.as_ref()).ok()
            });
    }
}
