use crate::bindings;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub enum OptionUnit {
    None,
    Percent,
}

impl From<OptionUnit> for VPPluginAPI_OptionUnit {
    fn from(unit: OptionUnit) -> Self {
        match unit {
            OptionUnit::None => VPPluginAPI_OptionUnit_NONE,
            OptionUnit::Percent => VPPluginAPI_OptionUnit_PERCENT,
        }
    }
}
