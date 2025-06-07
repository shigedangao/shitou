use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice};
use regex::Regex;
use std::{
    fs::File,
    io::BufWriter,
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};

const ADB_OUTPUT_NEW_LINE: &str = "\n";

/// List of command available that can be send through ADB
pub enum Command {
    TakeScreenshot,
    DeviceSize,
    NextProfile,
}

#[derive(Default, Debug)]
pub struct DeviceInfo {
    pub width: u16,
    pub height: u16,
}

impl DeviceInfo {
    /// Get the "next button profile coordinates on the hinge app depending on the Samsung GS20 device"
    pub fn get_next_profile_button_coordinates(&self) -> (u16, u16) {
        (self.width - 925, self.height - 469)
    }
}

pub struct Device {
    address: Ipv4Addr,
    port: u16,
    device: Option<ADBServerDevice>,
}

impl Device {
    /// Create a new ADB Server instance
    ///
    /// * `addr` - S
    /// * `port` - u16
    pub fn init<S: AsRef<str>>(addr: S, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let addr = addr.as_ref().parse::<Ipv4Addr>()?;

        Ok(Self {
            address: addr,
            port,
            device: None,
        })
    }

    /// Connect to the device through ADB
    /// /!\ Developer mode need to be enabled
    pub fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut server = ADBServer::new(SocketAddrV4::new(self.address, self.port));
        self.device = server.get_device().ok();

        Ok(())
    }

    /// Send a command through ADB
    ///
    /// * `cmd` - Command
    /// * `cmd_func` - A closure which can be use to send a custom command
    pub fn send_command<F>(
        &mut self,
        cmd: Command,
        cmd_func: F,
    ) -> Result<String, Box<dyn std::error::Error>>
    where
        F: Fn() -> String,
    {
        let cmd_arg = cmd_func();

        let args = match cmd {
            Command::DeviceSize => [cmd_arg.as_str()],
            Command::TakeScreenshot => [cmd_arg.as_str()],
            Command::NextProfile => [cmd_arg.as_str()],
        };

        let mut buffer = Vec::new();
        let mut writer = BufWriter::new(&mut buffer);

        if let Some(device) = &mut self.device {
            device.shell_command(&args, &mut writer)?;
        }

        // Read the data from the buffer. The writer is flush automatically by the adb library
        let output = String::from_utf8(writer.buffer().to_vec())?;

        Ok(output)
    }

    /// Pull the image from the storage
    ///
    /// # Arguments
    ///
    /// * `path_str` - S
    /// * `uuid` - S
    pub fn pull_image<S: AsRef<str>>(
        &mut self,
        path_str: S,
        uuid: S,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let Some(device) = &mut self.device else {
            return Err(Box::from("Unable to get the device"));
        };

        let output_file_path = PathBuf::from(format!("./output/sources/{}.png", uuid.as_ref()));
        let mut file = File::create(&output_file_path)?;
        device.pull(&path_str, &mut file)?;

        Ok(output_file_path)
    }
}

impl DeviceInfo {
    /// Create a new DeviceInfo struct which contains the screen size of the device
    ///
    /// * `adb_output` - String
    pub fn new(adb_output: String) -> Result<Self, Box<dyn std::error::Error>> {
        let info = adb_output
            .split(ADB_OUTPUT_NEW_LINE)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();

        let re = Regex::new(r"\d+")?;

        if let Some(data) = info.last() {
            let size = re.find_iter(data).map(|m| m.as_str()).collect::<Vec<_>>();

            let width = size
                .get(0)
                .and_then(|m| m.parse::<u16>().ok())
                .ok_or("Unable to get the width")?;

            let height = size
                .get(1)
                .and_then(|m| m.parse::<u16>().ok())
                .ok_or("Unable to get the height")?;

            return Ok(Self { width, height });
        }

        Err(Box::from("Unable to get the size of the device"))
    }
}
