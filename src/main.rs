extern crate chrono;

use std::env::args;
use std::process::exit;
use std::path::Path;
use std::cmp::Ordering;
use std::io::{BufReader, BufRead, BufWriter, Write, Seek, SeekFrom};
use std::fs::OpenOptions;

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

    exit(parse_and_fix(&resolveconf_fn));
}


fn parse_and_fix(resolveconf_fn: &str) -> i32 {

    let mut fh = OpenOptions::new().read(true).write(true).open(resolveconf_fn).expect("cant open file in rw mode");

    let mut raw_lines = BufReader::new(&fh).lines().map(|l| l.unwrap()).collect::<Vec<String>>(); //(&mut raw).expect("cant read file");

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

    // and write everything back into the file
    fh.seek(SeekFrom::Start(0)).expect("something wrong while trying to write file");
    let mut bufw = BufWriter::new(fh);
    writeln!(bufw, "# the nameservers in this file were (possibly) reordered using resolvesolver on {}", chrono::prelude::Local::now().to_rfc2822());
    writeln!(bufw, "#");
    for line in raw_lines {
        writeln!(bufw, "{}", line);
    }
    let _ = bufw.flush();


    0
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
