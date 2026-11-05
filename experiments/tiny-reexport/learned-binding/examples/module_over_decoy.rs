pub mod storage { pub struct Error; }
pub mod protocol { pub struct Error; }

use self::protocol::Error as NetworkError;
use self::storage as backend;

pub fn make() -> backend::Error { loop {} }
