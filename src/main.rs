use std::{env, fs};

use chrono::{Date, DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use clap::Parser as _;
use parser::Parser;
use processing::calc_weekly_records;

mod ast;
mod parser;
mod processing;
mod settings;

#[derive(Debug, Clone, clap::Parser)]
struct Cli {
    path: String,
    #[clap(long)]
    today: Option<NaiveDateTime>,
}

fn main() {
    let cli = Cli::parse();

    let user_today = cli.today.unwrap_or(Local::now().naive_local());
    let user_today: DateTime<Local> = Local.from_local_datetime(&user_today).unwrap();

    let path = cli.path;
    let today = Local::now();

    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            println!("🛑 {:?}", err);
            return;
        }
    };

    let mut parser = Parser::new(source.chars().collect());
    let result = parser.parse_file();
    let ast = match result {
        Ok(ast) => ast,
        Err(err) => {
            println!("🛑 {:?}", err);
            return;
        }
    };

    fs::write("out.txt", format!("{:#?}", ast)).unwrap();
    match calc_weekly_records(&ast, user_today) {
        Ok(duration) => println!("{:#?}", duration),
        Err(err) => println!("🛑 {:?}", err),
    }
}
