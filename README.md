# cgroups-rs
Native Rust library for managing control groups under Linux

# Example

## Create a control group, and limit the pid resource

``` rust
// Acquire a handle for the V1 cgroup hierarchy.
let hier = ::hierarchies::V1::new();
// Create a control group named "example" in the hierarchy.
let cg = Cgroup::new(&hier, String::from("example"), 0);
{
    // Get a handle to the pids controller of the control group.
    let pids: &PidController = cg.controller_of().expect("No pids controller in V1 hierarchy!");
    // Set the maximum amount of processes in the cgroup.
    pids.set_pid_max(PidMax::Value(10));
    // Check that this has had the desired effect by reading the value back from the kernel.
    assert_eq!(pids.get_pid_max(), Some(PidMax::Value(10)));
}
// Once done, delete the control group (and its associated controllers).
cg.delete();
```

# Disclaimer

This crate is licensed under:

- MIT License (see LICENSE-MIT); or
- Apache 2.0 LIcense (see LICENSE-Apache-2.0),

at your option.

Please note that this crate is under heavy development, we will use sematic
versioning, but during the `0.0.*` phase, no guarantees are made about
backwards compatibility.

Regardless, check back often and thanks for taking a look!
