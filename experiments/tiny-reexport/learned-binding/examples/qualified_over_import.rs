pub mod journal { pub struct Record; }
pub mod network { pub struct Record; }

use self::journal::Record;

pub fn make() -> network::Record { loop {} }
