use std::collections::HashMap;

use muscript_foundation::ident::CaseInsensitive;

use crate::{FunctionId, VarId};

#[derive(Debug, Default)]
pub struct ClassNamespace {
    pub all_var_names: Option<Vec<String>>,
    pub vars: HashMap<CaseInsensitive<String>, Option<VarId>>,

    pub all_function_names: Option<Vec<String>>,
    pub functions: HashMap<CaseInsensitive<String>, Option<FunctionId>>,
}

mod functions;
mod vars;
