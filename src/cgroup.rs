//! This module handles cgroup operations. Start here!

use {CgroupPid, Resources, ControllIdentifier, Controller, Hierarchy, Subsystem};

use std::convert::From;


/// A control group is the central structure to this crate.
///
///
/// # What are control groups?
///
/// Lifting over from the Linux kernel sources: 
///
/// > Control Groups provide a mechanism for aggregating/partitioning sets of
/// > tasks, and all their future children, into hierarchical groups with
/// > specialized behaviour.
///
/// This crate is an attempt at providing a Rust-native way of managing these cgroups.
pub struct Cgroup<'b> {
    /// The list of subsystems that control this cgroup
    subsystems: Vec<Subsystem>,

    /// The hierarchy.
    hier: &'b Hierarchy,
}

impl<'b> Cgroup<'b> {

    /// Create this control group.
    fn create(self: &Self) {
        for subsystem in &self.subsystems {
            subsystem.to_controller().create();
        }
    }

    /// Create a new control group in the hierarchy `hier`, with name `path`.
    ///
    /// Returns a handle to the control group that can be used to manipulate it.
    ///
    /// Note that if the handle goes out of scope and is dropped, the control group is _not_
    /// destroyed.
    pub fn new(hier: &Hierarchy, path: String) -> Cgroup {
        let cg = Cgroup::load(hier, path);
        cg.create();
        cg
    }

    /// Create a handle for a control group in the hierarchy `hier`, with name `path`.
    ///
    /// Returns a handle to the control group (that possibly does not exist until `create()` has
    /// been called on the cgroup.
    ///
    /// Note that if the handle goes out of scope and is dropped, the control group is _not_
    /// destroyed.
    pub fn load(hier: &Hierarchy, path: String) -> Cgroup {
        let mut subsystems = hier.subsystems();
        if path != ""  {
            subsystems = subsystems.into_iter().map(|x| x.enter(&path)).collect::<Vec<_>>();
        }

        let cg = Cgroup {
            subsystems: subsystems,
            hier: hier,
        };

        cg
    }

    /// The list of subsystems that this control group supports.
    pub fn subsystems(self: &Self) -> &Vec<Subsystem> {
        &self.subsystems
    }

    /// Deletes the control group.
    ///
    /// Note that this function makes no effort in cleaning up the descendant and the underlying
    /// system call will fail if there are any descendants. Thus, one should check whether it was
    /// actually removed, and remove the descendants first if not. In the future, this behavior
    /// will change.
    pub fn delete(self: Self) {
        self.subsystems.into_iter().for_each(|sub| {
            match sub {
                Subsystem::Pid(pidc) => pidc.delete(),
                Subsystem::Mem(c) => c.delete(),
                Subsystem::CpuSet(c) => c.delete(),
                Subsystem::CpuAcct(c) => c.delete(),
                Subsystem::Cpu(c) => c.delete(),
                Subsystem::Devices(c) => c.delete(),
                Subsystem::Freezer(c) => c.delete(),
                Subsystem::NetCls(c) => c.delete(),
                Subsystem::BlkIo(c) => c.delete(),
                Subsystem::PerfEvent(c) => c.delete(),
                Subsystem::NetPrio(c) => c.delete(),
                Subsystem::HugeTlb(c) => c.delete(),
                Subsystem::Rdma(c) => c.delete(),
            }
        });
    }

    /// Apply a set of resource limits to the control group.
    pub fn apply(self: &Self, res: &Resources) {
        for subsystem in &self.subsystems {
            subsystem.to_controller().apply(res);
        }
    }

    /// Retrieve a container based on type inference.
    ///
    /// ## Example:
    ///
    /// ```text
    /// let pids: &PidController = control_group.controller_of()
    ///                             .expect("No pids controller attached!");
    /// let cpu: &CpuController = control_group.controller_of()
    ///                             .expect("No cpu controller attached!");
    /// ```
    pub fn controller_of<'a, T>(self: &'a Self) -> Option<&'a T>
        where &'a T: From<&'a Subsystem>,
                  T: Controller + ControllIdentifier,
    {
        for i in &self.subsystems {
            if i.to_controller().control_type() == T::controller_type() {
                /*
                 * N.B.:
                 * https://play.rust-lang.org/?gist=978b2846bacebdaa00be62374f4f4334&version=stable&mode=debug&edition=2015
                 */
                return Some(i.into());
            }
        }
        None
    }

    /// Removes a task from the control group.
    ///
    /// Note that this means that the task will be moved back to the root control group in the
    /// hierarchy and any rules applied to that control group will _still_ apply to the task.
    pub fn remove_task(self: &Self, pid: CgroupPid) {
        self.hier.root_control_group().add_task(pid);
    }

    /// Attach a task to the control group.
    pub fn add_task(self: &Self, pid: CgroupPid) {
        self.subsystems().iter().for_each(|sub| sub.to_controller().add_task(&pid));
    }

    /// Returns an Iterator that can be used to iterate over the tasks that are currently in the
    /// control group.
    pub fn tasks(self: &Self) -> Vec<CgroupPid> {
        /* Collect the tasks from all subsystems */
        let mut v = self.subsystems().iter()
            .map(|x| x.to_controller().tasks())
            .fold(vec![], |mut acc, mut x| { acc.append(&mut x); acc });
        v.sort();
        v.dedup();
        v
    }

}
