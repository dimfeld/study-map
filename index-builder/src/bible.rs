use anyhow::{Context, Result};
use quick_xml::events::Event;
use std::path::PathBuf;
use structopt::StructOpt;
use tantivy::doc;

mod index;
mod stats;

#[derive(Debug, StructOpt)]
#[structopt(name = "bible-indexer")]
pub struct Config {
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,

    #[structopt(
        short,
        long,
        help = r#"The internal name for this bible. Defaults to file's basename, lowercased and prefixed with 'bible-'.
            e.g. ESV.xml becomes "bible-esv""#
    )]
    name: Option<String>,
}

fn get_name<'a>(e: &'a quick_xml::events::BytesStart) -> String {
    e.attributes()
        .map(|a| a.unwrap())
        .find(|a| a.key == b"n")
        .map(|a| String::from_utf8_lossy(&a.unescaped_value().unwrap()).to_string())
        .unwrap()
}

fn main() -> Result<()> {
    let config = Config::from_args();

    let book_id = config.name.unwrap_or_else(|| format!("bible-{}", config.file.file_stem().unwrap().to_string_lossy()));

    let mut reader = quick_xml::Reader::from_file(&config.file)
        .with_context(|| format!("Failed to open file {:?}", &config.file))?;

    let ind = index::open_index(None).map_err(|e| e.to_string())?;
    let writer = ind.writer(100000000).map_err(|e| e.to_string())?;

    let mut current_book : String;
    let mut current_chapter: u64;
    let mut current_verse: u64;

    let mut book_index: u64 = 0;

    let schema = ind.schema();

    let l0_field = schema.get_field("l0").unwrap();
    let l1_field = schema.get_field("l1").unwrap();
    let l2_field = schema.get_field("l2").unwrap();
    let text_field = schema.get_field("text").unwrap();
    let book_id_field= schema.get_field("book").unwrap();

    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"b" => {
                        let new_book = get_name(e);
                        if new_book != current_book {
                            current_book = new_book;
                            book_index += 1;
                        }
                    },
                    b"c" => {
                        current_chapter = get_name(e).parse::<u64>()?;
                    },
                    b"v" => {
                        current_verse = get_name(e).parse::<u64>()?;
                    },
                    _ => (),
                },
            },
            Ok(Event::Text(ref t)) => {
                if current_book.len() > 0 && current_chapter > 0 && current_verse > 0 {
                    let value = t.unescape_and_decode(&reader)?;
                    writer.add_document(doc!(
                        book_id_field => book_id,
                        l0_field => book_index,
                        l1_field=> current_chapter,
                        l2_field => current_verse,
                        text_field => value,
                    ));
                    // TODO keep statistics on book/chapter lengths
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => (),
        }
    }


    writer.commit().map_err(|e| e.to_string())?;

    Ok(())
}
