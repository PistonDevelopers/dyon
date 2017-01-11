use std::sync::Arc;
use std::io::{self, Read};
use std::fs::File;
use std::error::Error;
use piston_meta::*;
use super::io::io_error;

use Variable;

pub fn load_syntax_data(rules: &Syntax, file: &str, d: &str) -> Result<Vec<Variable>, String> {
    let mut tokens = vec![];
    try!(parse_errstr(&rules, &d, &mut tokens).map_err(|err|
        format!("When parsing data in `{}`:\n{}", file, err)));
    let mut res = vec![];
    let b: Arc<String> = Arc::new("bool".into());
    let s: Arc<String> = Arc::new("str".into());
    let n: Arc<String> = Arc::new("f64".into());
    let start: Arc<String> = Arc::new("start".into());
    let end: Arc<String> = Arc::new("end".into());
    for range_token in &tokens {
        let mut data = vec![];
        data.push(Variable::f64(range_token.offset as f64));
        data.push(Variable::f64(range_token.length as f64));
        match &range_token.data {
            &MetaData::Bool(ref name, val) => {
                data.push(Variable::Text(b.clone()));
                data.push(Variable::Text(name.clone()));
                data.push(Variable::bool(val));
            }
            &MetaData::String(ref name, ref val) => {
                data.push(Variable::Text(s.clone()));
                data.push(Variable::Text(name.clone()));
                data.push(Variable::Text(val.clone()));
            }
            &MetaData::F64(ref name, val) => {
                data.push(Variable::Text(n.clone()));
                data.push(Variable::Text(name.clone()));
                data.push(Variable::f64(val));
            }
            &MetaData::StartNode(ref name) => {
                data.push(Variable::Text(start.clone()));
                data.push(Variable::Text(name.clone()));
            }
            &MetaData::EndNode(ref name) => {
                data.push(Variable::Text(end.clone()));
                data.push(Variable::Text(name.clone()));
            }
        }
        res.push(Variable::Array(Arc::new(data)));
    }
    Ok(res)
}

fn load_metarules_data(meta: &str, s: &str, file: &str, d: &str) -> Result<Vec<Variable>, String> {
    let rules = try!(syntax_errstr(&s).map_err(|err|
        format!("When parsing meta syntax in `{}`:\n{}", meta, err)));
    load_syntax_data(&rules, file, d)
}

/// Loads a file using a meta file as syntax.
pub fn load_meta_file(meta: &str, file: &str) -> Result<Vec<Variable>, String> {
    let mut syntax_file = try!(File::open(meta).map_err(|err| io_error("open", meta, &err)));
    let mut s = String::new();
    try!(syntax_file.read_to_string(&mut s).map_err(|err| io_error("read", meta, &err)));
    let mut data_file = try!(File::open(file).map_err(|err| io_error("open", file, &err)));
    let mut d = String::new();
    try!(data_file.read_to_string(&mut d).map_err(|err| io_error("read", file, &err)));
    load_metarules_data(meta, &s, file, &d)
}

/// Loads a text file from url.
fn load_text_file_from_url(url: &str) -> Result<String, String> {
    use hyper::client::Client;
    use hyper::{Url};
    use hyper::status::StatusCode;
    use std::io::Read;

    let url_address = try!(Url::parse(url)
        .map_err(|e| format!("Error parsing url:\n`{}`\n", e)));
    let client = Client::new();
    let request = client.get(url_address);
    let mut response = try!(request.send()
        .map_err(|e| format!("Error fetching file over http `{}`:\n{}\n",
                             url, e.to_string())));
    if response.status == StatusCode::Ok {
        let mut data = String::new();
        try!(response.read_to_string(&mut data)
            .map_err(|e| format!("Error fetching file over http `{}`:\n{}\n",
                                 url, e.to_string())));
        Ok(data)
    } else {
        Err(format!("Error fetching file over http `{}:\n{}\n",
                    url, response.status))
    }
}

