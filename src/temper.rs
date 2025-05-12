use rusb::{Context, Device, DeviceHandle, Error, UsbContext};

use core::time::Duration;

const TEMPER_VENDOR: u16 = 0x0c45;
const TEMPER_PRODUCT: u16 = 0x7401;
const TIMEOUT: Duration = Duration::new(5, 0);

pub struct TemperStick<T: UsbContext> {
    handle: DeviceHandle<T>,
}

impl<T: UsbContext> TemperStick<T> {
    fn new(dev: Device<T>) -> Self {
        TemperStick {
            handle: (&dev).open().unwrap(),
        }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        for i in 0..2 {
            match self.handle.detach_kernel_driver(i) {
                Ok(()) => (),
                Err(_) => eprintln!("kernel driver already detached..."),
            }
        }

        self.handle.set_active_configuration(1)?;

        for i in 0..2 {
            self.handle.claim_interface(i)?
        }

        self.handle
            .write_control(0x21, 0x09, 0x0201, 0x00, &[0x01, 0x01], TIMEOUT)?;

        let init_msgs: [[u8; 8]; 3] = [
            [0x01, 0x80, 0x33, 0x01, 0x00, 0x00, 0x00, 0x00],
            [0x01, 0x82, 0x77, 0x01, 0x00, 0x00, 0x00, 0x00],
            [0x01, 0x86, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00],
        ];

        let mut buf = [0u8; 8];

        for msg in &init_msgs {
            self.handle
                .write_control(0x21, 0x09, 0x0200, 0x01, msg, TIMEOUT)?;

            match self.handle.read_interrupt(0x82, &mut buf, TIMEOUT) {
                Ok(8) => (),
                Ok(x) => {
                    dbg!("only read {} bytes!", x);
                    return Err(Error::Other);
                }
                Err(e) => return Err(e),
            }
        }

        match self.handle.read_interrupt(0x82, &mut buf, TIMEOUT) {
            Ok(8) => (),
            Ok(x) => {
                dbg!("only read {} bytes!", x);
                return Err(Error::Other);
            }
            Err(e) => return Err(e),
        }

        Ok(())
    }

    pub fn get_temp(&self) -> Result<f32, Error> {
        let mut msg: [u8; 8] = [0x01, 0x80, 0x33, 0x01, 0x00, 0x00, 0x00, 0x00];

        self.handle
            .write_control(0x21, 0x09, 0x0200, 0x01, &msg, TIMEOUT)?;

        msg = [0u8; 8];

        match self.handle.read_interrupt(0x82, &mut msg, TIMEOUT) {
            Ok(8) => (),
            Ok(x) => {
                dbg!("only read {} bytes!", x);
                return Err(Error::Other);
            }
            Err(e) => return Err(e),
        }

        let temp_i = (msg[3] as i32) + ((msg[2] as i32) << 8);

        let temp: f32 = (temp_i as f32) * (125.0 / 32000.0);

        Ok(temp)
    }
}

impl<T: UsbContext> Drop for TemperStick<T> {
    fn drop(&mut self) {
        for i in 0..2 {
            self.handle.release_interface(i).ok();
            self.handle.attach_kernel_driver(i).ok();
        }
    }
}

pub struct Temper {
    ctx: rusb::Context,
}

impl Temper {
    pub fn new() -> Result<Self, ()> {
        if let Ok(c) = Context::new() {
            Ok(Temper { ctx: c })
        } else {
            Err(())
        }
    }

    pub fn get_sticks(&self) -> Vec<TemperStick<Context>> {
        let sticks: Vec<TemperStick<Context>> = self
            .ctx
            .devices()
            .unwrap()
            .iter()
            .filter(|x| {
                let dev_desc = x.device_descriptor().unwrap();
                dev_desc.vendor_id() == TEMPER_VENDOR && dev_desc.product_id() == TEMPER_PRODUCT
            })
            .map(TemperStick::new)
            .collect();

        sticks
    }
}
