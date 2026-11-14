//! mockall: generate the mock instead of writing it by hand.
//!
//! Run:  cargo run -p advanced_testing --example mockall_basics
//! Test: cargo test -p advanced_testing --example mockall_basics
//! See a mock fail: cargo test -p advanced_testing --example mockall_basics -- --ignored

#[cfg(test)]
use mockall::automock;

/// `#[cfg_attr(test, automock)]` adds `MockWeatherApi` in test builds only,
/// so mockall (a dev-dependency) is never compiled into the real program.
#[cfg_attr(test, automock)]
trait WeatherApi {
    fn temperature(&self, city: &str) -> Result<i32, String>;
}

/// The code under test.
fn advice(api: &dyn WeatherApi, city: &str) -> String {
    match api.temperature(city) {
        Ok(t) if t >= 25 => format!("{city}: {t}°C, take water"),
        Ok(t) if t <= 5 => format!("{city}: {t}°C, take a coat"),
        Ok(t) => format!("{city}: {t}°C, enjoy"),
        Err(_) => format!("{city}: no forecast, take an umbrella just in case"),
    }
}

struct FixedWeather;
impl WeatherApi for FixedWeather {
    fn temperature(&self, city: &str) -> Result<i32, String> {
        Ok(if city == "Accra" { 31 } else { 12 })
    }
}

fn main() {
    println!("{}", advice(&FixedWeather, "Accra"));
    println!("{}", advice(&FixedWeather, "Leeds"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::Sequence;
    use mockall::predicate::eq;

    #[test]
    fn hot_days_mean_water() {
        let mut api = MockWeatherApi::new();
        api.expect_temperature() // expect a call to `temperature`...
            .with(eq("Accra")) // ...with this argument...
            .times(1) // ...exactly once...
            .returning(|_| Ok(31)); // ...and answer this.

        assert_eq!(advice(&api, "Accra"), "Accra: 31°C, take water");
    } // `api` is dropped here, and checks that times(1) was met

    #[test]
    fn errors_mean_an_umbrella() {
        let mut api = MockWeatherApi::new();
        api.expect_temperature()
            .returning(|_| Err("timeout".into()));

        assert!(advice(&api, "Leeds").contains("umbrella"));
    }

    #[test]
    fn answers_can_depend_on_the_arguments() {
        let mut api = MockWeatherApi::new();
        api.expect_temperature()
            .returning(|city| Ok(if city.starts_with('O') { -3 } else { 18 }));

        assert!(advice(&api, "Oslo").ends_with("take a coat"));
        assert!(advice(&api, "Lisbon").ends_with("enjoy"));
    }

    #[test]
    fn a_sequence_checks_the_order_of_calls() {
        let mut seq = Sequence::new();
        let mut api = MockWeatherApi::new();
        api.expect_temperature()
            .with(eq("Kumasi"))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(29));
        api.expect_temperature()
            .with(eq("Tamale"))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(35));

        advice(&api, "Kumasi");
        advice(&api, "Tamale");
    }

    // Run with -- --ignored to see what a broken expectation looks like.
    #[test]
    #[ignore = "fails on purpose, to show mockall's message"]
    fn expected_two_calls_but_got_one() {
        let mut api = MockWeatherApi::new();
        api.expect_temperature().times(2).returning(|_| Ok(20));
        advice(&api, "Accra");
    }
}
