// v151 Bug Reproduction: Rust Qualified Names with ::
// Rust uses :: for module paths, crate references, and trait methods
// Expected: Edges should be created, keys should be properly sanitized

use std::collections::HashMap;
use std::io::{self, Write};

mod my_app {
    pub mod services {
        use super::models::User;

        pub struct UserService;

        impl UserService {
            pub fn create_user(&self) -> User {
                // Edge 1: Qualified type from std
                let mut map: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();

                // Edge 2: Crate-level qualified path
                let config = crate::my_app::config::get_settings();

                // Edge 3: Fully qualified trait method
                let output = <String as std::fmt::Display>::fmt;

                // Edge 4: Nested module instantiation
                let user = super::models::User::new("test");

                self.process_user(&user);
                user
            }

            fn process_user(&self, user: &super::models::User) {
                // Edge 5: Qualified function call
                std::io::stdout().write_all(b"Processing user\n").unwrap();

                // Edge 6: Absolute path from crate root
                crate::my_app::events::publish("user.created");

                // Edge 7: Type-qualified method call (turbofish)
                let parsed = str::parse::<i32>("42").unwrap();
            }
        }
    }

    pub mod models {
        pub struct User {
            pub name: String,
        }

        impl User {
            pub fn new(name: &str) -> Self {
                User { name: name.to_string() }
            }
        }
    }

    pub mod config {
        pub fn get_settings() -> String {
            String::from("settings")
        }
    }

    pub mod events {
        pub fn publish(event: &str) {
            println!("Event: {}", event);
        }
    }
}

fn main() {
    // Edge 8: Absolute crate path usage
    let service = crate::my_app::services::UserService;
    let user = service.create_user();

    // Edge 9: std library qualified call
    std::process::exit(0);
}
