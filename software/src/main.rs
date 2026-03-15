#![no_main]
#![no_std]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal::{delay::DelayNs, i2c::I2c};
use panic_probe as _;
use stm32h5xx_hal::{delay::Delay, pac, prelude::*, time::Hertz};

// MCP960X base address: 0b110_0000 = 0x60
// Lower 3 bits (A2:A0) are set by the ADDR pin voltage.
// With ADDR tied to GND, the address is 0x60.
const MCP960X_BASE_ADDR: u8 = 0b01100000;

// Register pointers (Table 5-1)
#[allow(unused)]
const REG_DEVICE_ID: u8 = 0x20;

/// Read Device ID register (1 byte) to verify communication.
/// MCP9600 returns 0x40, MCP9601 returns 0x41.
#[allow(unused)]
fn read_device_id<I: I2c>(i2c: &mut I, addr: u8) -> Result<u8, I::Error> {
    i2c.write(addr, &[REG_DEVICE_ID])?;
    let mut buf = [0u8; 2]; // ID byte + revision byte
    i2c.read(addr, &mut buf)?;
    Ok(buf[0])
}

/// Decode a 16-bit two-byte read from TC (cold-junction) register into degrees Celsius.
/// Format: 12-bit signed integer + 4-bit fractional. Range: -40°C to +125°C.
#[allow(unused)]
fn decode_cold_junction(upper: u8, lower: u8) -> f32 {
    // TC register uses only 12 bits total (upper[7:4] unused for integer MSBs
    // at this range), but the decode formula is the same shape.
    if (upper & 0x80) != 0 {
        ((upper as f32) * 16.0 + (lower as f32) / 16.0) - 4096.0
    } else {
        (upper as f32) * 16.0 + (lower as f32) / 16.0
    }
}

fn decode_hot_junction(upper: u8, lower: u8) -> f32 {
    // The value is a 13.4 signed fixed-point number.
    // upper[7] = sign bit, upper[6:0] = integer MSBs, lower[7:4] = integer LSBs,
    // lower[3:0] = fractional part (0.0625°C per LSB)
    let _raw = ((upper as i16) << 8) | (lower as i16);

    defmt::debug!("{:08b}{:08b}", upper, lower);

    if (upper & 0x80) != 0 {
        // Negative temperature: extend sign and subtract 4096
        // Per datasheet pseudo-code: (UpperByte * 16 + LowerByte / 16) - 4096
        ((upper as f32) * 16.0 + (lower as f32) / 16.0) - 4096.0
    } else {
        // Positive temperature
        (upper as f32) * 16.0 + (lower as f32) / 16.0
    }
}

/// Read a 2-byte temperature register from the MCP960X.
/// Protocol: write register pointer, then read 2 bytes (Figure 4-3).
fn read_temp_register<I: I2c>(
    i2c: &mut I,
    addr: u8,
    reg: u8,
    buf: &mut [u8; 2],
) -> Result<(), I::Error> {
    // Step 1: write the register pointer
    i2c.write(addr, &[reg])?;
    // Step 2: read 2 bytes (device issues clock stretching per datasheet Section 4.1.7)
    i2c.read(addr, buf)?;

    Ok(())
}
const REG_STATUS: u8 = 0x04;
#[allow(unused)]
const REG_TD: u8 = 0x01; // Junctions temperature delta

#[derive(defmt::Format)]
struct Status {
    burst_complete: bool,
    th_updated: bool,
    /// MCP9601 only: short-circuit fault detected on thermocouple
    short_circuit: bool,
    input_range: bool, // bit 4 — if SET, TH and TΔ are NOT updated

    /// MCP9601 only: open-circuit or input out-of-range fault
    open_circuit: bool,
    alert4: bool,
    alert3: bool,
    alert2: bool,
    alert1: bool,
}

impl Status {
    fn from_byte(b: u8) -> Self {
        Status {
            burst_complete: (b & 0x80) != 0, // bit 7 (MCP9600) / SC on MCP9601
            th_updated: (b & 0x40) != 0,     // bit 6 (MCP9600) / OC on MCP9601
            // For MCP9601 the layout is:
            //   bit 7 = SC, bit 6 = OC, bit 5 = unused, bit 4 = Input Range,
            //   bits 3:0 = Alert 4:1 status
            short_circuit: (b & 0x80) != 0, // bit 7 — SC (MCP9601)
            open_circuit: (b & 0x40) != 0,  // bit 6 — OC / Input Range (MCP9601)
            alert4: (b & 0x08) != 0,
            input_range: (b & 0x10) != 0, // <-- the missing bit

            alert3: (b & 0x04) != 0,
            alert2: (b & 0x02) != 0,
            alert1: (b & 0x01) != 0,
        }
    }
}
#[allow(unused)]
const REG_TC: u8 = 0x02; // Cold-junction temperature (ambient)
const REG_TH: u8 = 0x00; // Hot-junction temperature (thermocouple + cold-junction compensated)

