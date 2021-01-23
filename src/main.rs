use std::fs::File;
use std::io::{Read, Write};
use clap::{App, Arg};
use lazy_static::lazy_static;
use std::env::var;
use std::process::Command;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref HOME_PATH: String = var("HOME").unwrap();
    static ref HIST_PATH: String = HOME_PATH.to_string() + "/.clipsway/history.ron";
}

#[derive(Debug, Serialize, Deserialize)]
struct History(Vec<Vec<String>>);

fn main() {
    let matches = App::new("Clipsway")
        .version("0.1.0")
        .arg(Arg::with_name("history")
            .short("s")
            .long("history")
            .takes_value(false)
            .help("Print clipboard history"))
        .arg(Arg::with_name("take")
            .short("t")
            .long("take")
            .takes_value(true)
            .value_name("NUMBER")
            .help("Take contents from history to clipboard"))
        .arg(Arg::with_name("store")
            .long("store")
            .help("Store clipboard contents to history"))
        .arg(Arg::with_name("daemon")
            .short("d")
            .long("daemon")
            .help("Run clipsway daemon"))
        .arg(Arg::with_name("clear")
            .long("clear")
            .help("Clear history"))
        .get_matches();

    let take_val = matches.value_of("take");

    if matches.is_present("store") {
        let (mimetype, clip_cont) = get_clipboard_contents();

        if clip_cont != "".to_string() {
            let mut hist = history_read();

            append_to_history(vec![mimetype, clip_cont], &mut hist);

            if hist.len() > 1000 {
                remove_from_history(0, &mut hist);
            }
        }
    }

    else if matches.is_present("daemon") {
        Command::new("bash").arg("daemon.sh").spawn().unwrap();
    }

    else if matches.is_present("history") {
        for (i, line) in history_read().iter().enumerate() {
            println!("{}: {}", i, line[1]);
        }
    }

    else if matches.is_present("clear") {
        let mut f = File::create(&*HIST_PATH).unwrap();

        let s = "([])";

        writeln!(&mut f, "{}", s).unwrap();
    }

    else if !(take_val.is_none()) {
        let mut hist = history_read();

        if take_val.unwrap() == "last".to_string() {
            let last = hist.last().unwrap();
            copy_to_clipboard(last[1].as_str(), last[0].as_str());
            remove_from_history(hist.len() - 1, &mut hist);
        } else {
            let num = take_val.unwrap().parse::<usize>().unwrap();
            copy_to_clipboard(hist[num][1].as_str(), hist[num][0].as_str());
            remove_from_history(num, &mut hist);
        }
    }

    else {
        eprintln!("Try --help");
    }
}

fn append_to_history(val: Vec<String>, history: &mut Vec<Vec<String>>) {
    let f = File::create(&*HIST_PATH).unwrap();
    history.push(val);

    ron::ser::to_writer(f, &History{ 0: history.to_owned() }).unwrap();
}

fn remove_from_history(num: usize, hist: &mut Vec<Vec<String>>) {
    let f = File::create(&*HIST_PATH).unwrap();
    hist.remove(num);

    ron::ser::to_writer(f, &History{ 0: hist.to_owned() }).unwrap();
}

fn get_clipboard_contents() -> (String, String) {
    use wl_clipboard_rs::paste::*;

    let raw_cont = get_contents(
        ClipboardType::Regular,
        Seat::Unspecified,
        MimeType::Any
    );
    match raw_cont {
        Ok((mut pipe, mimetype)) => {
            let mut contents = vec![];
            pipe.read_to_end(&mut contents).unwrap();
            (mimetype, String::from_utf8_lossy(&contents).to_string())
        }
        Err(_err) => ("text/plain".to_string(), "".to_string())
    }
}

fn copy_to_clipboard(cont: &str, mimetype: &str) {
    use wl_clipboard_rs::copy::*;

    let mime = MimeType::Specific(mimetype.to_string());
    let opts = Options::new();

    opts.copy(Source::Bytes(cont.as_bytes()), mime).unwrap();
}

fn history_read() -> Vec<Vec<String>> {
    let f = File::open(&*HIST_PATH).unwrap();
    let history: History = ron::de::from_reader(f).unwrap();
    history.0
}
