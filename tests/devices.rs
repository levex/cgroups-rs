//! Integration tests about the devices subsystem

use cgroups::devices::{DevicePermissions, DeviceType, DevicesController};
use cgroups::{Cgroup, DeviceResource};

#[test]
fn test_devices_parsing() {
    let hier = cgroups::hierarchies::V1::new();
    let cg = Cgroup::new(&hier, String::from("test_devices_parsing"));
    {
        let devices: &DevicesController = cg.controller_of().unwrap();

        // Deny access to all devices first
        devices
            .deny_device(
                DeviceType::All,
                -1,
                -1,
                &vec![
                    DevicePermissions::Read,
                    DevicePermissions::Write,
                    DevicePermissions::MkNod,
                ],
            )
            .expect("Failed to deny device");
        // Acquire the list of allowed devices after we denied all
        let allowed_devices = devices.allowed_devices();
        // Verify that there are no devices that we can access.
        assert!(allowed_devices.is_ok());
        assert_eq!(allowed_devices.unwrap(), Vec::new());

        // Now add mknod access to /dev/null device
        devices
            .allow_device(DeviceType::Char, 1, 3, &vec![DevicePermissions::MkNod])
            .expect("Failed to allow device");
        let allowed_devices = devices.allowed_devices();
        assert!(allowed_devices.is_ok());
        let allowed_devices = allowed_devices.unwrap();
        assert_eq!(allowed_devices.len(), 1);
        assert_eq!(
            allowed_devices[0],
            DeviceResource {
                allow: true,
                devtype: DeviceType::Char,
                major: 1,
                minor: 3,
                access: vec![DevicePermissions::MkNod],
            }
        );

        // Now deny, this device explicitly.
        devices
            .deny_device(DeviceType::Char, 1, 3, &DevicePermissions::all())
            .expect("Failed to deny device");
        // Finally, check that.
        let allowed_devices = devices.allowed_devices();
        // Verify that there are no devices that we can access.
        assert!(allowed_devices.is_ok());
        assert_eq!(allowed_devices.unwrap(), Vec::new());
    }
    cg.delete();
}
