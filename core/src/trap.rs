pub enum Trap {
    CodeEnd,
    NotSupported,
    External(ExternalOpcode),
}
