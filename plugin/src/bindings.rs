use crate::bindings;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub enum OptionUnit {
    None,
    Percent,
}

impl From<OptionUnit> for VPXPluginAPI_OptionUnit {
    fn from(unit: OptionUnit) -> Self {
        match unit {
            OptionUnit::None => VPXPluginAPI_OptionUnit_NONE,
            OptionUnit::Percent => VPXPluginAPI_OptionUnit_PERCENT,
        }
    }
}
