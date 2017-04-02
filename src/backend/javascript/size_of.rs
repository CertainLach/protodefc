use ::ir::TypeContainer;
use super::builder::Block;
use ::backend::imperative_base as ib;
use ::errors::*;

pub fn generate_size_of(typ: TypeContainer) -> Result<Block> {
    let base = ib::size_of::generate_size_of(typ.clone())?;

    let mut ib = Block::new();
    ib.let_assign("count".into(), "0".into());

    {
        let typ_inner = typ.borrow();
        ib.let_assign(ib::utils::input_for(&typ_inner.data), "input".into());
    }

    ib.scope(super::ib_to_js::build_block(&base)?);
    ib.return_("count".into());

    let mut b = Block::new();
    b.decl_fun("".into(), vec!["input".into()], ib);
    Ok(b)
}
