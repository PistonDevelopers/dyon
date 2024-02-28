#[cfg(feature = "file")]
use super::io::io_error;
use piston_meta::{parse_errstr, MetaData, Syntax};
#[cfg(feature = "file")]
use piston_meta::syntax_errstr;
#[cfg(feature = "file")]
use std::fs::File;
use std::io;
#[cfg(feature = "file")]
use std::io::Read;
use std::sync::Arc;

use crate::Variable;

pub fn parse_syntax_data(rules: &Syntax, file: &str, d: &str) -> Result<Vec<Variable>, String> {
    let mut tokens = vec![];
    parse_errstr(rules, d, &mut tokens)
        .map_err(|err| format!("When parsing data in `{}`:\n{}", file, err))?;
    let mut res = vec![];
    let b: Arc<String> = Arc::new("bool".into());
    let s: Arc<String> = Arc::new("str".into());
    let n: Arc<String> = Arc::new("f64".into());
    let start: Arc<String> = Arc::new("start".into());
    let end: Arc<String> = Arc::new("end".into());
    for range_token in &tokens {
        let mut data = vec![
            Variable::f64(range_token.offset as f64),
            Variable::f64(range_token.length as f64),
        ];
        match range_token.data {
            MetaData::Bool(ref name, val) => {
                data.push(Variable::Str(b.clone()));
                data.push(Variable::Str(name.clone()));
                data.push(Variable::bool(val));
            }
            MetaData::String(ref name, ref val) => {
                data.push(Variable::Str(s.clone()));
                data.push(Variable::Str(name.clone()));
                data.push(Variable::Str(val.clone()));
            }
            MetaData::F64(ref name, val) => {
                data.push(Variable::Str(n.clone()));
                data.push(Variable::Str(name.clone()));
                data.push(Variable::f64(val));
            }
            MetaData::StartNode(ref name) => {
                data.push(Variable::Str(start.clone()));
                data.push(Variable::Str(name.clone()));
            }
            MetaData::EndNode(ref name) => {
                data.push(Variable::Str(end.clone()));
                data.push(Variable::Str(name.clone()));
            }
        }
        res.push(Variable::Array(Arc::new(data)));
    }
    Ok(res)
}

#[cfg(feature = "file")]
fn load_metarules_data(meta: &str, s: &str, file: &str, d: &str) -> Result<Vec<Variable>, String> {
    let rules = syntax_errstr(s)
        .map_err(|err| format!("When parsing meta syntax in `{}`:\n{}", meta, err))?;
    parse_syntax_data(&rules, file, d)
}

/// Loads a file using a meta file as syntax.
#[cfg(feature = "file")]
pub fn load_meta_file(meta: &str, file: &str) -> Result<Vec<Variable>, String> {
    let mut syntax_file = File::open(meta).map_err(|err| io_error("open", meta, &err))?;
    let mut s = String::new();
    syntax_file
        .read_to_string(&mut s)
        .map_err(|err| io_error("read", meta, &err))?;
    let mut data_file = File::open(file).map_err(|err| io_error("open", file, &err))?;
    let mut d = String::new();
    data_file
        .read_to_string(&mut d)
        .map_err(|err| io_error("read", file, &err))?;
    load_metarules_data(meta, &s, file, &d)
}

#[cfg(not(feature = "file"))]
pub fn load_meta_file(_: &str, _: &str) -> Result<Vec<Variable>, String> {
    Err(super::FILE_SUPPORT_DISABLED.into())
}

/// Loads a text file from url.
#[cfg(all(not(target_family = "wasm"), feature = "http"))]
pub fn load_text_file_from_url(url: &str) -> Result<String, String> {
    use reqwest::{Client, StatusCode, Url};

    let url_address = Url::parse(url).map_err(|e| format!("Error parsing url:\n`{}`\n", e))?;
    let client = Client::new();
    let request = client.get(url_address);
    let mut response = request.send().map_err(|e| {
        format!(
            "Error fetching file over http `{}`:\n{}\n",
            url,
            e.to_string()
        )
    })?;
    if response.status() == StatusCode::OK {
        let mut data = String::new();
        response.read_to_string(&mut data).map_err(|e| {
            format!(
                "Error fetching file over http `{}`:\n{}\n",
                url,
                e.to_string()
            )
        })?;
        Ok(data)
    } else {
        Err(format!(
            "Error fetching file over http `{}:\n{}\n",
            url,
            response.status()
        ))
    }
}

#[cfg(not(all(not(target_family = "wasm"), feature = "http")))]
pub fn load_text_file_from_url(_url: &str) -> Result<String, String> {
    Err(super::HTTP_SUPPORT_DISABLED.into())
}

