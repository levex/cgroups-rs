//! Simple unit tests about the control groups system.
use cgroups::{Cgroup, CgroupPid, Hierarchy, Subsystem};
use cgroups::memory::{MemController, SetMemory};
use std::collections::HashMap;

#[test]
fn test_tasks_iterator() {
    let h = cgroups::hierarchies::auto();
    let h = Box::new(&*h);
    let pid = libc::pid_t::from(nix::unistd::getpid()) as u64;
    let cg = Cgroup::new(h, String::from("test_tasks_iterator"));
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


#[test]
fn test_cgroup_with_prefix() {
    let h = cgroups::hierarchies::auto();
    let h = Box::new(&*h);
    let mut prefixes = HashMap::new();
    prefixes.insert("memory".to_string(), "/memory/abc/def".to_string());
    let cg = Cgroup::new_with_prefix(h, String::from("test_cgroup_with_prefix"), prefixes);
    {
        let subsystems = cg.subsystems();
        println!("mem path: {:?}", &subsystems);
        subsystems.into_iter().for_each(|sub| match sub {
            Subsystem::Pid(c) => {println!("path {:?}", c);},
            // base: "/sys/fs/cgroup", path: "/sys/fs/cgroup/memory/abc/def/test_cgroup_with_prefix"
            Subsystem::Mem(c) => {println!("path {:?}", c);},
            _ => {}, 
        });
    }
    cg.delete();
}
