use std::env;
use std::fs::File;
use std::io::Write;

use clap::{App, Arg};

fn main() {
    let matches = App::new("jep106-build")
        .arg(
            Arg::with_name("pdf")
                .long("pdf")
                .takes_value(true)
                .value_name("PDF"),
        )
        .arg(
            Arg::with_name("version")
                .long("jep_version")
                .takes_value(true)
                .value_name("VERSION"),
        )
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    let dest_path = "codes.rs";

    let mut f = File::create(&dest_path).unwrap();

    let version = matches
        .value_of("version")
        .expect("Missing parameter: --jep_version");

    let contents =
        pdf_extract::extract_text(matches.value_of("pdf").expect("Missing parameter: --pdf"))
            .expect("Something went wrong reading the file");

    let mut data: Vec<Vec<Option<String>>> = vec![];

    for line in contents.lines() {
        use regex::Regex;
        let re = Regex::new(r"^[0-9]+\s+(.*?)\s+([01]\s+){8}([0-9A-F]{2})\s+$").unwrap();
        if let Some(capture) = re.captures(line) {
            if &capture[3] == "01" {
                data.push(vec![None; 256]);
            }
            data.iter_mut().last().expect("This is a bug.")
                [usize::from_str_radix(&capture[3], 16).expect("This is a bug.") & 0x7F] =
                Some(capture[1].into());
        }
    }

    let _ = f.write_all(
        format!(
            "pub(crate) const CODES: [[Option<&'static str>; 256]; {}] = [",
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

    f
        .write_all(
            format!(
                "
        /// Returns the JEP106 specification version code.
        pub const fn version() -> &'static str {{
            \"{}\"
        }}
    ",
                version
            )
            .as_bytes(),
        )
        .unwrap();
}
