extern crate chrono;

use std::env::args;
use std::process::exit;
use std::path::Path;
use std::cmp::Ordering;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::fs::OpenOptions;
use std::io;


const DEFAULT_RESOLVECONF_FN: &str = "/etc/resolv.conf";
fn main() {
    eprintln!(env!("CARGO_PKG_VERSION"));
    let args = args().collect::<Vec<String>>();
    let resolveconf_fn = if args.len() != 2 {
        eprintln!("no filename passed, defaulting to {}", DEFAULT_RESOLVECONF_FN);
        DEFAULT_RESOLVECONF_FN.to_string()
    } else {
        args[1].clone()
    };

    if !Path::new(&resolveconf_fn).exists() {
        eprintln!("No such file: {}", resolveconf_fn);
        exit(1);
    }

    // TODO: replace unwrap()s with proper error handling
    let new_content = parse_and_replace(&resolveconf_fn).unwrap();
    // TODO: introduce a `-i` flag to do inplace replacement
    // otherwise cat to stdout
    write_file(&resolveconf_fn, new_content).unwrap();
    exit(0);
}


fn parse_and_replace(resolveconf_fn: &str) -> Result<Vec<String>, io::Error> {

    let fh = OpenOptions::new().read(true).open(resolveconf_fn).expect("cant open file in rw mode");

    let mut raw_lines = BufReader::new(&fh).lines().map(|l| l.unwrap()).collect::<Vec<String>>();

    // we reuse the same lines so the structure of resolv.conf remains the same
    let mut lines_with_nameserver: Vec<usize> = Vec::new();

    // a nameserver line can hold multiple nameservers
    // so we order them within the lines itself, as well as ordering the lines
    // therefore we use a nested vec
    let mut nested_nameservers: Vec<Vec<String>> = Vec::new();

    // find all nameserver lines in the original file, and their line numbers
    for (line_no, line) in raw_lines.iter().enumerate(){
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with("nameserver") {
            lines_with_nameserver.push(line_no);
            let mut nameservers: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>()[1..].to_vec();
            assert!(!nameservers.is_empty());
            nameservers.sort_by(sort_v6_over_v4);
            nested_nameservers.push(nameservers.to_vec());
        }
    }
    nested_nameservers.sort_by(|a, b| sort_v6_over_v4(&a[0], &b[0]));

    // make sure we did not lose anything
    assert_eq!(lines_with_nameserver.len(), nested_nameservers.len());

    // replace lines 
    for (line_no, nameservers) in lines_with_nameserver.iter().zip(nested_nameservers) {
        raw_lines[*line_no] = format!("nameserver {}", nameservers.clone().join(" "));
    }

    Ok(raw_lines)
}

fn write_file(out_fn: &str, contents: Vec<String>) -> Result<(), io::Error> {
    let fh = OpenOptions::new().write(true).open(out_fn).expect("cant open file in rw mode");
    let mut bufw = BufWriter::new(fh);
    //TODO can we prevent this header from being repeated?
    writeln!(bufw, "# the nameservers in this file were (possibly) reordered using resolvesolver on {}", chrono::prelude::Local::now().to_rfc2822());
    writeln!(bufw, "#");
    for line in contents {
        writeln!(bufw, "{}", line);
    }
    let _ = bufw.flush();

    Ok(())
}

fn sort_v6_over_v4(a: &String, _b: &String) -> Ordering {
    //naive first version to test
    if a.contains(':') {
        // v6
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
