mod utils;

use utils::{History, HOME_PATH};
use clap::{App, Arg};
use std::process::Command;

fn main() {
    let matches = App::new("Clipsway")
        .version("0.1.0")
        .arg(Arg::with_name("history")
            .short("H")
            .long("history")
            .help("Print clipboard history"))
        .arg(Arg::with_name("take")
            .short("t")
            .long("take")
            .takes_value(true)
            .value_name("NUMBER")
            .help("Take contents from history to clipboard"))
        .arg(Arg::with_name("store")
            .short("s")
            .long("store")
            .help("Store clipboard contents to history"))
        .arg(Arg::with_name("daemon")
            .short("d")
            .long("daemon")
            .help("Run clipsway daemon"))
        .arg(Arg::with_name("clear")
            .short("c")
            .long("clear")
            .help("Clear history"))
        .get_matches();

    let take_val = matches.value_of("take");

    if matches.is_present("store") {
        let mut hist = History::new();
        hist.store();
        hist.write();
    }

    else if matches.is_present("daemon") {
        Command::new("bash").arg("-c").arg(format!("wl-paste -w {}/.clipsway/clipsway --store", &*HOME_PATH)).spawn().unwrap();
    }

    else if matches.is_present("history") {
        let hist = History::new();
        hist.print();
    }

    else if matches.is_present("clear") {
        History::clear_static();
    }

    else if !(take_val.is_none()) {
        let mut hist = History::new();
        hist.take(take_val.unwrap());
        hist.write();
    }

    else {
        eprintln!("Try --help");
    }
}
