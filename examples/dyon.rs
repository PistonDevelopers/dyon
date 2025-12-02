extern crate dyon;
extern crate read_token;

use std::io::Write;
use std::sync::Arc;

use dyon::{error, load, load_str, Module, Runtime};

fn main() {
    let mut file: Option<String> = None;
    let mut imports: Vec<Module> = vec![];

    let mut module = Module::new();
    let mut ctx = String::new();

    println!("=== Dyon 0.50 ===");
    println!("Type `help` for more information.");
    loop {
        if let Some(x) = file.as_ref() {
            print!("({}) ", x);
        }
        print!("> ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Expected input");

        let command = input.trim().to_string();

        if command == "\\" {
            input = String::new();
            let mut empty_lines = 0;
            loop {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).expect("Expected input");
                if line.trim() == "" {
                    empty_lines += 1;
                    if empty_lines >= 2 {break}
                } else {
                    if empty_lines > 0 {input.push_str("\n")}
                    empty_lines = 0;
                    input.push_str(&line);
                }
            }
        }

        match &*command {
            "bye" => break,
            "ctx" => {
                println!("{}", ctx);
                continue;
            }
            "clear" => {
                ctx = String::new();
                module = Module::new();
                file = None;
                continue;
            }
            "help" => {
                println!("{}", include_str!("../assets/repl/help.txt"));
                continue;
            }
            "save" => {
                if let Some(x) = file.as_ref() {
                    if std::fs::write(x, &ctx).is_err() {
                        println!("Saving failed.");
                    }
                } else {
                    println!("Could not save.\nUse `save \"<file>\"`.");
                }
                continue;
            }
            _ if command.starts_with("save ") => {
                let new_file = if let Some(x) = json_str(&command[5..]) {x}
                               else {
                                    println!("Saving failed.");
                                    continue;
                               };
                if std::fs::write(&new_file, &ctx).is_err() {
                    println!("Saving failed.");
                    continue;
                }
                file = Some(new_file);
                continue;
            }
            _ if command.starts_with("load ") => {
                println!("Loading...");

                let new_file = if let Some(x) = json_str(&command[5..]) {x}
                           else {
                                println!("Loading failed.");
                                continue;
                           };
                println!("  {}", new_file);

                ctx = match std::fs::read_to_string(&new_file) {
                    Ok(x) => x,
                    Err(_) => {
                        println!("Could not load {}", new_file);
                        continue;
                    }
                };
                file = Some(new_file);
                if reload(&ctx, &mut module, &imports) {
                    continue;
                }

                println!("Done!");
                continue;
            }
            _ if command.starts_with("import ") => {
                println!("Importing...");

                let file = if let Some(x) = json_str(&command[7..]) {x}
                           else {
                                println!("Import failed.");
                                continue;
                           };
                println!("  {}", file);
                let mut m = Module::new();
                if error(load(&file, &mut m)) {
                    continue;
                }
                imports.push(m);

                if reload(&ctx, &mut module, &imports) {
                    continue;
                }

                println!("Done!");
                continue;
            }
            _ if command.starts_with("call ") => {
                let f = if let Some(x) = json_str(&command[5..]) {x}
                        else {
                            println!("Could not call function.");
                            continue;
                        };
                input = format!("{{\n{}()\nreturn\n}}\n", f);
            }
            "" => {
                // Print separator for readability.
                print!("\n------------------------------------<o=o");
                println!("o=o>------------------------------------\n");
                continue;
            }
            _ => {}
        }

        let mut sub_module = Module::new();
        sub_module.import(&module);
        let res = load_str("dyonrepl", Arc::new(input.clone()), &mut sub_module);
        if res.is_err() {
            // Try evaluation expression.
            let code = format!("fn main() {{println({})}}", input);
            if error(run_str_with_module("dyonrepl", Arc::new(code), &module)) {
                error(res);
            }
        } else {
            println!("{}", input);

            ctx.push_str(&input);
            if reload(&ctx, &mut module, &imports) {
                continue;
            }
        }
    }
}

fn reload(ctx: &str, module: &mut Module, imports: &[Module]) -> bool {
    let mut new_module = Module::new();
    for m in imports {new_module.import(m)};
    if !error(load_str("dyonrep", Arc::new(ctx.into()), &mut new_module)) {
        *module = new_module;
        false
    } else {true}
}

fn run_str_with_module(
    source: &str,
    d: Arc<String>,
    module: &Module,
) -> Result<(), String> {
    let mut m = Module::new();
    m.import(module);
    load_str(source, d, &mut m)?;
    let mut runtime = Runtime::new();
    runtime.run(&Arc::new(m))?;
    Ok(())
}

// Parses a JSON string.
fn json_str(txt: &str) -> Option<String> {
    use read_token::ReadToken;
    let r = ReadToken::new(txt, 0);
    if let Some(range) = r.string() {
        if let Ok(txt) = r.parse_string(range.length) {
            Some(txt)
        } else {
            println!("ERROR:\nCould not parse string");
            None
        }
    } else {
        println!("ERROR:\nExpected string");
        None
    }
}
