extern crate chrono;
extern crate getopts;

use std::cmp::Ordering;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process::exit;

use getopts::Options;

const DEFAULT_RESOLVECONF_FN: &str = "/etc/resolv.conf";
const HEADER: &str = "# the nameservers in this file were (possibly) reordered using resolvesolver on ";

fn main() {
    eprintln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("i", "", "modify file in-place");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e.to_string());
            exit(1);
        }
    };

    let resolveconf_fn = if matches.free.is_empty() {
        eprintln!("no filename passed, defaulting to {}", DEFAULT_RESOLVECONF_FN);
        DEFAULT_RESOLVECONF_FN.to_string()
    } else {
        matches.free[0].clone()
    };

    if !Path::new(&resolveconf_fn).exists() {
        eprintln!("No such file: {}", resolveconf_fn);
        exit(1);
    }

    //if let Ok(new_content) = parse_and_replace(&resolveconf_fn) {
    //    if matches.opt_present("i") {
    //        eprintln!("altering file in-place");
    //        write_file(&resolveconf_fn, new_content).map_err(|e| eprintln!("error: {}", e));
    //    } else {
    //        new_content.iter().for_each(|l| println!("{}", l));
    //    }
    //}

    match parse_and_replace(&resolveconf_fn) {
        Ok(new_content) => {
            if matches.opt_present("i") {
                eprintln!("altering file in-place");
                match write_file(&resolveconf_fn, new_content) {
                    Ok(()) => exit(0),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit(1);
                    }
                };
            } else {
                new_content.iter().for_each(|l| println!("{}", l));
            }
        }
        Err(e) => {
            eprintln!("Parsing error: {}", e);
            exit(1);
        }
    };
}

fn parse_and_replace(resolveconf_fn: &str) -> Result<Vec<String>, io::Error> {
    let fh = OpenOptions::new().read(true).open(resolveconf_fn)?;
    let mut raw_lines = BufReader::new(&fh).lines().map(|l| l.unwrap()).collect::<Vec<String>>();

    // we reuse the same lines so the structure of resolv.conf remains the same
    let mut lines_with_nameserver: Vec<usize> = Vec::new();

    // a nameserver line can hold multiple nameservers
    // so we order them within the lines itself, as well as ordering the lines
    // therefore we use a nested vec
    let mut nested_nameservers: Vec<Vec<String>> = Vec::new();

    // find all nameserver lines in the original file, and their line numbers
    for (line_no, line) in raw_lines.iter().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with("nameserver") {
            lines_with_nameserver.push(line_no);
            let mut nameservers: Vec<String> = line.split_whitespace().skip(1).map(|s| s.to_string()).collect();
            assert!(!nameservers.is_empty());
            //nameservers.sort_by(|a, b| sort_v6_over_v4(&a.as_str(), &b.as_str()));
            nameservers.sort_by(|a, _b| {
                if a.contains(':') {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
            nested_nameservers.push(nameservers.to_vec());
        }
    }
    // TODO: also inline the sorting lambda here?
    nested_nameservers.sort_by(|a, b| sort_v6_over_v4(&a[0].as_str(), &b[0].as_str()));

    // make sure we did not lose anything
    assert_eq!(lines_with_nameserver.len(), nested_nameservers.len());

    // replace lines
    for (line_no, nameservers) in lines_with_nameserver.iter().zip(nested_nameservers) {
        raw_lines[*line_no] = format!("nameserver {}", nameservers.clone().join(" "));
    }

    refresh_header(&mut raw_lines);
    Ok(raw_lines)
}

fn refresh_header(lines: &mut Vec<String>) {
    if lines[0].contains(HEADER) {
        eprintln!("replacing header");
        lines[0] = gen_header();
    } else {
        lines.insert(0, gen_header());
        lines.insert(1, "#".to_string());
    }
}

fn gen_header() -> String {
    format!("{} {}", HEADER, chrono::prelude::Local::now().to_rfc2822())
}

fn write_file(out_fn: &str, contents: Vec<String>) -> Result<(), io::Error> {
    let fh = OpenOptions::new().write(true).open(out_fn)?;
    let mut bufw = BufWriter::new(fh);
    for line in contents {
        writeln!(bufw, "{}", line);
    }
    let _ = bufw.flush();

    Ok(())
}

//fn sort_v6_over_v4(a: &String, _b: &String) -> Ordering {
fn sort_v6_over_v4<'s, 't>(a: &'s &str, _b: &'t &str) -> Ordering {
    //naive first version to test
    if a.contains(':') {
        // v6
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
