//! The mini project on your PC: three days in the life of a self-watering
//! plant pot, with simulated hardware.
//!
//! ```text
//! cargo run -p no_std_and_embedded               # 72 hours
//! cargo run -p no_std_and_embedded -- 240        # 10 days
//! cargo run -p no_std_and_embedded --features alloc
//! ```
//!
//! This binary uses std (it prints). The library it drives is `no_std`:
//! on a real board, a ~30-line `main` would call the same `run_hour` with
//! the board's I2C, pin and delay instead of the simulated ones.

use no_std_and_embedded::controller::{Config, Controller, Event};
use no_std_and_embedded::report::{self, MAX_ENCODED_LEN, Status};
use no_std_and_embedded::run_hour;
use no_std_and_embedded::sensor::SoilSensor;
use no_std_and_embedded::sim::{SimDelay, SimPump, SimSoilChip, World};

fn main() {
    let hours: u32 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(72);

    let world = World::new(520, temperature_at(0));
    let mut sensor = SoilSensor::new(SimSoilChip::new(&world), SimDelay::default())
        .expect("the simulated SM-01 is always there");
    let mut controller = Controller::new(SimPump::new(&world), Config::HOUSE_PLANT)
        .expect("a simulated pin can't fail");
    let mut status = Status {
        uptime_hours: 0,
        moisture_permille: 0,
        temp_centi: 0,
        pump_on: false,
        waterings: 0,
        sensor_faults: 0,
    };

    println!("Self-watering pot, {hours} hours. Water below 30.0%, stop above 45.0%.");
    println!("A loose cable unplugs the sensor at 05:00-06:59 on day 3.\n");
    println!("time        moisture                      temp     pump");
    for hour in 0..hours {
        world.temp_centi.set(temperature_at(hour));
        world.sensor_unplugged.set((53..55).contains(&hour));
        let faults_before = status.sensor_faults;
        let event = run_hour(&mut sensor, &mut controller, &mut status).expect("infallible pin");
        let read_failed = status.sensor_faults > faults_before;
        if event != Event::Nothing || read_failed || hour % 6 == 0 {
            println!(
                "day {} {:02}:00  {} {:>3}.{}%  {:>5.2}C  {:<4} {}",
                hour / 24 + 1,
                hour % 24,
                bar(status.moisture_permille),
                status.moisture_permille / 10,
                status.moisture_permille % 10,
                f64::from(status.temp_centi) / 100.0,
                if status.pump_on { "ON" } else { "off" },
                if read_failed && event == Event::Nothing {
                    "<- sensor read failed (last good value shown)"
                } else {
                    describe(event)
                },
            );
        }
        world.advance_hour();
    }

    let line = report::status_line(&status).expect("80 bytes is enough");
    println!(
        "\nDebug console (formatted into a TextBuf<80>, no String):\n  {}",
        line.as_str()
    );

    let mut buffer = [0u8; MAX_ENCODED_LEN];
    let bytes = report::encode(&status, &mut buffer).expect("MAX_ENCODED_LEN fits any Status");
    println!(
        "Radio packet (postcard into a [u8; {MAX_ENCODED_LEN}] on the stack): {} bytes {:?}",
        bytes.len(),
        bytes
    );
    let decoded = report::decode(bytes).expect("we just encoded it");
    println!("Decoded at the base station: {decoded:?}");

    #[cfg(feature = "alloc")]
    println!(
        "With --features alloc, encode_vec gives the same bytes: {:?}",
        report::encode_vec(&status).expect("encodes")
    );

    let (chip, delay) = sensor.release();
    println!(
        "\nThe driver made {} I2C transactions and waited {} ms in total.",
        chip.transactions,
        delay.total_ns / 1_000_000
    );
    println!("The pump switched {} times.", controller.release().switches);
}

/// Warmest (26.00 °C) at 14:00, coolest (16.40 °C) at 02:00.
fn temperature_at(hour: u32) -> i16 {
    let from_two_pm = (hour % 24).abs_diff(14);
    let distance = from_two_pm.min(24 - from_two_pm) as i16;
    2600 - distance * 80
}

/// 20 characters: one block per 5%.
fn bar(permille: u16) -> String {
    let filled = usize::from(permille / 50).min(20);
    format!("{}{}", "#".repeat(filled), ".".repeat(20 - filled))
}

fn describe(event: Event) -> &'static str {
    match event {
        Event::Nothing => "",
        Event::PumpOn => "<- dry: pump on",
        Event::PumpOffWet => "<- wet enough: pump off, soaking",
        Event::PumpOffTimeout => "<- ran too long: pump off",
        Event::PumpOffSensorFault => "<- SENSOR FAULT: pump off",
        Event::Ready => "<- soaked in: watching again",
    }
}
