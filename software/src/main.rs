#![no_std]
#![no_main]

mod common;

use core::future;

use embassy_executor::Spawner;
use esp_hal::{
    clock::CpuClock,
    spi::master::{Config, Spi},
    timer::timg::TimerGroup,
};

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let spi = Spi::new(peripherals.SPI2, Config::default()).unwrap();
    let mut spi = spi
        .with_cs(peripherals.GPIO4)
        .with_sck(peripherals.GPIO6)
        .with_miso(peripherals.GPIO5);

    let mut buffer: [u8; 4] = [0; 4];
    match spi.read(&mut buffer) {
        Ok(_) => {
            for byte in buffer {
                defmt::info!("{:08b}", byte);
            }
        }
        Err(e) => {
            defmt::error!("{}", e);
        }
    }

    future::pending().await
}
