use std::cell::{RefCell, Ref, RefMut};
use std::rc::{Rc, Weak};
use std::fmt::Debug;
use std::any::Any;

use ::errors::*;

use self::variant::VariantType;

pub mod variant;
mod debug_printer;

mod field_property_reference;
mod field_reference;
pub use self::field_reference::FieldReference;
pub use self::field_property_reference::FieldPropertyReference;

mod target_type;
pub use self::target_type::TargetType;

use ::context::compilation_unit::{CompilationUnit, TypePath};

#[derive(Clone)]
pub struct TypeContainer(Rc<RefCell<Type>>);
impl TypeContainer {
    pub fn new(typ: Type) -> TypeContainer {
        TypeContainer(Rc::new(RefCell::new(typ)))
    }

    pub fn borrow(&self) -> Ref<Type> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<Type> {
        self.0.borrow_mut()
    }

    pub fn downgrade(&self) -> WeakTypeContainer {
        WeakTypeContainer(Rc::downgrade(&self.0))
    }
}

#[derive(Debug, Clone)]
pub struct WeakTypeContainer(Weak<RefCell<Type>>);
impl WeakTypeContainer {
    pub fn upgrade(&self) -> TypeContainer {
        TypeContainer(self.0.upgrade().unwrap())
    }
}

//pub type TypeContainer = Rc<RefCell<Type>>;
//pub type WeakTypeContainer = Weak<RefCell<Type>>;

pub struct Type {
    pub data: TypeData,
    pub variant: self::variant::Variant,
}

#[derive(Debug)]
pub struct TypeData {
    pub name: TypePath,
    pub children: Vec<TypeContainer>,

    /// Added in AssignParentPass
    pub parent: Option<WeakTypeContainer>,

    /// Added in AssignIdentPass
    /// Idents increase with causality.
    pub ident: Option<u64>,
}

impl Default for TypeData {
    fn default() -> TypeData {
        TypeData {
            name: TypePath::with_no_ns("".to_owned()),
            children: Vec::new(),

            parent: None,
            ident: None,
        }
    }
}

pub type ReferenceResolver = Fn(&TypeVariant, &TypeData, &FieldReference)
                                -> Result<WeakTypeContainer>;

/// Every primitive type supported by the compiler needs to
/// implement this trait. It is used by compiler passes/backends
/// to get details on how the type should function.
pub trait TypeVariant: Debug + Any {

    /// Used by the compiler to check whether it is legal to refer
    /// to a property of a given type variant.
    ///
    /// This is used by virtual container fields to get their value
    /// when writing.
    fn has_property(&self, data: &TypeData, prop_name: &str)
                    -> Option<TargetType>;

    /// Used by the compiler to determine if the type can be used
    /// as a count / in a union.
    fn get_result_type(&self, data: &TypeData) -> Option<TargetType>;

    /// Used by the resolve_reference pass to fetch a named child
    /// for the given type.
    ///
    /// This should only be implemented for composite types, simple
    /// types should simply return an error here.
    fn resolve_child_name(&self, data: &TypeData, name: &str)
                          -> Result<WeakTypeContainer>;

    fn do_resolve_references(&mut self, data: &mut TypeData,
                             resolver: &ReferenceResolver) -> Result<()>;

    fn get_type(&self, data: &TypeData) -> VariantType;

    fn resolve_on_context(&self, data: &TypeData, current_path: &TypePath,
                          context: &CompilationUnit) -> Result<()>;

}
