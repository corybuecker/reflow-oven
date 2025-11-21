use core::cell::RefCell;
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_time::Timer;
use esp_hal::{Blocking, spi::master::Spi};

#[allow(dead_code)]
pub struct Temperature {
    value: CriticalSectionMutex<RefCell<f32>>,
    offset: CriticalSectionMutex<RefCell<f32>>,
}

impl Temperature {
    pub fn default() -> Self {
        Self::new(0.0)
    }

    fn new(value: f32) -> Self {
        Temperature {
            value: CriticalSectionMutex::new(RefCell::new(value)),
            offset: CriticalSectionMutex::new(RefCell::new(0.0)),
        }
    }
}

#[allow(dead_code)]
impl<'temperature> Temperature {
    pub async fn read_continuous(&self, spi: &mut Spi<'temperature, Blocking>) -> ! {
        self.initialize_offset(spi);

        loop {
            self.read(spi);
            Timer::after_millis(100).await;
        }
    }

    pub fn current_reading(&self) -> f32 {
        self.value.lock(|value| *value.borrow())
    }

    fn initialize_offset(&self, spi: &mut Spi<'temperature, Blocking>) {
        let mut buffer: [u8; 4] = [0; 4];

        match spi.read(&mut buffer) {
            Ok(_) => {
                let [b0, b1, b2, b3] = buffer;

                let raw_data: u32 =
                    ((b0 as u32) << 24) | ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

                let cold_junction_temperature = (raw_data >> 4) & 0b0000_1111_1111_1111;
                let thermocouple_temperature = (raw_data >> 18) & 0b0011_1111_1111_1111;

                let cold_junction_temperature = cold_junction_temperature as f32 * 0.0625;
                let thermocouple_temperature = thermocouple_temperature as f32 * 0.25;

                defmt::info!(
                    "offset is {}",
                    cold_junction_temperature - thermocouple_temperature
                );

                self.offset.lock(|value| {
                    let mut v = value.borrow_mut();
                    *v = cold_junction_temperature - thermocouple_temperature;
                })
            }
            Err(e) => {
                defmt::error!("{}", e);
            }
        }
    }

    fn read(&self, spi: &mut Spi<'temperature, Blocking>) {
        let mut buffer: [u8; 4] = [0; 4];
        let mut read_value: f32 = 0.0;

        match spi.read(&mut buffer) {
            Ok(_) => {
                let [b0, b1, b2, b3] = buffer;

                let raw_data: u32 =
                    ((b0 as u32) << 24) | ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
                // defmt::debug!("{:032b}", raw_data);
                let thermocouple_temperature = (raw_data >> 18) & 0b0011_1111_1111_1111;

                #[allow(unused)]
                let junction_temperature = (raw_data >> 4) & 0b0000_1111_1111_1111;

                let mut local_offset = 0.0;
                self.offset.lock(|offset| {
                    let offset = offset.borrow();
                    local_offset = *offset;
                });

                read_value = thermocouple_temperature as f32 * 0.25;
                // read_value = (thermocouple_temperature as f32 * 0.25) + local_offset;
                // defmt::debug!("thermocouple_temperature {}", read_value);
            }
            Err(e) => {
                defmt::error!("{}", e);
            }
        }

        self.value.lock(|value| {
            let mut value = value.borrow_mut();
            *value = read_value;
        })
    }
}
