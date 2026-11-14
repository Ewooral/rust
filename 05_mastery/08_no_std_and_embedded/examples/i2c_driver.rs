//! An I2C driver, written against embedded-hal, talking to a simulated chip.
//! Run: cargo run -p no_std_and_embedded --example i2c_driver
//!
//! `LoggingBus` wraps ANY I2c bus and prints every operation: because the
//! driver only knows the trait, we can slip it in between without touching
//! the driver. (Logic analysers do the same job with real wires.)

use embedded_hal::i2c::{ErrorType, I2c, Operation};

use no_std_and_embedded::sensor::{self, SoilSensor};
use no_std_and_embedded::sim::{SimDelay, SimSoilChip, World};

/// Wraps another bus and prints what goes over it.
struct LoggingBus<B> {
    inner: B,
}

impl<B: I2c> ErrorType for LoggingBus<B> {
    type Error = B::Error;
}

impl<B: I2c> I2c for LoggingBus<B> {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), B::Error> {
        let result = self.inner.transaction(address, operations);
        let mut line = format!("    0x{address:02X}:");
        for operation in operations.iter() {
            match operation {
                Operation::Write(bytes) => line += &format!(" write {bytes:02X?}"),
                Operation::Read(buffer) => line += &format!(" read {buffer:02X?}"),
            }
        }
        if let Err(err) = &result {
            line += &format!("  -> {err:?}");
        }
        println!("{line}");
        result
    }
}

fn main() {
    // The "physical world" the chip measures: 43.1% moisture, 21.50 °C.
    let world = World::new(431, 2150);
    let bus = LoggingBus {
        inner: SimSoilChip::new(&world),
    };

    println!("SoilSensor::new checks the chip ID:");
    let mut sensor = SoilSensor::new(bus, SimDelay::default()).unwrap();

    println!("\nread(): start a measurement, wait 5 ms, read 4 bytes from 0x02:");
    let reading = sensor.read().unwrap();
    println!("  -> {reading:?}");

    println!("\nThe sensor's cable comes loose:");
    world.sensor_unplugged.set(true);
    println!("  -> {:?}", sensor.read());
    world.sensor_unplugged.set(false);

    // The driver OWNS the bus. To use the bus for something else, take it back.
    let (bus, delay) = sensor.release();
    println!(
        "\nreleased the bus after {} transactions; the driver waited {} ms",
        bus.inner.transactions,
        delay.total_ns / 1_000_000
    );

    println!("\nA different chip at the same address:");
    let mut impostor = SimSoilChip::new(&world);
    impostor.chip_id = 0x77;
    let result = SoilSensor::new(LoggingBus { inner: impostor }, SimDelay::default());
    println!("  -> {:?}", result.map(|_| "ok"));

    println!(
        "\nThe datasheet in code: address 0x{:02X}, chip ID 0x{:02X}, {} ms per measurement",
        sensor::ADDRESS,
        sensor::CHIP_ID,
        sensor::MEASURE_MS
    );
}