/// Loads an url using a meta file as syntax.
#[cfg(all(not(target_family = "wasm"), feature = "http"))]
pub fn load_meta_url(meta: &str, url: &str) -> Result<Vec<Variable>, String> {
    let mut syntax_file = File::open(meta).map_err(|err| io_error("open", meta, &err))?;
    let mut s = String::new();
    syntax_file
        .read_to_string(&mut s)
        .map_err(|err| io_error("read", meta, &err))?;
    let d = load_text_file_from_url(url)?;
    load_metarules_data(meta, &s, url, &d)
}

#[cfg(not(all(not(target_family = "wasm"), feature = "http")))]
pub fn load_meta_url(_meta: &str, _url: &str) -> Result<Vec<Variable>, String> {
    Err(super::HTTP_SUPPORT_DISABLED.into())
}

// Downloads a file from url.
#[cfg(all(not(target_family = "wasm"), feature = "http"))]
pub fn download_url_to_file(url: &str, file: &str) -> Result<String, String> {
    use reqwest::{Client, StatusCode, Url};
    use std::io::copy;

    let url_address = Url::parse(url).map_err(|e| format!("Error parsing url:\n`{}`\n", e))?;
    let client = Client::new();
    let request = client.get(url_address);
    let mut response = request.send().map_err(|e| {
        format!(
            "Error fetching file over http `{}`:\n{}\n",
            url,
            e.to_string()
        )
    })?;
    if response.status() == StatusCode::OK {
        let mut f = File::create(file)
            .map_err(|err| format!("Could not create file `{}`:\n{}", file, err.to_string()))?;
        copy(&mut response, &mut f).map_err(|e| {
            format!(
                "Error fetching file over http `{}`:\n{}\n",
                url,
                e.to_string()
            )
        })?;
        Ok(file.into())
    } else {
        Err(format!(
            "Error fetching file over http `{}:\n{}\n",
            url,
            response.status()
        ))
    }
}

#[cfg(not(all(not(target_family = "wasm"), feature = "http")))]
pub fn download_url_to_file(_url: &str, _file: &str) -> Result<String, String> {
    Err(super::HTTP_SUPPORT_DISABLED.into())
}

pub fn json_from_meta_data(data: &[Variable]) -> Result<String, io::Error> {
    fn is_start_node(v: &Variable) -> bool {
        if let Variable::Array(ref arr) = *v {
            if let Variable::Str(ref t) = arr[2] {
                &**t == "start"
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_end_node(v: &Variable) -> bool {
        if let Variable::Array(ref arr) = *v {
            if let Variable::Str(ref t) = arr[2] {
                &**t == "end"
            } else {
                false
            }
        } else {
            false
        }
    }

    use piston_meta::json::write_string;
    use std::cmp::{max, min};
    use std::io::Write;

    let indent_offset = 0;
    let mut w: Vec<u8> = vec![];

    // Start indention such that it balances off to zero.
    let starts = data.iter().filter(|x| is_start_node(x)).count() as u32;
    let ends = data.iter().filter(|x| is_end_node(x)).count() as u32;
    let mut indent: u32 = max(starts, ends) - min(starts, ends);
    let mut first = true;
    for (i, d) in data.iter().enumerate() {
        let is_end = if is_end_node(d) {
            indent -= 1;
            true
        } else {
            false
        };
        let print_comma = !first && !is_end;
        if print_comma {
            writeln!(w, ",")?;
        } else if i != 0 {
            writeln!(w)?;
        }
        first = false;
        for _ in 0..indent_offset + indent {
            write!(w, " ")?;
        }
        if let Variable::Array(ref arr) = *d {
            let name = if let Variable::Str(ref t) = arr[3] {
                t
            } else {
                ""
            };
            if let Variable::Str(ref t) = arr[2] {
                match &***t {
                    "start" => {
                        first = true;
                        write_string(&mut w, name)?;
                        write!(w, ":{}", "{")?;
                        indent += 1;
                    }
                    "end" => {
                        write!(w, "{}", "}")?;
                    }
                    "bool" => {
                        if let Variable::Bool(val, _) = arr[4] {
                            write_string(&mut w, name)?;
                            write!(w, ":{}", val)?;
                        }
                    }
                    "f64" => {
                        if let Variable::F64(val, _) = arr[4] {
                            write_string(&mut w, name)?;
                            write!(w, ":{}", val)?;
                        }
                    }
                    "str" => {
                        if let Variable::Str(ref val) = arr[4] {
                            write_string(&mut w, name)?;
                            write!(w, ":")?;
                            write_string(&mut w, val)?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    writeln!(w)?;
    Ok(String::from_utf8(w).unwrap())
}
