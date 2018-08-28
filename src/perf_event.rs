/* Perf event controller */
use std::path::PathBuf;

use {Controllers, Controller, Resources, ControllIdentifier, Subsystem};

#[derive(Debug, Clone)]
pub struct PerfEventController {
    base: PathBuf,
    path: PathBuf,
}

impl Controller for PerfEventController {
    fn control_type(self: &Self) -> Controllers { Controllers::PerfEvent }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, _res: &Resources) {
    }
}

impl ControllIdentifier for PerfEventController {
    fn controller_type() -> Controllers {
        Controllers::PerfEvent
    }
}

impl<'a> From<&'a Subsystem> for &'a PerfEventController {
    fn from(sub: &'a Subsystem) -> &'a PerfEventController {
        unsafe {
            match sub {
                Subsystem::PerfEvent(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
            }
        }
    }
}

impl PerfEventController {
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }
}
