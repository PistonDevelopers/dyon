use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use std::sync::Arc;

use range::Range;
use read_token::{NumberSettings, ReadToken};

use super::io::io_error;

use Variable;

type Strings = HashSet<Arc<String>>;

/// Loads data from a file.
#[cfg(feature = "file")]
pub fn load_file(file: &str) -> Result<Variable, String> {
    let mut data_file = try!(File::open(file).map_err(|err| io_error("open", file, &err)));
    let mut d = String::new();
    try!(data_file.read_to_string(&mut d).map_err(|err| io_error("read", file, &err)));
    load_data(&d)
}

#[cfg(not(feature = "file"))]
pub fn load_file(_: &str) -> Result<Variable, String> {
    Err(super::FILE_SUPPORT_DISABLED.into())
}

/// Loads data from text.
pub fn load_data(data: &str) -> Result<Variable, String> {
    let mut read = ReadToken::new(data, 0);
    let mut strings: Strings = HashSet::new();
    opt_w(&mut read);
    expr(&mut read, &mut strings, data)
}

static NUMBER_SETTINGS: NumberSettings = NumberSettings {
    allow_underscore: true,
};

const SEPS: &str = &"(){}[],.:;\n\"\\";

fn expr(
    read: &mut ReadToken,
    strings: &mut Strings,
    data: &str
) -> Result<Variable, String> {
    if let Some(range) = read.tag("{") {
        // Object.
        *read = read.consume(range.length);
        return object(read, strings, data);
    }
    if let Some(range) = read.tag("[") {
        // Array.
        *read = read.consume(range.length);
        return array(read, strings, data);
    }
    if let Some(range) = read.tag("(") {
        // Vec4.
        *read = read.consume(range.length);
        return vec4(read, data);
    }
    if let Some(range) = read.tag("#") {
        use read_color::rgb_maybe_a;

        // Color.
        *read = read.consume(range.length);
        let (range, _) = read.until_any_or_whitespace(SEPS);
        let val = read.raw_string(range.length);
        if let Some((rgb, a)) = rgb_maybe_a(&mut val.chars()) {
            let v = [
                        f32::from(rgb[0]) / 255.0,
                        f32::from(rgb[1]) / 255.0,
                        f32::from(rgb[2]) / 255.0,
                        f32::from(a.unwrap_or(255)) / 255.0
                    ];
            return Ok(Variable::Vec4(v));
        } else {
            return Err(error(range, "Expected hex color in format `FFFFFF`or `FFFFFFFF`", data));
        }
    }
    if let Some(range) = read.tag("link") {
        // Link.
        *read = read.consume(range.length);
        return link(read, strings, data);
    }
    // Text.
    if let Some(range) = read.string() {
        match read.parse_string(range.length) {
            Ok(s) => {
                *read = read.consume(range.length);
                return Ok(Variable::Text(
                    if let Some(s) = strings.get(&s) {
                        s.clone()
                    } else {
                        Arc::new(s)
                    }
                ));
            }
            Err(err_range) => {
                let (range, err) = err_range.decouple();
                return Err(error(range, &format!("{}", err), data))
            }
        }
    }
    // Number.
    if let Some(range) = read.number(&NUMBER_SETTINGS) {
        match read.parse_number(&NUMBER_SETTINGS, range.length) {
            Ok(val) => {
                *read = read.consume(range.length);
                return Ok(Variable::f64(val));
            }
            Err(err) => return Err(error(range, &format!("{}", err), data)),
        }
    }
    // Boolean.
    if let Some(range) = read.tag("false") {
        *read = read.consume(range.length);
        return Ok(Variable::bool(false));
    }
    if let Some(range) = read.tag("true") {
        *read = read.consume(range.length);
        return Ok(Variable::bool(true));
    }
    // Option.
    if let Some(range) = read.tag("none()") {
        *read = read.consume(range.length);
        return Ok(Variable::Option(None));
    }
    if let Some(range) = read.tag("some(") {
        *read = read.consume(range.length);
        opt_w(read);
        let res = try!(expr(read, strings, data));
        opt_w(read);
        return if let Some(range) = read.tag(")") {
            *read = read.consume(range.length);
            Ok(Variable::Option(Some(Box::new(res))))
        } else {
            Err(error(read.start(), "Expected `)`", data))
        }
    }
    Err(error(read.start(), "Reached end of file", data))
}

