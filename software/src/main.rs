#![no_std]
#![no_main]

mod led;
mod profile;
mod temperature;

use crate::{
    led::{create_channel, green_led, off, red_led},
    profile::Profile,
    temperature::Temperature,
};
use defmt::println;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_time::{Instant, Timer};
use esp_backtrace as _;
use esp_hal::{
    Async,
    gpio::Output,
    i2c::master::{Config, I2c},
    interrupt::software::SoftwareInterruptControl,
    rmt::{Channel, Tx},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_rtos::main;

esp_bootloader_esp_idf::esp_app_desc!();

async fn adjust_temperature_wrapper<'a>(
    mut output: Output<'a>,
    mut channel: Channel<'a, Async, Tx>,
    temperature: &Temperature,
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
            let _ = red_led(&mut channel).await;

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

#[main]
async fn main(_spawner: Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 64 * 1024);

    defmt::info!("starting up...");
    let peripherals = esp_hal::init(Default::default());
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    let sda = peripherals.GPIO4;
    let scl = peripherals.GPIO5;

    let mut i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_hz(40000)),
    )
    .unwrap()
    .with_scl(scl)
    .with_sda(sda);

    let mut led_channel = create_channel(peripherals.RMT, peripherals.GPIO8).await;
    let _ = off(&mut led_channel).await;

    let output = Output::new(
        peripherals.GPIO10,
        esp_hal::gpio::Level::Low,
        Default::default(),
    );

    let temperature = Temperature::default();

    join(
        temperature.read_continuous(&mut i2c),
        adjust_temperature_wrapper(output, led_channel, &temperature),
    )
    .await;

    loop {}
}
