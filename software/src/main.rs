#![no_std]
#![no_main]

mod common;
mod led;
mod profile;
mod temperature;

use crate::{
    led::{create_channel, off, red_led},
    profile::Profile,
    temperature::Temperature,
};
use core::future;
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
use esp_hal::{
    Async, Blocking,
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    rmt::{Channel, Tx},
    spi::master::{Config, Spi},
    timer::timg::TimerGroup,
};
use static_cell::StaticCell;

static TEMPERATURE: StaticCell<Temperature> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let spi = Spi::new(peripherals.SPI2, Config::default()).unwrap();

    let spi = spi
        .with_cs(peripherals.GPIO4)
        .with_miso(peripherals.GPIO5)
        .with_sck(peripherals.GPIO6);

    let temperature = TEMPERATURE.init(Temperature::default());

    let _ = spawner.spawn(read_temperature_wrapper(temperature, spi));

    let output = Output::new(
        peripherals.GPIO7,
        Level::Low,
        OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull),
    );
    let mut channel = create_channel(peripherals.RMT, peripherals.GPIO8).await;

    Timer::after_millis(500).await;
    let _ = off(&mut channel).await;
    Timer::after_secs(5).await;

    let _ = spawner.spawn(adjust_temperature_wrapper(output, channel, temperature));

    future::pending().await
}

#[embassy_executor::task]
async fn read_temperature_wrapper(
    temperature: &'static Temperature,
    mut spi_peripheral: Spi<'static, Blocking>,
) -> () {
    temperature.read_continuous(&mut spi_peripheral).await
}

#[embassy_executor::task]
async fn adjust_temperature_wrapper(
    mut output: Output<'static>,
    mut channel: Channel<'static, Async, Tx>,
    temperature: &'static Temperature,
) -> () {
    let profile = Profile::new();
    let program_start = Instant::now();

    loop {
        let runtime = Instant::now() - program_start;
        let desired_temperature = profile.get_target(runtime.as_millis() as f32 / 1000.0);
        let current_temperature = temperature.current_reading();

        defmt::info!(
            "desired={} current={}",
            desired_temperature,
            current_temperature
        );

        if current_temperature > desired_temperature {
            output.set_low();
            let _ = off(&mut channel).await;
        } else {
            output.set_high();
            let _ = red_led(&mut channel).await;
        }

        Timer::after_millis(50).await;
    }
}
