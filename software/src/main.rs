#![no_std]
#![no_main]

mod common;
mod led;
mod profile;
mod temperature;

use crate::{
    led::{blue_led, create_channel, green_led, off, red_led},
    profile::Profile,
    temperature::Temperature,
};
use core::future;
use defmt::println;
use embassy_executor::Spawner;
use embassy_futures::join::join;
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
async fn main(_spawner: Spawner) -> ! {
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

    let output = Output::new(
        peripherals.GPIO7,
        Level::Low,
        OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::PushPull),
    );
    let mut channel = create_channel(peripherals.RMT, peripherals.GPIO8).await;

    let _ = blue_led(&mut channel).await;
    Timer::after_secs(5).await;

    join(
        read_temperature_wrapper(temperature, spi),
        adjust_temperature_wrapper(output, channel, temperature),
    )
    .await;

    future::pending().await
}

async fn read_temperature_wrapper(
    temperature: &'static Temperature,
    mut spi_peripheral: Spi<'static, Blocking>,
) -> () {
    temperature.read_continuous(&mut spi_peripheral).await
}

async fn adjust_temperature_wrapper(
    mut output: Output<'static>,
    mut channel: Channel<'static, Async, Tx>,
    temperature: &'static Temperature,
) -> () {
    let mut profile = Profile::new();

    if temperature.current_reading() < profile.heatsoak_temperature_target {
        while temperature.current_reading() < profile.heatsoak_temperature_target {
            defmt::info!(
                "heatsoak_current_temperature={}",
                temperature.current_reading()
            );

            output.set_high();

            #[allow(unused)]
            red_led(&mut channel).await;

            Timer::after_secs(1).await;
        }
        output.set_low();

        #[allow(unused)]
        off(&mut channel).await;
    }

    let program_start = Instant::now();
    println!("runtime,desired_temperature,current_temperature,control_output",);

    loop {
        let runtime = Instant::now() - program_start;
        let runtime = runtime.as_millis() as f32 / 1000.0;

        let current_temperature = temperature.current_reading();
        let desired_temperature = profile.desired_temperature(runtime);
        let control_output = profile.control_output(runtime, current_temperature);

        println!(
            "{},{},{},{}",
            runtime, desired_temperature, current_temperature, control_output
        );

        if runtime > profile.cooling_time_target {
            output.set_low();
            let _ = green_led(&mut channel).await;
        } else if control_output > 0.0 {
            output.set_high();
            let _ = red_led(&mut channel).await;
        } else {
            output.set_low();
            let _ = off(&mut channel).await;
        }

        Timer::after_millis(50).await;
    }
}
