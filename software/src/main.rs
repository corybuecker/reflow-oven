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
        .with_cs(peripherals.GPIO16)
        .with_sck(peripherals.GPIO7)
        .with_miso(peripherals.GPIO2);

    let mut buffer: [u8; 64] = [0; 64];
    match spi.read(&mut buffer) {
        Ok(_) => {
            defmt::error!("{}", &buffer);
        }
        Err(e) => {
            defmt::error!("{}", e);
        }
    }

    future::pending().await
}
