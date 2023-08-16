use std::fmt;
use syn_solidity::Type;

// Copied & modified from alloy-sol-macro/src/expand/ty.rs

// Implements [`fmt::Display`] which formats a [`Type`] to its canonical
/// representation. This is then used in function, error, and event selector
/// generation.
pub(super) struct TypePrinter<'ast> {
    // cx: &'ast ExpCtxt<'ast>,
    ty: &'ast Type,
}

impl<'ast> TypePrinter<'ast> {
    pub(super) fn new(ty: &'ast Type) -> Self {
        Self { ty }
    }
}

impl fmt::Display for TypePrinter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ty {
            Type::Int(_, None) => f.write_str("int256"),
            Type::Uint(_, None) => f.write_str("uint256"),

            Type::Array(array) => {
                Self::new(&array.ty).fmt(f)?;
                f.write_str("[")?;
                if let Some(size) = &array.size {
                    size.fmt(f)?;
                }
                f.write_str("]")
            }
            Type::Tuple(tuple) => {
                f.write_str("(")?;
                for (i, ty) in tuple.types.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    Self::new(ty).fmt(f)?;
                }
                f.write_str(")")
            }

            // NOTE: No support for custom types yet
            // Type::Custom(name) => self.cx.custom_type(name).fmt(f),
            ty => ty.fmt(f),
        }
    }
}
