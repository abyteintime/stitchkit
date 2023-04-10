use std::fmt::{self, Debug, Display, Formatter};

use bitflags::BitFlags;
use muscript_foundation::source::SourceFileSet;

use crate::{
    class::VarKind,
    function::{Function, FunctionFlags, FunctionImplementation, ParamFlags},
    Environment, FunctionId, VarId,
};

use super::{Ir, NodeId, NodeKind, Register, RegisterId, Sink, Terminator, Value};

fn local(
    env: &Environment,
    sources: &SourceFileSet,
    f: &mut Formatter<'_>,
    local: VarId,
) -> fmt::Result {
    let var = env.get_var(local);
    let VarKind::Var { ty, .. } = var.kind else { unreachable!("locals must be `var`") };
    let name = sources.span(var.source_file_id, &var.name);

    write!(f, "{} ${name}", env.type_name(ty))
}

pub struct DumpIr<'a> {
    pub sources: &'a SourceFileSet,
    pub env: &'a Environment,
    pub ir: &'a Ir,
}

impl<'a> DumpIr<'a> {
    fn register_id(&self, f: &mut Formatter<'_>, register_id: RegisterId) -> fmt::Result {
        let i = NodeId::from(register_id).to_u32();
        let node = self.ir.node(register_id.into());
        match &node.kind {
            NodeKind::Register(register) => {
                write!(f, "%{}_{i}", register.name)?;
            }
            NodeKind::Sink(_) => unreachable!(),
        }
        Ok(())
    }

    fn function_id(&self, f: &mut Formatter<'_>, function_id: FunctionId) -> fmt::Result {
        let function = self.env.get_function(function_id);
        let class_name = self.env.class_name(function.class_id);
        write!(f, "{class_name}.{}", function.mangled_name)
    }

    fn register(&self, f: &mut Formatter<'_>, node_id: NodeId, register: &Register) -> fmt::Result {
        let i = node_id.to_u32();
        write!(
            f,
            "%{}_{i}: {} = ",
            register.name,
            self.env.type_name(register.ty),
        )?;
        match &register.value {
            Value::Void => f.write_str("void")?,

            Value::Bool(value) => write!(f, "{value}")?,
            Value::Int(value) => write!(f, "int {value}")?,
            Value::Float(value) => write!(f, "float {value}")?,
            Value::String(value) => write!(f, "string {value:?}")?,

            Value::Local(var_id) => {
                f.write_str("local ")?;
                local(self.env, self.sources, f, *var_id)?;
            }

            Value::None => f.write_str("none")?,

            Value::CallFinal {
                function,
                arguments: args,
            } => {
                f.write_str("call final ")?;
                self.function_id(f, *function)?;
                f.write_str(" (")?;
                for (i, register) in args.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }
                    self.register_id(f, *register)?;
                }
                f.write_str(")")?;
            }
            Value::Default => f.write_str("default")?,
        }
        Ok(())
    }

    fn sink(&self, f: &mut Formatter<'_>, sink: &Sink) -> fmt::Result {
        match sink {
            Sink::Discard(register_id) => {
                f.write_str("discard ")?;
                self.register_id(f, *register_id)?;
            }
            Sink::Store(lvalue, rvalue) => {
                f.write_str("store [")?;
                self.register_id(f, *lvalue)?;
                f.write_str("], ")?;
                self.register_id(f, *rvalue)?;
            }
        }
        Ok(())
    }

    fn terminator(&self, f: &mut Formatter<'_>, terminator: &Terminator) -> fmt::Result {
        match terminator {
            Terminator::Unreachable => f.write_str("unreachable")?,
            Terminator::Return(register_id) => {
                f.write_str("return ")?;
                self.register_id(f, *register_id)?;
            }
        }
        Ok(())
    }
}

