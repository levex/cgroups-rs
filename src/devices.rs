//! This module contains the implementation of the `devices` cgroup subsystem.
//! 
//! See the Kernel's documentation for more information about this subsystem, found at:
//!  [Documentation/cgroup-v1/devices.txt](https://www.kernel.org/doc/Documentation/cgroup-v1/devices.txt)
use std::path::PathBuf;
use std::io::{Read, Write};

use {DeviceResource, CgroupError, DeviceResources, Controllers, Controller, Resources, ControllIdentifier, Subsystem};

/// A controller that allows controlling the `devices` subsystem of a Cgroup.
///
/// In essence, using the devices controller, it is possible to allow or disallow sets of devices to
/// be used by the control group's tasks.
#[derive(Debug, Clone)]
pub struct DevicesController{
    base: PathBuf,
    path: PathBuf,
}

/// An enum holding the different types of devices that can be manipulated using this controller.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeviceType {
    /// The rule applies to all devices.
    All,
    /// The rule only applies to character devices.
    Char,
    /// The rule only applies to block devices.
    Block,
}

impl Default for DeviceType {
    fn default() -> Self { DeviceType::All }
}

impl DeviceType {
    /// Convert a DeviceType into the character that the kernel recognizes.
    pub fn to_char(self: &Self) -> char {
        match self {
            DeviceType::All => 'a',
            DeviceType::Char => 'c',
            DeviceType::Block => 'b',
        }
    }

    /// Convert the kenrel's representation into the DeviceType type.
    pub fn from_char(c: Option<char>) -> Option<DeviceType> {
        match c {
            Some('a') => Some(DeviceType::All),
            Some('c') => Some(DeviceType::Char),
            Some('b') => Some(DeviceType::Block),
            _ => None,
        }
    }
}

/// An enum with the permissions that can be allowed/denied to the control group.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DevicePermissions {
    /// Permission to read from the device.
    Read,
    /// Permission to write to the device.
    Write,
    /// Permission to execute the `mknod(2)` system call with the device's major and minor numbers.
    /// That is, the permission to create a special file that refers to the device node.
    MkNod,
}

impl DevicePermissions {
    /// Convert a DevicePermissions into the character that the kernel recognizes.
    pub fn to_char(self: &Self) -> char {
        match self {
            DevicePermissions::Read => 'r',
            DevicePermissions::Write => 'w',
            DevicePermissions::MkNod => 'm',
        }
    }

    /// Convert a char to a DevicePermission if there is such a mapping.
    pub fn from_char(c: char) -> Option<DevicePermissions> {
        match c {
            'r' => Some(DevicePermissions::Read),
            'w' => Some(DevicePermissions::Write),
            'm' => Some(DevicePermissions::MkNod),
            _ => None,
        }
    }

    /// Checks whether the string is a valid descriptor of DevicePermissions.
    pub fn is_valid(s: &String) -> bool {
        if s == "" {
            return false;
        }
        for i in s.chars() {
            if i != 'r' && i != 'w' && i != 'm' {
                return false;
            }
        }
        return true;
    }

    /// Returns a Vec will all the permissions that a device can have.
    pub fn all() -> Vec<DevicePermissions> {
        vec![
            DevicePermissions::Read,
            DevicePermissions::Write,
            DevicePermissions::MkNod,
        ]
    }

    /// Convert a string into DevicePermissions.
    ///
    /// NOTE: This function makes no effort in verifying the String.
    pub fn from_string(s: &String) -> Vec<DevicePermissions> {
        let mut v = Vec::new();
        if s == "" {
            return v;
        }
        for e in s.chars() {
            v.push(DevicePermissions::from_char(e).unwrap());
        }

        v
    }
}

impl Controller for DevicesController {
    fn control_type(self: &Self) -> Controllers { Controllers::Devices }
    fn get_path<'a>(self: &'a Self) -> &'a PathBuf { &self.path }
    fn get_path_mut<'a>(self: &'a mut Self) -> &'a mut PathBuf { &mut self.path }
    fn get_base<'a>(self: &'a Self) -> &'a PathBuf { &self.base }

    fn apply(self: &Self, res: &Resources) {
        /* get the resources that apply to this controller */
        let res: &DeviceResources = &res.devices;

        if res.update_values {
            for i in &res.devices {
                if i.allow {
                    let _ = self.allow_device(i.devtype, i.major, i.minor, &i.access);
                } else {
                    let _ = self.deny_device(i.devtype, i.major, i.minor, &i.access);
                }
            }
        }
    }
}

