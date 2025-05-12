// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

macro_rules! copy_from_template {
    ($tmpl:literal -> $root:ident, $($files:expr,)*) => {
        $(
            std::fs::write(
                $root.join($files),
                include_str!(concat!($tmpl, "/", $files)),
            )?;
        )*
    };
}

macro_rules! debug {
    (@$color:ident, $($msg:expr),*) => {{
        use crate::utils::color::Color;
        let msg = format!($($msg),*);
        log::debug!("{}", msg.$color())
    }};
}

macro_rules! info {
    (@$color:ident, $($msg:expr),*) => {{
        use crate::utils::color::Color;
        let msg = format!($($msg),*);
        log::info!("{}", msg.$color())
    }};
}

macro_rules! warn {
    (@$color:ident, $($msg:expr),*) => {{
        use crate::utils::color::Color;
        let msg = format!($($msg),*);
        log::info!("{}", msg.$color())
    }};
}
