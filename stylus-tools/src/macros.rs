// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

macro_rules! copy_from_template {
    ($proj:expr, $tmpl:literal -> $root:ident, $($files:expr),* $(,)?) => {
        $(
            let mut filename = $root.join($files);
            if filename.extension() == Some(std::ffi::OsStr::new("tmpl")) {
                filename = filename.file_stem().unwrap().into();
            }
            let template = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $tmpl, "/", $files));
            std::fs::write(
                filename,
                template.replace("$lib", $proj),
            )?;
        )*
    };
}

macro_rules! copy_from_template_if_dne {
    (($proj:expr), $tmpl:literal -> $root:ident, $($files:expr),* $(,)?) => {
        $(
            if !$root.join($files).exists() {
                copy_from_template!($proj, $tmpl -> $root, $files);
            }
        )*
    }
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
        log::warn!("{}", msg.$color())
    }};
}

macro_rules! greyln {
    ($($msg:expr),*) => {{
        use crate::utils::color::Color;
        let msg = format!($($msg),*);
        println!("{}", msg.grey())
    }};
}

#[allow(unused)]
macro_rules! mintln {
    ($($msg:expr),*) => {{
        use crate::utils::color::Color;
        let msg = format!($($msg),*);
        println!("{}", msg.mint())
    }};
}

macro_rules! egreyln {
    ($($msg:expr),*) => {{
        use crate::utils::color::Color;
        let msg = format!($($msg),*);
        eprintln!("{}", msg.grey())
    }};
}
