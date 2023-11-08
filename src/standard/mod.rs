mod config;
mod gasometer;
mod invoker;

pub use self::config::Config;
pub use self::gasometer::Gasometer;
pub use self::invoker::Invoker;

pub type Machine = crate::Machine<crate::RuntimeState>;
pub type Efn<H> = crate::Efn<crate::RuntimeState, H, crate::Opcode>;
pub type Etable<H, F = Efn<H>> = crate::Etable<crate::RuntimeState, H, crate::Opcode, F>;
pub type GasedMachine<'config> = crate::GasedMachine<crate::RuntimeState, Gasometer<'config>>;