impl ControllIdentifier for DevicesController {
    fn controller_type() -> Controllers {
        Controllers::Devices
    }
}

impl<'a> From<&'a Subsystem> for &'a DevicesController {
    fn from(sub: &'a Subsystem) -> &'a DevicesController {
        unsafe {
            match sub {
                Subsystem::Devices(c) => c,
                _ => {
                    assert_eq!(1, 0);
                    ::std::mem::uninitialized()
                },
            }
        }
    }
}

impl DevicesController {
    /// Constructs a new `DevicesController` with `oroot` serving as the root of the control group.
    pub fn new(oroot: PathBuf) -> Self {
        let mut root = oroot;
        root.push(Self::controller_type().to_string());
        Self {
            base: root.clone(),
            path: root,
        }
    }

    /// Allow a (possibly, set of) device(s) to be used by the tasks in the control group.
    ///
    /// When `-1` is passed as `major` or `minor`, the kernel interprets that value as "any",
    /// meaning that it will match any device.
    pub fn allow_device(self: &Self, devtype: DeviceType, major: i64, minor: i64, perm: &Vec<DevicePermissions>) -> Result<(), CgroupError> {
        let perms = perm.iter().map(DevicePermissions::to_char).collect::<String>();
        let minor = if minor == -1 { "*".to_string() } else { format!("{}", minor) };
        let major = if major == -1 { "*".to_string() } else { format!("{}", major) };
        let final_str = format!("{} {}:{} {}", devtype.to_char(), major, minor, perms);
        self.open_path("devices.allow", true).and_then(|mut file| {
            file.write_all(final_str.as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Deny the control group's tasks access to the devices covered by `dev`.
    ///
    /// When `-1` is passed as `major` or `minor`, the kernel interprets that value as "any",
    /// meaning that it will match any device.
    pub fn deny_device(self: &Self, devtype: DeviceType, major: i64, minor: i64, perm: &Vec<DevicePermissions>) -> Result<(), CgroupError> {
        let perms = perm.iter().map(DevicePermissions::to_char).collect::<String>();
        let minor = if minor == -1 { "*".to_string() } else { format!("{}", minor) };
        let major = if major == -1 { "*".to_string() } else { format!("{}", major) };
        let final_str = format!("{} {}:{} {}", devtype.to_char(), major, minor, perms);
        self.open_path("devices.deny", true).and_then(|mut file| {
            file.write_all(final_str.as_ref()).map_err(CgroupError::WriteError)
        })
    }

    /// Get the current list of allowed devices.
    pub fn allowed_devices(self: &Self) -> Result<Vec<DeviceResource>, CgroupError> {
        self.open_path("devices.list", false).and_then(|mut file| {
            let mut s = String::new();
            let res = file.read_to_string(&mut s);
            match res {
                Ok(_) => {
                    s.lines().fold(Ok(Vec::new()), |acc, line| {
                        let ls = line.to_string().split(|c| c == ' ' || c == ':').map(|x| x.to_string()).collect::<Vec<String>>();
                        if acc.is_err() || ls.len() != 4 {
                            println!("line 204: acc: {:?}, ls: {:?}", acc, ls);
                            Err(CgroupError::ParseError)
                        } else {
                            let devtype = DeviceType::from_char(ls[0].chars().nth(0));
                            let mut major = ls[1].parse::<i64>();
                            let mut minor = ls[2].parse::<i64>();
                            if major.is_err() && ls[1] == "*".to_string() {
                                major = Ok(-1);
                            }
                            if minor.is_err() && ls[2] == "*".to_string() {
                                minor = Ok(-1);
                            }
                            if devtype.is_none() || major.is_err() || minor.is_err() || !DevicePermissions::is_valid(&ls[3]) {
                                println!("line 211: acc: {:?}, ls: {:?}, devtype: {:?}, major {:?} minor {:?} ls3 {:?}",
                                         acc, ls, devtype, major, minor, &ls[3]);
                                Err(CgroupError::ParseError)
                            } else {
                                let access = DevicePermissions::from_string(&ls[3]);
                                let mut acc = acc.unwrap();
                                acc.push(DeviceResource {
                                    allow: true,
                                    devtype: devtype.unwrap(),
                                    major: major.unwrap(),
                                    minor: minor.unwrap(),
                                    access: access,
                                });
                                Ok(acc)
                            }
                        }
                    })
                },
                Err(e) => Err(CgroupError::ReadError(e)),
            }
        })
    }
}
