use crate::{cst::default_properties::DefaultPropertiesBlock, Parse, PredictiveParse};

keyword!(KDefaultProperties = "defaultproperties");
keyword!(KStructDefaultProperties = "structdefaultproperties");

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemDefaultProperties {
    pub keyword: KDefaultProperties,
    pub block: DefaultPropertiesBlock,
}

#[derive(Debug, Clone, Parse, PredictiveParse)]
pub struct ItemStructDefaultProperties {
    pub keyword: KStructDefaultProperties,
    pub block: DefaultPropertiesBlock,
}
