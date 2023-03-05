use muscript_parsing_derive::{Parse, PredictiveParse};

use crate::ast::default_properties::DefaultPropertiesBlock;

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
