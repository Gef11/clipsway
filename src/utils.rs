use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::str;
use lazy_static::lazy_static;
use std::env::var;
use std::process::Command;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref HOME_PATH: String = var("HOME").unwrap();
    pub static ref HIST_PATH: String = HOME_PATH.to_string() + "/.clipsway/history.ron";
    pub static ref IMAGE_PATH: String = HOME_PATH.to_string() + "/.clipsway/images";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History(Vec<(String, Vec<u8>)>);
impl History {
    pub fn new() -> Self {
        let f = File::open(&*HIST_PATH).unwrap();
        let history: History = ron::de::from_reader(f).unwrap();
        history
    }

    pub fn push(&mut self, mimetype: String, cont: Vec<u8>) {
        self.0.push((mimetype, cont));
    }

    pub fn remove(&mut self, num: usize) {
        self.0.remove(num);
    }

    pub fn take(&mut self, val: &str) {
        let num: usize;

        if val == "last" {
            num = self.0.len() - 1;
        } else {
            num = val.parse::<usize>().unwrap();
        }

        let (mime, cont) = self.0[num].to_owned();

        if &mime[..5] == "image" {
            let path = str::from_utf8(cont.as_slice()).unwrap();
            let image = Image::read(path);
            fs::remove_file(path).unwrap();

            Clipboard::set(mime, image);
        } else {
            Clipboard::set(mime, cont);
        }
        self.remove(num);
    }
    
    pub fn clear(&mut self) {
        self.0.clear();
        History::clear_static();
    }
    
    pub fn clear_static() {
        let mut f = File::create(&*HIST_PATH).unwrap();

        writeln!(&mut f, "([])").unwrap();

        for entry in fs::read_dir(&*IMAGE_PATH).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
    }

    pub fn store(&mut self) {
        let (mimetype, clip_cont) = Clipboard::get();

        if &mimetype[..5] == "image" {
            let image_path = format!("{}/{}", &*IMAGE_PATH, Image::get_num());
            self.push(mimetype, image_path.as_bytes().to_vec());
            Image::save(image_path.as_str(), clip_cont);
        } else {
            self.push(mimetype, clip_cont);
        }

        if self.0.len() > 1000 {
            self.remove(0);
        }
    }

    pub fn print(&self) {
        for (i, line) in self.0.iter().enumerate() {
            let (mimetype, line) = line;
            let line = str::from_utf8(line.as_slice()).unwrap();

            if &mimetype[..5] == "image" {
                println!("{}:", i);
                io::stdout().write_all(Command::new("img2sixel").arg(line).output().unwrap().stdout.as_slice()).unwrap();
            } else {
                println!("{}: {}", i, line);
            }
        }
    }

    pub fn write(&mut self) {
        let f = File::create(&*HIST_PATH).unwrap();
        ron::ser::to_writer(f, self).unwrap();
    }
}

pub struct Clipboard;
impl Clipboard {
    pub fn get() -> (String, Vec<u8>) {
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

    pub fn set(mimetype: String, cont: Vec<u8>) {
        use wl_clipboard_rs::copy::*;

        let mime = MimeType::Specific(mimetype);
        let opts = Options::new();

        opts.copy(Source::Bytes(&cont), mime).unwrap();
    }
}

pub struct Image;
impl Image {
    pub fn get_num() -> usize {
        let num = fs::read_dir(&*IMAGE_PATH).unwrap().map(|s| s.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>().last().unwrap_or(&"0".to_string()).parse::<usize>().unwrap();
        num + 1
    }

    pub fn save(path: &str, cont: Vec<u8>) {
        let mut f = File::create(path).unwrap();
        f.write_all(&cont).unwrap();
    }

    pub fn read(path: &str) -> Vec<u8> {
        let mut f = File::open(path).unwrap();
        let mut buffer = vec![];
        f.read_to_end(&mut buffer).unwrap();
        buffer
    }
}
