use std::fmt::{self, Debug, Formatter};

use muscript_foundation::source::SourceFileSet;

use crate::{class::VarKind, Environment, VarId};

use super::Ir;

pub struct DumpIr<'a> {
    pub sources: &'a SourceFileSet,
    pub env: &'a Environment,
    pub ir: &'a Ir,
}

impl<'a> DumpIr<'a> {
    fn local(&self, f: &mut Formatter<'_>, local: VarId) -> fmt::Result {
        let var = self.env.get_var(local);
        let VarKind::Var { ty, .. } = var.kind else { unreachable!("locals must be `var`") };
        let name = self.sources.span(var.source_file_id, &var.name);

        write!(f, "{} ${name}", self.env.type_name(ty))
    }
}

impl<'a> Debug for DumpIr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.env.type_name(self.ir.return_ty))?;
        for i in 0..self.ir.param_count {
            if i != 0 {
                f.write_str(", ")?;
            }
            self.local(f, self.ir.locals[i as usize])?;
        }
        f.write_str(")\n{\n")?;
        f.write_str("}")?;

        Ok(())
    }
}
