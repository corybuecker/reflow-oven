use core::cell::RefCell;
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_time::Timer;
use esp_hal::{Blocking, i2c::master::I2c};

#[allow(dead_code)]
pub struct Temperature {
    value: CriticalSectionMutex<RefCell<f32>>,
}

impl Temperature {
    pub fn default() -> Self {
        Self::new(0.0)
    }

    fn new(value: f32) -> Self {
        Temperature {
            value: CriticalSectionMutex::new(RefCell::new(value)),
        }
    }
}

#[allow(dead_code)]
impl<'temperature> Temperature {
    pub async fn read_continuous(&self, i2c: &mut I2c<'temperature, Blocking>) -> ! {
        loop {
            Timer::after_micros(100).await;
            self.read(i2c).await;
            Timer::after_millis(100).await;
        }
    }

    pub fn current_reading(&self) -> f32 {
        self.value.lock(|value| *value.borrow())
    }

    async fn read(&self, i2c: &mut I2c<'temperature, Blocking>) {
        let write_buffer = [0b00000100];
        let mut read_buffer = [0u8; 2];
        let mut read_value: f32 = 0.0;

        match i2c.write_read(0b1100000, &write_buffer, &mut read_buffer) {
            Ok(_) => {
                let [upper, lower] = read_buffer;
                defmt::debug!("{:08b}", read_buffer);
                let raw: u16 = (upper as u16) << 8 | lower as u16;
                let raw = raw as i16;

                read_value = raw as f32 / 16.0;
            }
            Err(e) => {
                defmt::error!("error reading temperature: {}", e);
            }
        }

        self.value.lock(|value| {
            let mut value = value.borrow_mut();
            *value = read_value;
        })
    }
}
