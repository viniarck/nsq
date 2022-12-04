use std::io;

use std::fs::File;
use std::io::BufRead;

pub fn read_nameservers(filename: String) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let mut names: Vec<String> = Vec::new();
    for line in io::BufReader::new(file).lines() {
        if let Ok(l) = line {
            if !l.contains("nameserver") {
                continue;
            }
            let nameserver = l.split("nameserver").last();
            match nameserver {
                Some(ns) => names.push(ns.trim().to_string()),
                None => (),
            };
        }
    }
    Ok(names)
}