fn read_status<I: I2c>(i2c: &mut I, addr: u8) -> Result<Status, I::Error> {
    i2c.write(addr, &[REG_STATUS])?;
    let mut buf = [0u8; 1];
    i2c.read(addr, &mut buf)?;
    Ok(Status::from_byte(buf[0]))
}
#[allow(unused)]
fn read_device_config<I: I2c>(i2c: &mut I, addr: u8) -> Result<u8, I::Error> {
    i2c.write(addr, &[0x06])?;
    let mut buf = [0u8; 1];
    i2c.read(addr, &mut buf)?;
    Ok(buf[0])
}
#[allow(unused)]
fn read_sensor_config<I: I2c>(i2c: &mut I, addr: u8) -> Result<u8, I::Error> {
    i2c.write(addr, &[0x05])?;
    let mut buf = [0u8; 1];
    i2c.read(addr, &mut buf)?;
    Ok(buf[0])
}

fn read_raw_adc<I: I2c>(i2c: &mut I, addr: u8) -> Result<i32, I::Error> {
    i2c.write(addr, &[0x03])?;
    let mut buf = [0u8; 3];
    i2c.read(addr, &mut buf)?;
    // 18-bit signed value, sign-extended from bit 17
    let raw = ((buf[0] as i32) << 16) | ((buf[1] as i32) << 8) | (buf[2] as i32);
    // Sign extend from bit 17
    let raw = if (raw & 0x20000) != 0 {
        raw | 0xFFFC0000u32 as i32
    } else {
        raw
    };
    Ok(raw)
}

#[entry]
fn main() -> ! {
    let core_peripherals = cortex_m::Peripherals::take().unwrap();
    let device_peripherals = pac::Peripherals::take().unwrap();

    let power = device_peripherals.PWR.constrain();
    let power_configuration = power.vos0().freeze();

    let rcc = device_peripherals.RCC.constrain();
    let ccdr = rcc
        .sys_ck(Hertz::MHz(250))
        .freeze(power_configuration, &device_peripherals.SBS);

    let mut delay = Delay::new(core_peripherals.SYST, &ccdr.clocks);

    let gpiob = device_peripherals.GPIOB.split(ccdr.peripheral.GPIOB);

    let scl = gpiob.pb6.into_alternate_open_drain::<4>();
    let sda = gpiob.pb7.into_alternate_open_drain::<4>();

    let mut i2c =
        device_peripherals
            .I2C1
            .i2c((scl, sda), 45.kHz(), ccdr.peripheral.I2C1, &ccdr.clocks);

    // Adjust A2:A0 to match your ADDR pin wiring (0x60–0x67)
    let device_addr = MCP960X_BASE_ADDR; // 0x60, ADDR = GND

    defmt::info!("test");
    loop {
        // Verify communication by reading Device ID
        // match read_device_id(&mut i2c, device_addr) {
        //     Ok(id) => defmt::info!("MCP960X Device ID: {:#04x} (expect 0x40 or 0x41)", id),
        //     Err(_) => defmt::error!("Failed to read Device ID — check wiring and address"),
        // }
        let mut buf = [0u8; 2];

        match read_raw_adc(&mut i2c, device_addr) {
            Ok(t) => {
                defmt::debug!("{}", t);
            }
            Err(_e) => {
                defmt::debug!("Error {}", 2);
            }
        }

        match read_temp_register(&mut i2c, device_addr, REG_TH, &mut buf) {
            Ok(()) => {
                let tc = decode_hot_junction(buf[0], buf[1]);
                defmt::info!("TH raw bytes: {:#04x} {:#04x}", buf[0], buf[1]);
                defmt::info!("TH: {}", tc);
            }
            Err(_) => defmt::error!("Failed to read TC register"),
        }

        // match read_temp_register(&mut i2c, device_addr, 0x06, &mut buf) {
        //     Ok(()) => {
        //         // let tc = decode_hot_junction(buf[0], buf[1]);
        //         // defmt::info!("TC (hot junction): {} °C", tc);
        //     }
        //     Err(_) => defmt::error!("Failed to read TC register"),
        // }

        match read_status(&mut i2c, device_addr) {
            Ok(status) => defmt::info!(
                "STATUS: OC={} SC={} InputRange={}",
                status.open_circuit,
                status.short_circuit,
                status.input_range
            ),
            Err(_) => defmt::error!("Failed to read STATUS"),
        }

        // match read_device_config(&mut i2c, device_addr) {
        //     Ok(cfg) => defmt::info!("Device config: {:#010b}", cfg),
        //     Err(_) => defmt::error!("Failed to read device config"),
        // }

        // match read_sensor_config(&mut i2c, device_addr) {
        //     Ok(cfg) => defmt::info!("Sensor config: {:#010b}", cfg),
        //     Err(_) => defmt::error!("Failed to read sensor config"),
        // }

        // match read_status(&mut i2c, device_addr) {
        //     Ok(status) => {
        //         if status.open_circuit {
        //             defmt::warn!("STATUS: Open-circuit or input out-of-range fault!");
        //         }
        //         if status.short_circuit {
        //             defmt::warn!("STATUS: Short-circuit fault detected!");
        //         }
        //         defmt::info!(
        //             "STATUS: OC={} SC={} alerts=[4:{} 3:{} 2:{} 1:{}]",
        //             status.open_circuit,
        //             status.short_circuit,
        //             status.alert4,
        //             status.alert3,
        //             status.alert2,
        //             status.alert1,
        //         );
        //     }
        //     Err(_) => defmt::error!("Failed to read STATUS register"),
        // }

        delay.delay_ms(500);
    }
}
