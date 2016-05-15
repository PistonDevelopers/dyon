use std::sync::Arc;
use std::io::{self, Read};
use std::fs::File;
use std::error::Error;
use piston_meta::*;

use Variable;

fn io_error(action: &str, file: &str, err: &io::Error) -> String {
    format!("IO Error when attempting to {} `{}`: {}\n{}", action, file, err.description(),
        match err.cause() {
            None => "",
            Some(cause) => cause.description()
        })
}

fn load_metarules_data(meta: &str, s: &str, file: &str, d: &str) -> Result<Vec<Variable>, String> {
    let rules = try!(syntax_errstr(&s).map_err(|err|
        format!("When parsing meta syntax in `{}`:\n{}", meta, err)));
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
        data.push(Variable::F64(range_token.offset as f64));
        data.push(Variable::F64(range_token.length as f64));
        match &range_token.data {
            &MetaData::Bool(ref name, val) => {
                data.push(Variable::Text(b.clone()));
                data.push(Variable::Text(name.clone()));
                data.push(Variable::Bool(val));
            }
            &MetaData::String(ref name, ref val) => {
                data.push(Variable::Text(s.clone()));
                data.push(Variable::Text(name.clone()));
                data.push(Variable::Text(val.clone()));
            }
            &MetaData::F64(ref name, val) => {
                data.push(Variable::Text(n.clone()));
                data.push(Variable::Text(name.clone()));
                data.push(Variable::F64(val));
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