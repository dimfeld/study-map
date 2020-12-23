use anyhow::{Context, Result};
use quick_xml::events::Event;
use std::path::Path;

fn get_name<'a>(e: &'a quick_xml::events::BytesStart) -> String {
    e.attributes()
        .map(|a| a.unwrap())
        .find(|a| a.key == b"n")
        .map(|a| String::from_utf8_lossy(&a.unescaped_value().unwrap()).to_string())
        .unwrap()
}

pub struct Passage<'a> {
    pub book: &'a str,
    pub book_index: usize,
    pub chapter: usize,
    pub verse: usize,
    pub text: String,
}

pub fn read<F: FnMut(Passage) -> Result<()>>(path: &Path, mut callback: F) -> Result<()> {
    let mut reader = quick_xml::Reader::from_file(path)
        .with_context(|| format!("Failed to open file {:?}", path))?;

    let mut current_book = String::new();
    let mut current_chapter = 0;
    let mut current_verse = 0;

    let mut book_index = 0;

    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"b" => {
                    let new_book = get_name(e);
                    if new_book != current_book {
                        current_book = new_book;
                        book_index += 1;
                    }
                }
                b"c" => {
                    current_chapter = get_name(e).parse::<usize>()?;
                }
                b"v" => {
                    current_verse = get_name(e).parse::<usize>()?;
                }
                _ => (),
            },
            Ok(Event::Text(ref t)) => {
                if current_book.len() > 0 && current_chapter > 0 && current_verse > 0 {
                    let value = t.unescape_and_decode(&reader)?;
                    callback(Passage {
                        book: current_book.as_ref(),
                        book_index,
                        chapter: current_chapter,
                        verse: current_verse,
                        text: value,
                    })?;
                    // TODO keep statistics on book/chapter lengths
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => (),
        }
    }

    Ok(())
}
