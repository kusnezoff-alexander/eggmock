mod network;
mod macros;

pub use paste;
pub use egg;
pub use libc;
pub use seq_macro;

pub use network::*;

define_network! {
    pub enum "mig" = Mig {
        "t" = True(0),
        "f" = False(0),
        "!" = Not(1),
        "m" = Maj(3)
    }
}

define_network! {
    pub enum "aig" = Aig {
        "t" = True(0),
        "f" = False(0),
        "!" = Not(1),
        "a" = And(2)
    }
}
