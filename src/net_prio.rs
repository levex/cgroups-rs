/* Network priority controller */
use std::path::PathBuf;
use std::io::{BufReader, BufRead, Write, Read};
use std::fs::File;
use std::collections::HashMap;

use {NetworkResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};

#[derive(Debug, Clone)]
pub struct NetPrioController {
    base: PathBuf,
    path: PathBuf,
}

impl Controller for NetPrioController {
    fn control_type(self: &Self) -> Controllers { Controllers::NetPrio }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let res: &NetworkResources = &res.network;

        if res.update_values {
            for i in &res.priorities {
                self.set_if_prio(&i.name, i.priority);
            }
        }
    }
}

impl ControllIdentifier for NetPrioController {
    fn controller_type() -> Controllers {
        Controllers::NetPrio
    }
}

impl<'a> From<&'a Subsystem> for &'a NetPrioController {
    fn from(sub: &'a Subsystem) -> &'a NetPrioController {
        unsafe {
            match sub {
                Subsystem::NetPrio(c) => c,
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

impl NetPrioController {
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }
    pub fn prio_idx(self: &Self) -> u64 {
        self.open_path("net_prio.prioidx", false)
            .and_then(read_u64_from)
            .unwrap_or(0)
    }

    pub fn ifpriomap(self: &Self) -> HashMap<String, u64> {
        self.open_path("net_prio.ifpriomap", false)
            .and_then(|file| {
                let bf = BufReader::new(file);
                Some(bf.lines().map(|line| {
                    let l = line.unwrap();
                    let mut sp = l.split_whitespace();
                    (sp.nth(0).unwrap().to_string(),
                     sp.nth(1).unwrap().trim().parse().unwrap())
                }).collect())
            }).unwrap_or(HashMap::new())
    }

    pub fn set_if_prio(self: &Self, eif: &String, prio: u64) {
        self.open_path("net_prio.ifpriomap", true)
            .and_then(|mut file| {
                Some(file.write_all(format!("{} {}", eif, prio).as_ref()))
            });
    }
}
