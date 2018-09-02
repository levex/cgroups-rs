//! Simple unit tests about the control groups system.
extern crate cgroups;
use cgroups::{Cgroup, CgroupPid};

extern crate nix;
extern crate libc;

#[test]
fn test_tasks_iterator() {
    let hier = cgroups::hierarchies::V1::new();
    let pid = libc::pid_t::from(nix::unistd::getpid()) as u64;
    let cg = Cgroup::new(&hier, String::from("test_tasks_iterator"));
    {
        // Add a task to the control group.
        cg.add_task(CgroupPid::from(pid));
        let mut tasks = cg.tasks().into_iter();
        // Verify that the task is indeed in the control group
        assert_eq!(tasks.next(), Some(CgroupPid::from(pid)));
        assert_eq!(tasks.next(), None);
        
        // Now, try removing it.
        cg.remove_task(CgroupPid::from(pid));
        tasks = cg.tasks().into_iter();

        // Verify that it was indeed removed.
        assert_eq!(tasks.next(), None);
    }
    cg.delete();
}
