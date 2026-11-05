pub mod frames { pub struct Header; }
pub mod files { pub struct Header; }

use self::files::Header as OnDisk;
use self::frames::Header as OnWire;

pub fn make() -> OnWire { loop {} }
