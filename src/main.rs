use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use serde_json::Value;

#[derive(clap::Parser)]
struct Opts {
    #[arg(long, value_name = "PDF")]
    pdf: PathBuf,
    #[arg(long = "jep_version", value_name = "VERSION")]
    jep_version: String,
    #[arg(long, value_name = "FORMAT", default_value_t = default::FORMAT)]
    format: Format,
}

#[derive(clap::ValueEnum, Clone)]
enum Format {
    Rust,
    Json,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Format::Rust => "rust",
            Format::Json => "json",
        })
    }
}

mod default {
    use crate::Format;

    pub const FORMAT: Format = Format::Rust;
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let dest_path = match opts.format {
        Format::Rust => "codes.rs",
        Format::Json => "codes.json",
    };

    let f = File::create(dest_path)?;

    let contents = pdf_extract::extract_text(&opts.pdf)
        .with_context(|| format!("Failed to extract text from file '{}'. Usually errors happen when the PDF has a bad format. Try exporting to PDF/A.", opts.pdf.display()))?;

    let mut data: Vec<Vec<Option<String>>> = vec![];

    let re = Regex::new(r"^[0-9]+\s+(.*?)\s+([01]\s+){8}([0-9A-F]{2})\s+$").unwrap();

    for line in contents.lines() {
        if let Some(capture) = re.captures(line) {
            if &capture[3] == "01" {
                data.push(vec![None; 256]);
            }
            data.iter_mut().last().expect("This is a bug.")
                [usize::from_str_radix(&capture[3], 16).expect("This is a bug.") & 0x7F] =
                Some(capture[1].into());
        }
    }

    match opts.format {
        Format::Rust => make_rust(f, data, opts.jep_version)?,
        Format::Json => make_json(f, data, opts.jep_version)?,
    }

    Ok(())
}

fn make_rust(
    mut f: File,
    data: Vec<Vec<Option<String>>>,
    jep_version: String,
) -> Result<(), anyhow::Error> {
    let _ = f.write_all(
        format!(
            "pub(crate) static CODES: [[Option<&'static str>; 256]; {}] = [",
            data.len()
        )
        .as_bytes(),
    );
    for bank in data.iter() {
        let _ = f.write(b"[");
        for company in bank {
            if let Some(company) = company {
                let _ = f.write_all(format!("Some(\"{}\"),", company).as_bytes());
            } else {
                let _ = f.write_all(b"None,");
            }
        }
        let _ = f.write_all(b"],");
    }
    let _ = f.write_all(b"];");
    f.write_all(
        format!(
            "
        /// Returns the JEP106 specification version code.
        pub const fn version() -> &'static str {{
            \"{}\"
        }}
    ",
            jep_version
        )
        .as_bytes(),
    )?;
    Ok(())
}

fn make_json(
    f: File,
    data: Vec<Vec<Option<String>>>,
    jep_version: String,
) -> Result<(), anyhow::Error> {
    let mut manufacturers = vec![];

    for bank in data.into_iter() {
        for manufacturer in bank {
            manufacturers.push(if let Some(manufacturer) = manufacturer {
                Value::String(manufacturer)
            } else {
                Value::Null
            });
        }
    }

    let data = Value::Object({
        let mut map = serde_json::Map::new();
        map.insert("version".to_string(), Value::String(jep_version));
        map.insert("manufacturers".to_string(), Value::Array(manufacturers));
        map
    });

    serde_json::to_writer_pretty(f, &data)?;
    Ok(())
}
