extern crate dynamo;

fn main() {
    match dynamo::run("source/test.rs") {
        Err(err) => {
            println!("");
            println!(" --- ERROR --- ");
            println!("{}", err);
        }
        Ok(()) => {}
    }
}
