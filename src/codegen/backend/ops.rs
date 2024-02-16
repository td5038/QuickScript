use anyhow::Result;
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_module::Module;

use crate::{
    ast::operation::Operation,
    codegen::context::{CodegenContext, CompilerContext},
};

use super::Backend;

pub trait OperationCompiler<'a, M: Module>: Backend<'a, M> {
    fn compile_op(
        cctx: &mut CompilerContext<'a, M>,
        ctx: &mut CodegenContext,
        expr: Operation,
    ) -> Result<Value>;
}

impl<'a, M: Module, T: Backend<'a, M>> OperationCompiler<'a, M> for T {
    fn compile_op(
        cctx: &mut CompilerContext<'a, M>,
        ctx: &mut CodegenContext,
        expr: Operation,
    ) -> Result<Value> {
        let data = expr.data();
        let left = Self::compile(cctx, ctx, data.left.content.clone())?;
        let right = Self::compile(cctx, ctx, data.right.content.clone())?;

        match expr {
            Operation::Add(_) => Ok(ctx.builder.ins().fadd(left, right)),
            Operation::Subtract(_) => Ok(ctx.builder.ins().fsub(left, right)),
            Operation::Multiply(_) => Ok(ctx.builder.ins().fmul(left, right)),
            Operation::Divide(_) => Ok(ctx.builder.ins().fdiv(left, right)),

            _ => todo!("This operation is not implemented yet!"),
        }
    }
}