fn object(
    read: &mut ReadToken,
    strings: &mut Strings,
    data: &str
) -> Result<Variable, String> {
    use std::collections::HashMap;

    let mut res: HashMap<Arc<String>, Variable> = HashMap::new();
    let mut was_comma = false;
    loop {
        opt_w(read);

        if let Some(range) = read.tag("}") {
            *read = read.consume(range.length);
            break;
        }

        if !res.is_empty() && !was_comma {
            return Err(error(read.start(), "Expected `,`", data));
        }

        let key: Arc<String>;
        if let Some(range) = read.string() {
            match read.parse_string(range.length) {
                Ok(s) => {
                    // Use reference to existing string to reduce memory.
                    key = if let Some(s) = strings.get(&s) {
                        s.clone()
                    } else {
                        Arc::new(s)
                    };
                    *read = read.consume(range.length);
                }
                Err(err_range) => {
                    let (range, err) = err_range.decouple();
                    return Err(error(range, &format!("{}", err), data))
                }
            }
        } else {
            let (range, _) = read.until_any_or_whitespace(SEPS);
            if range.length == 0 {
                return Err(error(range, "Expected key", data));
            } else {
                let k = read.raw_string(range.length);
                // Use reference to existing string to reduce memory.
                key = if let Some(s) = strings.get(&k) {
                    s.clone()
                } else {
                    Arc::new(k)
                };
                *read = read.consume(range.length);
            };
        }

        opt_w(read);

        if let Some(range) = read.tag(":") {
            *read = read.consume(range.length);
        } else {
            return Err(error(read.start(), "Expected `:`", data));
        }

        opt_w(read);

        res.insert(key, try!(expr(read, strings, data)));

        was_comma = comma(read);
    }
    Ok(Variable::Object(Arc::new(res)))
}

fn array(
    read: &mut ReadToken,
    strings: &mut Strings,
    data: &str
) -> Result<Variable, String> {
    use std::sync::Arc;

    let mut res = vec![];
    let mut was_comma = false;
    loop {
        opt_w(read);

        if let Some(range) = read.tag("]") {
            *read = read.consume(range.length);
            break;
        }

        if !res.is_empty() && !was_comma {
            return Err(error(read.start(), "Expected `,`", data));
        }

        res.push(try!(expr(read, strings, data)));
        was_comma = comma(read);
    }
    Ok(Variable::Array(Arc::new(res)))
}

fn link(
    read: &mut ReadToken,
    strings: &mut Strings,
    data: &str
) -> Result<Variable, String> {
    use Link;

    opt_w(read);

    if let Some(range) = read.tag("{") {
        *read = read.consume(range.length);
    } else {
        return Err(error(read.start(), "Expected `{`", data));
    }

    let mut link = Link::new();

    opt_w(read);

    loop {
        opt_w(read);

        if let Some(range) = read.tag("}") {
            *read = read.consume(range.length);
            break;
        }

        match link.push(&try!(expr(read, strings, data))) {
            Ok(()) => {}
            Err(err) => return Err(err),
        };
    }
    Ok(Variable::Link(Box::new(link)))
}

