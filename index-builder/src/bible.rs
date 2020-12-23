use anyhow::{anyhow, Result};
use stats::{L0L1Stats, Stats};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use tantivy::doc;

mod index;
mod read_bible;
mod stats;

#[derive(Debug, StructOpt)]
#[structopt(name = "bible-indexer")]
pub struct Config {
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,

    #[structopt(
        short,
        long,
        help = r#"The internal name for this bible. Defaults to file's
            basename, lowercased and prefixed with 'bible-'.
            e.g. ESV.xml becomes "bible-esv""#
    )]
    name: Option<String>,

    #[structopt(
        short,
        long,
        help = r#"The title for this bible. Defaults to file's basename + " Bible""#
    )]
    title: Option<String>,

    #[structopt(short, long, help = r#"Defaults to ./data"#)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let config = Config::from_args();

    let Config {
        name,
        file,
        title: title_arg,
        output,
    } = config;

    let title = title_arg.unwrap_or_else(|| {
        let stem = file.file_stem().unwrap().to_string_lossy();
        if stem.ends_with("Bible") {
            return String::from(stem);
        }
        format!("{} Bible", stem)
    });

    let book_id =
        name.unwrap_or_else(|| format!("bible-{}", file.file_stem().unwrap().to_string_lossy()));

    let data_path = output.unwrap_or_else(|| std::env::current_dir().unwrap().join("data"));
    let ind =
        index::open_index(data_path.as_path()).map_err(|e| anyhow!("Opening index: {}", e))?;
    let mut writer = ind
        .writer(100000000)
        .map_err(|e| anyhow!("Creating writer: {}", e))?;
    let schema = ind.schema();

    let l0_field = schema.get_field("l0").unwrap();
    let l1_field = schema.get_field("l1").unwrap();
    let l2_field = schema.get_field("l2").unwrap();
    let text_field = schema.get_field("text").unwrap();
    let book_id_field = schema.get_field("book").unwrap();

    let mut stats = L0L1Stats::new(title);

    read_bible::read(&file, |passage| {
        stats.add(
            passage.book_index - 1,
            Some(passage.book.as_ref()),
            passage.chapter - 1,
            None,
            None,
            passage.text.as_ref(),
        );

        writer.add_document(doc!(
            book_id_field => book_id.clone(),
            l0_field => passage.book_index as u64 - 1,
            l1_field=> passage.chapter as u64 - 1,
            l2_field => passage.verse as u64 - 1,
            text_field => passage.text,
        ));

        Ok(())
    })?;

    writer.commit().map_err(|e| anyhow!("{}", e))?;

    let meta_file = File::create(format!(
        "{}/stats-{}.json",
        data_path.to_string_lossy(),
        book_id
    ))?;
    serde_json::to_writer(&meta_file, &stats)?;
    meta_file.sync_all()?;
    drop(meta_file);

    Ok(())
}
