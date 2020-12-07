//! Integration tests about the pids subsystem
use cgroups::pid::{PidController, PidMax};
use cgroups::Cgroup;
use cgroups::Controller;

use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult};

use libc::pid_t;

#[test]
fn create_and_delete_cgroup() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("create_and_delete_cgroup"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        pidcontroller
            .set_pid_max(PidMax::Value(1337))
            .expect("Failed to set max pid");
        let max = pidcontroller.get_pid_max();
        assert!(max.is_ok());
        assert_eq!(max.unwrap(), PidMax::Value(1337));
    }
    cg.delete();
}

#[test]
fn test_pids_current_is_zero() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_pids_current_is_zero"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        let current = pidcontroller.get_pid_current();
        assert_eq!(current.unwrap(), 0);
    }
    cg.delete();
}

#[test]
fn test_pids_events_is_zero() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_pids_events_is_zero"));
    {
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        let events = pidcontroller.get_pid_events();
        assert!(events.is_ok());
        assert_eq!(events.unwrap(), 0);
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
        let before = before.unwrap();

        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                // move the process into the control group
                pids.add_task(&(pid_t::from(child) as u64).into()).unwrap();

                println!("added task to cg: {:?}", child);

                // Set limit to one
                pids.set_pid_max(PidMax::Value(1))
                    .expect("Failed to set max pid");
                println!("err = {:?}", pids.get_pid_max());

                // wait on the child
                let res = waitpid(child, None);
                if let Ok(WaitStatus::Exited(_, e)) = res {
                    assert_eq!(e, 0i32);
                } else {
                    panic!("found result: {:?}", res);
                }

                // Check pids.events
                let events = pids.get_pid_events();
                assert!(events.is_ok());
                assert_eq!(events.unwrap(), before + 1);
            }
            Ok(ForkResult::Child) => loop {
                let pids_max = pids.get_pid_max();
                if pids_max.is_ok() && pids_max.unwrap() == PidMax::Value(1) {
                    if let Err(_) = fork() {
                        unsafe { libc::exit(0) };
                    } else {
                        unsafe { libc::exit(1) };
                    }
                }
            },
            Err(_) => panic!("failed to fork"),
        }
    }
    cg.delete();
}