fn vec4(read: &mut ReadToken, data: &str) -> Result<Variable, String> {
    let x = if let Some(range) = read.number(&NUMBER_SETTINGS) {
        match read.parse_number(&NUMBER_SETTINGS, range.length) {
            Ok(x) => {
                *read = read.consume(range.length);
                x
            }
            Err(err) => return Err(error(range, &format!("{}", err), data)),
        }
    } else {
        return Err(error(read.start(), "Expected x component", data));
    };
    comma(read);
    let y = if let Some(range) = read.number(&NUMBER_SETTINGS) {
        match read.parse_number(&NUMBER_SETTINGS, range.length) {
            Ok(y) => {
                *read = read.consume(range.length);
                y
            }
            Err(err) => return Err(error(range, &format!("{}", err), data)),
        }
    } else {
        return Err(error(read.start(), "Expected y component", data));
    };
    let (z, w) = if comma(read) {
        if let Some(range) = read.number(&NUMBER_SETTINGS) {
            match read.parse_number(&NUMBER_SETTINGS, range.length) {
                Ok(z) => {
                    *read = read.consume(range.length);
                    comma(read);
                    if let Some(range) = read.number(&NUMBER_SETTINGS) {
                        match read.parse_number(&NUMBER_SETTINGS, range.length) {
                            Ok(w) => {
                                *read = read.consume(range.length);
                                (z, w)
                            }
                            Err(err) => return Err(error(range, &format!("{}", err), data)),
                        }
                    } else { (z, 0.0) }
                }
                Err(err) => return Err(error(range, &format!("{}", err), data)),
            }
        } else { (0.0, 0.0) }
    } else { (0.0, 0.0) };
    opt_w(read);
    if let Some(range) = read.tag(")") {
        *read = read.consume(range.length);
    } else {
        return Err(error(read.start(), "Expected `)`", data));
    }
    Ok(Variable::Vec4([x as f32, y as f32, z as f32, w as f32]))
}

/// Reads optional whitespace including comments.
fn opt_w(read: &mut ReadToken) {
    loop {
        let start = *read;
        let range = read.whitespace();
        *read = read.consume(range.length);

        // Single line comment.
        if let Some(range) = read.tag("//") {
            *read = read.consume(range.length);
            let (range, _) = read.until_any("\n");
            *read = read.consume(range.length);
        }

        multi_line_comment(read);

        if read.subtract(&start).length == 0 { break; }
    }
}

fn multi_line_comment(read: &mut ReadToken) {
    // Multi-line comment.
    if let Some(range) = read.tag("/*") {
        *read = read.consume(range.length);
        let (range, _) = read.until_any("*/");
        *read = read.consume(range.length);
        loop {
            let start = *read;

            if read.tag("*/").is_none() {
                if let Some(range) = read.tag("*") {
                    *read = read.consume(range.length);
                    let (range, _) = read.until_any("*/");
                    *read = read.consume(range.length);
                }
            }

            let start_multi_line = *read;
            multi_line_comment(read);
            if read.subtract(&start_multi_line).length > 0 {
                let (range, _) = read.until_any("*/");
                *read = read.consume(range.length);
            } else {
                *read = start_multi_line;
                if let Some(range) = read.tag("/") {
                    *read = read.consume(range.length);
                    let (range, _) = read.until_any("*/");
                    *read = read.consume(range.length);
                }
            }

            if read.subtract(&start).length == 0 { break; }
        }

        if let Some(range) = read.tag("*/") {
            *read = read.consume(range.length);
        }
    }
}

/// Reads comma.
fn comma(read: &mut ReadToken) -> bool {
    let mut res = false;
    opt_w(read);
    if let Some(range) = read.tag(",") {
        *read = read.consume(range.length);
        res = true;
    }
    opt_w(read);
    res
}

/// Generates error message using Piston-Meta's error handler.
fn error(range: Range, msg: &str, data: &str) -> String {
    use piston_meta::ParseErrorHandler;

    let mut handler = ParseErrorHandler::new(data);
    let mut buf: Vec<u8> = vec![];
    handler.write_msg(&mut buf, range, msg).unwrap();
    String::from_utf8(buf).unwrap()
}
