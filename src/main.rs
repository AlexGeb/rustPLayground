extern crate anyhow;
extern crate graphql_client;
extern crate hello_macro;
extern crate hello_macro_derive;
extern crate log;
extern crate prettytable;
extern crate serde;
extern crate structopt;

mod request;

use anyhow::Result;
use hello_macro::HelloMacro;
use hello_macro_derive::HelloMacro;

#[derive(HelloMacro)]
struct Pancakes;

#[derive(HelloMacro)]
struct Beach;

fn main() -> Result<(), anyhow::Error> {
    Pancakes::hello_macro();
    Beach::hello_macro();
    request::request()
}
