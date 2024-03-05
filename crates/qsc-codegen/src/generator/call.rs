use anyhow::Result;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Value};
use cranelift_module::{Linkage, Module};
use crate::context::{CodegenContext, CompilerContext};
use qsc_ast::ast::stmt::call::CallNode;
use super::Backend;

pub trait CallCompiler<'i, 'a, M: Module>: Backend<'i, 'a, M> {
    fn compile_call(
        cctx: &mut CompilerContext<'i, 'a, M>,
        ctx: &mut CodegenContext,
        call: CallNode<'i>,
    ) -> Result<Value>;
}

impl<'i, 'a, M: Module, T: Backend<'i, 'a, M>> CallCompiler<'i, 'a, M> for T {
    fn compile_call(
        cctx: &mut CompilerContext<'i, 'a, M>,
        ctx: &mut CodegenContext,
        call: CallNode<'i>,
    ) -> Result<Value> {
        let mut sig = cctx.module.make_signature();

        if cctx.functions.contains_key(call.func) {
            let ptr = Self::ptr(cctx);
            let func = cctx.functions.get(call.func).unwrap();
            
            let args = func
                .args
                .iter()
                .map(|v| v.type_.as_str())
                .collect::<Vec<String>>()
                .join(", ");

            debug!(
                "Using local function for call: {}({}) -> {}",
                call.func, args, func.ret.map(|v| v.as_str()).unwrap_or(String::new())
            );

            sig.params.append(
                &mut func
                    .args
                    .iter()
                    .map(|p| AbiParam::new(Self::query_type_with_pointer(ptr, p.type_.as_str())))
                    .collect(),
            );

            sig.returns.push(AbiParam::new(Self::query_type(
                cctx,
                func.ret.map(|v| v.as_str()).unwrap_or(String::new()),
            )));
        } else {
            let args = call
                .args
                .iter()
                .map(|arg| {
                    if let Ok(ident) = arg.value.data.as_symbol() {
                        if ctx.vars.contains_key(ident.value) {
                            return ctx
                                .vars
                                .get(ident.value)
                                .unwrap()
                                .1
                                .map(|v| v.as_str())
                                .unwrap_or("ptr".to_string());
                        }
                    }

                    "ptr".to_string()
                })
                .collect::<Vec<String>>();

            debug!(
                "Using imported function for call (Linkage::Import): {}({}) -> i32",
                call.func,
                args.join(", ")
            );

            sig.params.append(
                &mut args
                    .iter()
                    .map(|ty| AbiParam::new(Self::query_type(cctx, ty.clone())))
                    .collect(),
            );

            sig.returns
                .push(AbiParam::new(Self::query_type(cctx, "i32".to_string())));
        }

        let callee = cctx
            .module
            .declare_function(&call.func, Linkage::Import, &sig)?;

        let local_callee = cctx
            .module
            .declare_func_in_func(callee, &mut ctx.builder.func);

        let mut args = Vec::new();

        for arg in call.args {
            args.push(Self::compile(cctx, ctx, arg.value)?);
        }

        let call = ctx.builder.ins().call(local_callee, &args);
        let result = ctx.builder.inst_results(call)[0];

        Ok(result)
    }
}