impl<'a> Debug for DumpIr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("{\n")?;

        for &var in &self.ir.locals {
            f.write_str("    local ")?;
            local(self.env, self.sources, f, var)?;
            writeln!(f)?;
        }
        if !self.ir.locals.is_empty() {
            writeln!(f)?;
        }

        for (i, basic_block) in self.ir.basic_blocks.iter().enumerate() {
            if i != 0 {
                writeln!(f)?;
            }
            writeln!(f, "{}_{i}:", basic_block.label)?;
            for &node_id in &basic_block.flow {
                f.write_str("    ")?;
                let node = self.ir.node(node_id);
                match &node.kind {
                    NodeKind::Register(register) => self.register(f, node_id, register)?,
                    NodeKind::Sink(sink) => self.sink(f, sink)?,
                }
                writeln!(f)?;
            }
            f.write_str("    ")?;
            self.terminator(f, &basic_block.terminator)?;
            writeln!(f)?;
        }

        f.write_str("}")?;

        Ok(())
    }
}

pub struct DumpFunction<'a> {
    pub sources: &'a SourceFileSet,
    pub env: &'a Environment,
    pub function: &'a Function,
    pub ir: Option<&'a Ir>,
}

impl<'a> Debug for DumpFunction<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.env.type_name(self.function.return_ty))?;
        for (i, param) in self.function.params.iter().enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            local(self.env, self.sources, f, self.function.params[i].var)?;
            if !param.flags.is_empty() {
                write!(f, " {}", param.flags)?;
            }
        }
        f.write_str(") ")?;
        match self.function.implementation {
            FunctionImplementation::Script => (),
            FunctionImplementation::Event => f.write_str("event ")?,
            FunctionImplementation::Native => f.write_str("native ")?,
            FunctionImplementation::Opcode(index) => write!(f, "opcode({index}) ")?,
        }
        writeln!(f, "{}", self.function.flags)?;

        if let Some(ir) = self.ir {
            DumpIr {
                sources: self.sources,
                env: self.env,
                ir,
            }
            .fmt(f)?;
        }
        Ok(())
    }
}

struct FlagDisplay<T> {
    flags: T,
    i: usize,
}

impl<T> FlagDisplay<T> {
    fn new(flags: T) -> Self {
        Self { flags, i: 0 }
    }

    fn flag(
        &mut self,
        f: &mut Formatter<'_>,
        single_flag: T,
        flag_name: &str,
    ) -> Result<&mut Self, fmt::Error>
    where
        T: BitFlags,
    {
        if self.flags.contains(single_flag) {
            if self.i != 0 {
                f.write_str(" ")?;
            }
            f.write_str(flag_name)?;
            self.i += 1;
        }
        Ok(self)
    }
}

impl Display for ParamFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        FlagDisplay::new(*self)
            .flag(f, ParamFlags::COERCE, "coerce")?
            .flag(f, ParamFlags::OPTIONAL, "optional")?
            .flag(f, ParamFlags::OUT, "out")?
            .flag(f, ParamFlags::SKIP, "skip")?;
        Ok(())
    }
}

impl Display for FunctionFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        FlagDisplay::new(*self)
            .flag(f, FunctionFlags::CLIENT, "client")?
            .flag(f, FunctionFlags::EDITOR_ONLY, "editoronly")?
            .flag(f, FunctionFlags::EXEC, "exec")?
            .flag(f, FunctionFlags::EXPENSIVE, "expensive")?
            .flag(f, FunctionFlags::FINAL, "final")?
            .flag(f, FunctionFlags::ITERATOR, "iterator")?
            .flag(f, FunctionFlags::LATENT, "latent")?
            .flag(f, FunctionFlags::MULTICAST, "multicast")?
            .flag(f, FunctionFlags::NO_OWNER_REPLICATION, "noownerreplication")?
            .flag(f, FunctionFlags::RELIABLE, "reliable")?
            .flag(f, FunctionFlags::SERVER, "server")?
            .flag(f, FunctionFlags::SIMULATED, "simulated")?
            .flag(f, FunctionFlags::SINGULAR, "singular")?
            .flag(f, FunctionFlags::STATIC, "static")?;

        Ok(())
    }
}
