use std::fs::{self, File};
use std::io::{self, Read, Write};
use clap::{App, Arg};
use lazy_static::lazy_static;
use std::env::var;
use std::process::Command;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref HOME_PATH: String = var("HOME").unwrap();
    static ref HIST_PATH: String = HOME_PATH.to_string() + "/.clipsway/history.ron";
    static ref IMAGE_PATH: String = HOME_PATH.to_string() + "/.clipsway/images";
}

#[derive(Debug, Serialize, Deserialize)]
struct History(Vec<(String, String)>);

fn main() {
    let matches = App::new("Clipsway")
        .version("0.1.0")
        .arg(Arg::with_name("history")
            .short("s")
            .long("history")
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
        let mut hist = history_read();

        if &mimetype[..5] == "image" {
            let image_path = format!("{}/{}", &*IMAGE_PATH, get_image_num());
            append_to_history((mimetype, image_path.to_owned()), &mut hist);
            save_image(image_path, clip_cont);
        } else {
            append_to_history((mimetype, String::from_utf8(clip_cont).unwrap()), &mut hist);
        }

        if hist.len() > 1000 {
            remove_from_history(0, &mut hist);
        }
    }

    else if matches.is_present("daemon") {
        Command::new("bash").arg("-c").arg(format!("wl-paste -w {}/.clipsway/clipsway --store", &*HOME_PATH)).spawn().unwrap();
    }

    else if matches.is_present("history") {
        for (i, line) in history_read().iter().enumerate() {
            let (mimetype, line) = line;
            if &mimetype[..5] == "image" {
                println!("{}:", i);
                io::stdout().write_all(Command::new("img2sixel").arg(line).output().unwrap().stdout.as_slice()).unwrap();
            } else {
                println!("{}: {}", i, line);
            }
        }
    }

    else if matches.is_present("clear") {
        let mut f = File::create(&*HIST_PATH).unwrap();
        let s = "([])";

        writeln!(&mut f, "{}", s).unwrap();

        for entry in fs::read_dir(&*IMAGE_PATH).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
    }

    else if !(take_val.is_none()) {
        take(take_val.unwrap());
    }

    else {
        eprintln!("Try --help");
    }
}

fn append_to_history(val: (String, String), history: &mut Vec<(String, String)>) {
    let f = File::create(&*HIST_PATH).unwrap();
    history.push(val);

    ron::ser::to_writer(f, &History{ 0: history.to_owned() }).unwrap();
}

fn remove_from_history(num: usize, hist: &mut Vec<(String, String)>) {
    let f = File::create(&*HIST_PATH).unwrap();
    hist.remove(num);

    ron::ser::to_writer(f, &History{ 0: hist.to_owned() }).unwrap();
}

fn get_clipboard_contents() -> (String, Vec<u8>) {
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
            (mimetype, contents)
        }
        Err(_err) => ("text/plain".to_string(), b"".to_vec())
    }
}

fn copy_to_clipboard(mimetype: String, cont: Vec<u8>) {
    use wl_clipboard_rs::copy::*;

    let mime = MimeType::Specific(mimetype);
    let opts = Options::new();

    opts.copy(Source::Bytes(&cont), mime).unwrap();
}

fn history_read() -> Vec<(String, String)> {
    let f = File::open(&*HIST_PATH).unwrap();
    let history: History = ron::de::from_reader(f).unwrap();
    history.0
}

fn take(val: &str) {
    let mut hist = history_read();
    let num: usize;

    if val == "last" {
        num = hist.len() - 1;
    } else {
        num = val.parse::<usize>().unwrap();
    }

    let (mime, cont) = hist[num].to_owned();

    if &mime[..5] == "image" {
        let image = read_image(&cont);
        fs::remove_file(&cont).unwrap();

        copy_to_clipboard(mime, image);
    } else {
        copy_to_clipboard(mime, cont.as_bytes().to_vec());
    }
    remove_from_history(num, &mut hist);
}

fn get_image_num() -> usize {
    let num = fs::read_dir(&*IMAGE_PATH).unwrap().map(|s| s.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>().last().unwrap_or(&"0".to_owned()).parse::<usize>().unwrap();
    num + 1
}

fn save_image(path: String, cont: Vec<u8>) {
    let mut f = File::create(path).unwrap();
    f.write_all(&cont).unwrap();
}

fn read_image(path: &str) -> Vec<u8> {
    let mut f = File::open(path).unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();
    buffer
}
