use rusb::{
	Context,
	UsbContext,
	devices,
	Device,
	DeviceHandle,
	Error
};

pub struct Temper {
	ctx: rusb::Context,
}

pub struct TemperStick <T: UsbContext> {
	dev: Device<T>,
	handle: DeviceHandle<T>
}

const TEMPER_VENDOR:u16 = 0x0c45;
const TEMPER_PRODUCT:u16 = 0x7401;

impl Temper {
	pub fn new()->Result<Self, ()> {
		if let Ok(c)=Context::new() {
			Ok(Temper{ctx:c})
		}else{
			Err(())
		}
	}

	pub fn count_sticks(&self) -> Result<usize, Error> {
		Ok(self.ctx.devices()?.iter().filter(|x| {
				let dev_desc=x.device_descriptor().unwrap();
				dev_desc.vendor_id()==TEMPER_VENDOR && dev_desc.product_id()==TEMPER_PRODUCT
			}).count())
	}

	pub fn get_sticks(&self) -> Vec<TemperStick<UsbContext>> {
	
	}
}
