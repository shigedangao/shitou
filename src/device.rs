use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice};
use std::{
    io::{BufReader, BufWriter, Write},
    net::{Ipv4Addr, SocketAddrV4},
};

enum Command {
    SwipeRight(f64, f64),
    TakeScreenshot,
    DeviceSize,
    NextProfile,
}

struct Device {
    address: Ipv4Addr,
    port: u16,
    device: Option<ADBServerDevice>,
}

impl Device {
    fn init<S: AsRef<str>>(addr: S, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let addr = addr.as_ref().parse::<Ipv4Addr>()?;

        Ok(Self {
            address: addr,
            port,
            device: None,
        })
    }

    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut server = ADBServer::new(SocketAddrV4::new(self.address, self.port));
        self.device = server.get_device().ok();

        Ok(())
    }

    fn send_command<S, F>(
        &mut self,
        cmd: Command,
        mut process_func: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        S: AsRef<str>,
        F: FnMut(String) -> Result<(), Box<dyn std::error::Error>>,
    {
        let args = match cmd {
            Command::DeviceSize => ["wm size"],
            Command::TakeScreenshot => ["screencap -p /sdcard/screenshot.png"],
            Command::SwipeRight(_, _) => [""],
            Command::NextProfile => [""],
        };

        let mut buffer = Vec::new();
        let mut writer = BufWriter::new(&mut buffer);

        if let Some(device) = &mut self.device {
            device.shell_command(&args, &mut writer)?;
        }

        // Read the data from the buffer. The writer is flush automatically by the adb library
        let output = String::from_utf8(writer.buffer().to_vec())?;

        // Process the output by using the given method
        process_func(output)?;

        Ok(())
    }
}
