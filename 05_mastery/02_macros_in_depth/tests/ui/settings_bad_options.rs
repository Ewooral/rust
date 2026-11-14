use macros_in_depth::Settings;

#[derive(Settings)]
struct Server {
    #[setting(defualt = "8080")]
    port: u16,
    #[setting(default = 4)]
    workers: u8,
}

fn main() {}
