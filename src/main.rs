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
const HEADER: &str = "# Nameservers (possibly) reordered by resolvloser on";

fn main() {
    eprintln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("i", "", "modify in-place (otherwise output on stdout)");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            let brief = format!("Usage: {} RESOLVCONF_FILE [options]", env!("CARGO_PKG_NAME"));
            print!("{}", opts.usage(&brief));
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
    let mut raw_lines = BufReader::new(&fh)
        .lines()
        .map(|l| l.unwrap())
        .collect::<Vec<String>>();

    let (line_nos, mut nameservers): (Vec<usize>, Vec<String>) = raw_lines
        .iter()
        .enumerate()
        .filter(|(_line_no, l)| l.starts_with("nameserver"))
        .filter_map(|(line_no, l)| l.split_whitespace()
                    .nth(1)
                    .map(|ns| (line_no, ns.to_string()) )
                    )
        .unzip()
    ; 

    nameservers.sort_by(|a, b| sort_v6_over_v4(a, b));
    //nameservers.sort_by(sort_v6_over_v4);

    // make sure we did not lose anything
    assert_eq!(line_nos.len(), nameservers.len(), "lost a nameserver..");

    // replace lines
    for (line_no, nameserver) in line_nos.iter().zip(nameservers) {
        raw_lines[*line_no] = format!("nameserver {}", nameserver);
    }

    refresh_header(&mut raw_lines);
    Ok(raw_lines)
}

fn refresh_header(lines: &mut Vec<String>) {
    if lines[0].contains(HEADER) {
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

//fn sort_v6_over_v4<'s, 't>(a: &'s &str, _b: &'t &str) -> Ordering {
fn sort_v6_over_v4(a: &str, _b: &str) -> Ordering {
    // naive sorting
    // perhaps prioritizing global over link-local (or the other way around) makes sense
    if a.contains(':') {
        // it's v6
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
