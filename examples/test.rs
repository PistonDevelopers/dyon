extern crate dyon;

fn main() {
    match dyon::run("source/test.rs") {
        Err(err) => {
            println!("");
            println!(" --- ERROR --- ");
            println!("{}", err);
        }
        Ok(()) => {}
    }
}
