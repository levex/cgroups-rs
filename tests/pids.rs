//! Integration tests about the pids subsystem
extern crate cgroups;
use cgroups::{CgroupError, CgroupPid, Cgroup, Resources, PidResources};
use cgroups::pid::{PidController, PidMax};
use cgroups::Controller;

extern crate nix;
use nix::unistd::{Pid, fork, ForkResult};
use nix::sys::wait::{waitpid, WaitStatus};

extern crate libc;
use libc::pid_t;

use std::thread;

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
fn test_pids_current_is_zero() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_pids_current_is_zero"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        assert_eq!(pidcontroller.get_pid_current(), 0);
    }
    cg.delete();
}

#[test]
fn test_pids_events_is_zero() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_pids_events_is_zero"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        assert_eq!(pidcontroller.get_pid_events(), 0);
    }
    cg.delete();
}

#[test]
fn test_pid_events_is_not_zero() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_pid_events_is_not_zero"));
    {
        let pids: &PidController = cg.controller_of().unwrap();
        let before = pids.get_pid_events();

        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                // move the process into the control group
                pids.add_task(&(pid_t::from(child) as u64).into());

                println!("added task to cg: {:?}", child);

                // Set limit to one
                pids.set_pid_max(PidMax::Value(1));
                println!("err = {:?}", pids.get_pid_max());

                // wait on the child
                let res = waitpid(child, None);
                if let Ok(WaitStatus::Exited(_, e)) = res {
                    assert_eq!(e, 0i32);
                } else {
                    panic!("found result: {:?}", res);
                }

                // Check pids.events
                assert_eq!(pids.get_pid_events(), before + 1);
            },
            Ok(ForkResult::Child) => {
                loop {
                    if pids.get_pid_max() == Some(PidMax::Value(1)) {
                        if let Err(_) = fork() {
                            unsafe { libc::exit(0) };
                        } else {
                            unsafe { libc::exit(1) };
                        }
                    }
                }
            },
            Err(_) => panic!("failed to fork"),
        }
    }
    cg.delete();
}
