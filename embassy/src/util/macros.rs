#![macro_use]

#[macro_export]
macro_rules! depanic {
    ($( $i:expr ),*) => {
        {
            defmt::error!($( $i ),*);
            panic!();
        }
    }
}

#[macro_export]
macro_rules! deassert {
    ($cond:expr) => {
        deassert!($cond, "assertion failed");
    };
    ($cond:expr, $msg:literal) => {
        {
            if !$cond {
                defmt::error!($msg);
                panic!();
            }
        }
    };
    ($cond:expr, $msg:literal, $( $i:expr ),*) => {
        {
            if !$cond {
                defmt::error!($msg, $( $i ),*);
                panic!();
            }
        }
    };
}
