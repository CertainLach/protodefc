use super::builder::{Block, Expr};
use ::backend::imperative_base as ib;
use ::errors::*;
use itertools::Itertools;

pub fn build_block(block: &ib::Block) -> Result<Block> {
    let mut b = Block::new();

    for operation in &block.0 {
        match *operation {
            ib::Operation::Assign { ref name, ref value } =>
                b.var_assign(name.0.clone(), build_expr(value)?.into()),
            ib::Operation::AddCount(ref expr) =>
                b.assign("count".into(),
                         format!("count + {}", build_expr(expr)?.0).into()),
            ib::Operation::Block(ref block) => b.scope(build_block(block)?),
            ib::Operation::ForEachArray { ref array, ref index,
                                          ref typ, ref block } => {
                let index_var = &index.0;
                let length_var = format!("{}_length", index.0);

                let mut ib = Block::new();
                {
                    let expr = format!("{}[{}]", array.0, index_var);
                    ib.let_assign(typ.0.clone(), expr.into());

                    ib.scope(build_block(block)?);
                }

                b.let_assign(length_var.clone(),
                             format!("{}.length", array.0).into());
                b.for_(
                    format!("var {} = 0", index_var).into(),
                    format!("{} < {}", index_var, length_var).into(),
                    format!("{}++", index_var).into(),
                    ib
                );
            },
            ib::Operation::MapValue { ref input, ref output,
                                      operation: ib::MapOperation::ArrayLength } => {
                let input_var = &input.0;
                let output_var = &output.0;

                b.var_assign(output_var.clone(),
                             format!("{}.length", input_var).into());
            }
            ib::Operation::MapValue {
                ref input, ref output,
                operation: ib::MapOperation::UnionTagToExpr(ref cases) } => {

                let cases: Result<Vec<(Expr, Block)>> = cases.iter()
                    .map(|&ib::UnionTagCase { ref variant_name, ref block,
                                              ref variant_var }| {
                        build_block(block).map(|ib| {
                            let mut iib = Block::new();

                            if let &Some(ref variant_var_inner) = variant_var {
                                iib.var_assign(
                                    variant_var_inner.0.clone(),
                                    format!("{}.data", input.0).into()
                                );
                            }
                            iib.block(ib);

                            (format!("case \"{}\"", variant_name).into(), iib)
                        })
                    }).collect();

                b.switch(
                    format!("{}.tag", input.0).into(),
                    cases?
                );
            },
            ib::Operation::MapValue {
                ref input, ref output,
                operation: ib::MapOperation::LiteralToExpr(ref cases) } => {

                let cases: Result<Vec<(Expr, Block)>> = cases.iter()
                    .map(|&ib::LiteralCase { ref value, ref block }| {
                        build_block(block).map(|ib| {
                            (
                                format!("case {}",
                                        build_literal(value).0).into(),
                                ib
                            )
                        })
                    })
                    .collect();

                b.switch(
                    format!("{}", input.0).into(),
                    cases?
                );
            }
            ib::Operation::ConstructContainer { ref output, ref fields } => {
                let obj_fields = fields.iter()
                    .map(|&(ref name, ref var)| format!("{}: {}", name, var.0))
                    .join(", ");

                b.var_assign(output.0.clone(), format!("{{ {} }}", obj_fields).into());
            }
            ib::Operation::ConstructArray { ref count, ref ident, ref item_var,
                                            ref block, ref output } => {
                let index_var = format!("array_{}_index", ident);

                let mut ib = Block::new();
                ib.block(build_block(block)?);
                ib.expr(format!("{}.push({})",
                                output.0.clone(), item_var.0.clone()).into());

                b.var_assign(output.0.clone(), "[]".into());
                b.for_(
                    format!("var {} = 0", index_var).into(),
                    format!("{} < {}", index_var, count.0).into(),
                    format!("{}++", index_var).into(),
                    ib
                );
            }
            ib::Operation::ConstructUnion { ref union_tag, ref output,
                                            ref input, .. } => {
                b.var_assign(
                    output.0.clone(),
                    format!("{{ tag: \"{}\", data: {} }}", union_tag, input.0).into()
                );
            }
            ib::Operation::TypeCall { ref input, ref output, ref type_name,
                                      typ: ib::CallType::SizeOf } => {
                let res = format!("types[\"{}\"][\"size_of\"]({})",
                                  type_name, input.0);
                b.var_assign(output.0.clone(), res.into());
            }
            ib::Operation::TypeCall { ref input, ref output, ref type_name,
                                      typ: ib::CallType::Serialize } => {
                let res = format!("types[\"{}\"][\"serialize\"]({}, buffer, offset)",
                                  type_name, input.0);
                b.var_assign("offset".to_owned(), res.into());
            }
            ib::Operation::TypeCall { ref input, ref output, ref type_name,
                                      typ: ib::CallType::Deserialize } => {
                let res = format!("types[\"{}\"][\"deserialize\"]({}, offset)",
                                  type_name, input.0);
                b.var_assign(format!("[{}, offset]", output.0), res.into());
            }
        }
    }

    Ok(b)
}

fn build_expr(expr: &ib::Expr) -> Result<Expr> {
    let res = match *expr {
        ib::Expr::InputData => format!("input").into(),
        ib::Expr::Var(ref var) => var.0.clone().into(),
        ib::Expr::Literal(ib::Literal::Number(ref num)) =>
            num.clone(),
        ib::Expr::ContainerField { ref value, ref field } =>
            format!("{}[{:?}]", build_expr(value)?.0, field),
        _ => unimplemented!(),
    };
    Ok(res.into())
}

fn build_literal(lit: &ib::Literal) -> Expr{
    match lit {
        &ib::Literal::Number(ref val) => val.to_owned().into(),
    }
}
