pub mod reader { pub struct Ticket; }
pub mod writer { pub struct Ticket; }

use self::reader::Ticket as Choice;
use self::writer::Ticket as Decoy;

pub fn make() -> nonexistent::Choice { loop {} }
