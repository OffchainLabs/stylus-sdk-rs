use crate::ty;
use std::fmt::Write;
use syn::Type;

macro_rules! calldata_type_template {
    () => {
        "__{}__Calldata"
    };
}

macro_rules! returndata_type_template {
    () => {
        "__{}__Returndata"
    };
}

macro_rules! generated_handler_name_template {
    () => {
        "__{}__Handler"
    };
}

macro_rules! calldata_sig_name_template {
    () => {
        "sig_{}"
    };
}

// pub(crate) fn signature(params: Vec<Type>) -> String {
//     let mut sig = String::new();
//     sig.push('(');
//     let mut first = true;
//     for param in params {
//         if !first {
//             sig.push(',');
//         }
//         write!(sig, "{}", ty::TypePrinter::new(&param)).unwrap();
//         first = false;
//     }
//     sig.push(')');
//     sig.into()
// }

pub(crate) use calldata_sig_name_template;
pub(crate) use calldata_type_template;
pub(crate) use generated_handler_name_template;
pub(crate) use returndata_type_template;