/// Loads an url using a meta file as syntax.
pub fn load_meta_url(meta: &str, url: &str) -> Result<Vec<Variable>, String> {
    let mut syntax_file = try!(File::open(meta).map_err(|err| io_error("open", meta, &err)));
    let mut s = String::new();
    try!(syntax_file.read_to_string(&mut s).map_err(|err| io_error("read", meta, &err)));
    let d = try!(load_text_file_from_url(url));
    load_metarules_data(meta, &s, url, &d)
}

// Downloads a file from url.
pub fn download_url_to_file(url: &str, file: &str) -> Result<String, String> {
    use hyper::client::Client;
    use hyper::{Url};
    use hyper::status::StatusCode;
    use std::io::copy;
    use std::fs::File;

    let url_address = try!(Url::parse(url)
        .map_err(|e| format!("Error parsing url:\n`{}`\n", e)));
    let client = Client::new();
    let request = client.get(url_address);
    let mut response = try!(request.send()
        .map_err(|e| format!("Error fetching file over http `{}`:\n{}\n",
                             url, e.to_string())));
    if response.status == StatusCode::Ok {
        let mut f = try!(File::create(file).map_err(|err| {
            format!("Could not create file `{}`:\n{}", file, err.description())
        }));
        try!(copy(&mut response, &mut f)
            .map_err(|e| format!("Error fetching file over http `{}`:\n{}\n",
                                 url, e.to_string())));
        Ok(file.into())
    } else {
        Err(format!("Error fetching file over http `{}:\n{}\n",
                    url, response.status))
    }
}

pub fn json_from_meta_data(data: &Vec<Variable>) -> Result<String, io::Error> {
    fn is_start_node(v: &Variable) -> bool {
        if let &Variable::Array(ref arr) = v {
            if let &Variable::Text(ref t) = &arr[2] {
                &**t == "start"
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_end_node(v: &Variable) -> bool {
        if let &Variable::Array(ref arr) = v {
            if let &Variable::Text(ref t) = &arr[2] {
                &**t == "end"
            } else {
                false
            }
        } else {
            false
        }
    }

    use std::cmp::{ min, max };
    use std::io::Write;
    use piston_meta::json::write_string;

    let indent_offset = 0;
    let mut w: Vec<u8> = vec![];

    // Start indention such that it balances off to zero.
    let starts = data.iter()
        .filter(|x| is_start_node(x))
        .count() as u32;
    let ends = data.iter()
        .filter(|x| is_end_node(x))
        .count() as u32;
    let mut indent: u32 = max(starts, ends) - min(starts, ends);
    let mut first = true;
    for (i, d) in data.iter().enumerate() {
        let is_end = if is_end_node(d) {
            indent -= 1;
            true
        } else { false };
        let print_comma = !first && !is_end;
        if print_comma {
            try!(writeln!(w, ","));
        } else if i != 0 {
            try!(writeln!(w, ""));
        }
        first = false;
        for _ in 0 .. indent_offset + indent {
            try!(write!(w, " "));
        }
        if let &Variable::Array(ref arr) = d {
            let name = if let &Variable::Text(ref t) = &arr[3] {
                t
            } else {
                ""
            };
            if let &Variable::Text(ref t) = &arr[2] {
                match &***t {
                    "start" => {
                        first = true;
                        try!(write_string(&mut w, name));
                        try!(write!(w, ":{}", "{"));
                        indent += 1;
                    }
                    "end" => {
                        try!(write!(w, "{}", "}"));
                    }
                    "bool" => {
                        if let &Variable::Bool(val, _) = &arr[4] {
                            try!(write_string(&mut w, name));
                            try!(write!(w, ":{}", val));
                        }
                    }
                    "f64" => {
                        if let &Variable::F64(val, _) = &arr[4] {
                            try!(write_string(&mut w, name));
                            try!(write!(w, ":{}", val));
                        }
                    }
                    "str" => {
                        if let &Variable::Text(ref val) = &arr[4] {
                            try!(write_string(&mut w, name));
                            try!(write!(w, ":"));
                            try!(write_string(&mut w, val));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    try!(writeln!(w, ""));
    Ok(String::from_utf8(w).unwrap())
}
