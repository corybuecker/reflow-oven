use core::cell::RefCell;
use embassy_stm32::{
    gpio::Output,
    mode::Async,
    spi::{Spi, mode::Master},
};
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_time::Timer;

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
    pub async fn read_continuous(
        &self,
        cs: &mut Output<'temperature>,
        spi: &mut Spi<'temperature, Async, Master>,
    ) -> ! {
        loop {
            cs.set_low();
            Timer::after_micros(100).await;
            self.read(spi).await;
            cs.set_high();
            Timer::after_millis(100).await;
        }
    }

    pub fn current_reading(&self) -> f32 {
        self.value.lock(|value| *value.borrow())
    }

    async fn read(&self, spi: &mut Spi<'temperature, Async, Master>) {
        let mut buffer: [u8; 4] = [0; 4];
        let mut read_value: f32 = 0.0;

        match spi.read(&mut buffer).await {
            Ok(_) => {
                let [b0, b1, b2, b3] = buffer;
                // defmt::debug!("thermocouple_temperature {}", buffer);

                let raw_data: u32 =
                    ((b0 as u32) << 24) | ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
                // defmt::debug!("{:032b}", raw_data);
                let thermocouple_temperature = (raw_data >> 18) & 0b0011_1111_1111_1111;

                #[allow(unused)]
                let junction_temperature = (raw_data >> 4) & 0b0000_1111_1111_1111;

                read_value = thermocouple_temperature as f32 * 0.25;
                // read_value = (thermocouple_temperature as f32 * 0.25) + local_offset;
                // defmt::debug!("thermocouple_temperature {}", read_value);
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
