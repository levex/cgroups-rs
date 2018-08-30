//! Integration tests about the pids subsystem
extern crate cgroups;

use cgroups::{Cgroup, Resources, PidResources};
use cgroups::pid::{PidController, PidMax};

#[test]
fn create_and_delete_cgroup() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("create_and_delete_cgroup"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        pidcontroller.set_pid_max(PidMax::Value(1337));
        assert_eq!(pidcontroller.get_pid_max(), Some(PidMax::Value(1337)));
    }
    cg.delete();
}

#[test]
fn test_pid_pids_current_is_zero() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_pid_pids_current_is_zero"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        assert_eq!(pidcontroller.get_pid_current(), 0);
    }
    cg.delete();
}

#[test]
fn test_pid_pids_events_is_zero() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_pid_pids_events_is_zero"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        assert_eq!(pidcontroller.get_pid_events(), 0);
    }
    cg.delete();
}
