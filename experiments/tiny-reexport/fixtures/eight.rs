pub fn make() -> A {
    H
}

pub use B as A;
pub use C as B;
pub use D as C;
pub use E as D;
pub use F as E;
pub use G as F;
pub use H as G;
pub struct H;
