use channel::dnp3::master::{
    AssociationConfig, AssociationHandler, AssociationInformation, Classes, EventClasses,
};

pub struct NoopAssociationHandler;
impl AssociationHandler for NoopAssociationHandler {}

pub struct NoopAssociationInformation;
impl AssociationInformation for NoopAssociationInformation {}

/// Master stack config:
///   disableUnsolOnStartup = False        -> don't send "disable unsolicited"
///   integrityOnEventOverflowIIN = False  -> no automatic integrity poll on overflow
pub fn association_config() -> AssociationConfig {
    let mut config = AssociationConfig::new(
        EventClasses::none(), // disable unsolicited on startup: none
        EventClasses::all(), // enable unsolicited on startup: none (outstation has it off)
        Classes::all(),       // startup integrity scan (the opendnp3 default)
        EventClasses::all(), // no event scan when "events available" IIN is set
    );
    config.auto_integrity_scan_on_buffer_overflow = false;
    config
}