// Rust qualified names using ::
mod core {
    pub struct Config {}
}

mod services {
    use crate::core::Config;

    pub fn init() {
        let _cfg = crate::core::Config {};
    }
}

fn main() {
    std::collections::HashMap::<String, i32>::new();
}
