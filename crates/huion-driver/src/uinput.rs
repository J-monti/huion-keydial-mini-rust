use evdev::uinput::VirtualDevice;
use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, Key, RelativeAxisType};
use std::thread;
use std::time::Duration;

use crate::hid;

pub fn create_device() -> Result<VirtualDevice, Box<dyn std::error::Error>> {
    let mut keys = AttributeSet::<Key>::new();
    for k in hid::all_supported_keys() {
        keys.insert(k);
    }

    // Retry up to 30 seconds waiting for /dev/uinput to be accessible
    let mut last_err = None;
    for attempt in 1..=30 {
        match VirtualDeviceBuilder::new()
            .and_then(|b| b.name("Huion Keydial Mini").with_keys(&keys))
            .and_then(|b| b.with_relative_axes(&AttributeSet::from_iter([RelativeAxisType::REL_WHEEL])))
            .and_then(|b| b.build())
        {
            Ok(vdev) => return Ok(vdev),
            Err(e) => {
                if attempt == 1 {
                    println!("Waiting for /dev/uinput to be accessible...");
                }
                if attempt % 10 == 0 {
                    println!("  Still waiting... (attempt {attempt}/30)");
                }
                last_err = Some(e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    Err(format!(
        "Failed to open /dev/uinput after 30 seconds: {}. \
         Ensure your user is in the 'input' group and /dev/uinput has group=input mode=0660",
        last_err.unwrap()
    ).into())
}
