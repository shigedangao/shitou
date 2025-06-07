use std::path::PathBuf;
use std::time;

use crate::device::{Command, Device, DeviceInfo};
use crate::ocr::Ocr;
use uuid::Uuid;

mod device;
mod ocr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = Device::init("127.0.0.1", 5037)?;
    device.connect()?;

    // Get the size of the device
    let size_adb_output = device.send_command(Command::DeviceSize, || "wm size".to_string())?;
    let device_info = DeviceInfo::new(size_adb_output)?;

    // Cancel for the next profile button coordinates
    let (x, y) = device_info.get_next_profile_button_coordinates();

    // Initialize an instance of the ocrs model
    let ocr_engine = Ocr::new(
        PathBuf::from("./model/text-detection.rten"),
        PathBuf::from("./model/text-recognition.rten"),
        vec!["<name>"],
    )?;

    let tx = ocr_engine.start_process_image_loop()?;

    // For loop to continuously process the profile
    loop {
        let uuid = Uuid::new_v4().to_string();
        let file_path_str = format!("/sdcard/hinge-{}.png", uuid);

        // Take a screenshot of the profile
        device.send_command(Command::TakeScreenshot, || {
            let screenshot_cmd = format!("screencap -p {}", file_path_str);

            screenshot_cmd
        })?;

        // Push dislike button to move to the next profile
        device.send_command(Command::NextProfile, || format!("input tap {} {}", x, y))?;

        // Wait for a few ms in order for the profile to load
        std::thread::sleep(time::Duration::from_secs(1));

        // Pull the screenshot into the local folder
        let output_file_path = device.pull_image(file_path_str, uuid.clone())?;

        tx.clone().send(output_file_path)?;
    }
}
