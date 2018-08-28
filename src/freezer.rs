/* Freezer controller */
use std::path::PathBuf;
use std::io::{Read, Write};

use {Controllers, Controller, Resources, ControllIdentifier, Subsystem};

#[derive(Debug, Clone)]
pub struct FreezerController{
    base: PathBuf,
    path: PathBuf,
}

pub enum FreezerState {
    Thawed,
    Freezing,
    Frozen,
}

impl Controller for FreezerController {
    fn control_type(self: &Self) -> Controllers { Controllers::Freezer }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, _res: &Resources) {
    }
}

impl ControllIdentifier for FreezerController {
    fn controller_type() -> Controllers {
        Controllers::Freezer
    }
}

impl<'a> From<&'a Subsystem> for &'a FreezerController {
    fn from(sub: &'a Subsystem) -> &'a FreezerController {
        unsafe {
            match sub {
                Subsystem::Freezer(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
            }
        }
    }
}

impl FreezerController {
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }
    pub fn freeze(self: &Self) {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("FROZEN".to_string().as_ref()).ok()
        });
    }

    pub fn thaw(self: &Self) {
        self.open_path("freezer.state", true).and_then(|mut file| {
            file.write_all("THAWED".to_string().as_ref()).ok()
        });
    }

    pub fn state(self: &Self) -> FreezerState {
        self.open_path("freezer.state", false).and_then(|mut file| {
            let mut s = String::new();
            let _ = file.read_to_string(&mut s);
            match s.as_ref() {
                "FROZEN" => Some(FreezerState::Frozen),
                "THAWED" => Some(FreezerState::Thawed),
                "FREEZING" => Some(FreezerState::Freezing),
                _ => None,
            }
        }).unwrap_or(FreezerState::Thawed)
    }
}
