//! Integration test about setting resources using `apply()`
extern crate cgroups;

use cgroups::pid::{PidController, PidMax};
use cgroups::{Cgroup, PidResources, Resources};

#[test]
fn pid_resources() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("pid_resources"));
    {
        let res = Resources {
            pid: PidResources {
                update_values: true,
                maximum_number_of_processes: PidMax::Value(512),
            },
            ..Default::default()
        };
        cg.apply(&res);

        // verify
        let pidcontroller: &PidController = cg.controller_of().unwrap();
        let pid_max = pidcontroller.get_pid_max();
        assert_eq!(pid_max.is_ok(), true);
        assert_eq!(pid_max.unwrap(), PidMax::Value(512));
    }
    cg.delete();
}
