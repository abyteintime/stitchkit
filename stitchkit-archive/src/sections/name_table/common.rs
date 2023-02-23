use crate::name::ArchivedName;

use super::builder::{Error, NameTableBuilder};

pub struct CommonNames {
    pub none: ArchivedName,
    pub package: CommonPackageNames,
    pub class: CommonClassNames,
}

pub struct CommonPackageNames {
    pub core: ArchivedName,
}

pub struct CommonClassNames {
    pub object: ArchivedName,
    pub class: ArchivedName,
    pub package: ArchivedName,
}

impl CommonNames {
    pub fn get_or_insert_into(builder: &mut NameTableBuilder) -> Result<Self, Error> {
        Ok(Self {
            none: builder.get_or_insert("None")?,
            package: CommonPackageNames::get_or_insert_into(builder)?,
            class: CommonClassNames::get_or_insert_into(builder)?,
        })
    }
}

impl CommonPackageNames {
    pub fn get_or_insert_into(builder: &mut NameTableBuilder) -> Result<Self, Error> {
        Ok(Self {
            core: builder.get_or_insert("Core")?,
        })
    }
}

impl CommonClassNames {
    pub fn get_or_insert_into(builder: &mut NameTableBuilder) -> Result<Self, Error> {
        Ok(Self {
            object: builder.get_or_insert("Object")?,
            class: builder.get_or_insert("Class")?,
            package: builder.get_or_insert("Package")?,
        })
    }
}
