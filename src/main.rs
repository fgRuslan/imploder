use std::{fs::{self, File}, io::{Read, Seek, Write}, time::UNIX_EPOCH};
use pklib;
use rawzip::{ZipArchiveWriter, time::ZipDateTime};
use clap::{Parser, Subcommand};

const AFTER_TEST: &str = "
Examples:
    imploder directory/ out.zip

    please note that the contents of the \"directory\" will be at the root of the archive
";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(after_help = AFTER_TEST)]
struct Args {
    directory: String,
    output_archive: String
}

fn implode_test(input_dir: String, output_archive: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    let mut archive = rawzip::ZipArchiveWriter::new(&mut output);

    let mut file_list = Vec::new();
    let dir_heh = std::fs::read_dir(input_dir)?;

    for file_path in dir_heh {
        let shit = file_path.unwrap().path().display().to_string();
        file_list.push(shit);
    }

    for mut file in file_list {
        let mut curr_file = File::open(&file)?;
        let mut buf: Vec<u8> = Vec::new();
        let byte_count = curr_file.read_to_end(&mut buf)?;

        file = String::from(file.split('\\').last().unwrap());
        //println!("{}: {}", file, byte_count);

        let mod_time_unix = curr_file.metadata()?.modified()?.duration_since(UNIX_EPOCH)?.as_secs();
        let mod_time2 = ZipDateTime::from_unix(mod_time_unix.cast_signed());
        
        println!("processing {}...", file);
        process_file(&mut archive, &file.as_str(), mod_time2, &buf)?;
    }

    archive.finish()?;

    {
        let mut file = File::create(output_archive)?;
        file.write_all(output.as_slice())?;
    }

    Ok::<(), Box<dyn std::error::Error>>(())
}

fn process_file(archive: &mut ZipArchiveWriter<&mut Vec<u8>>, filename: &str, modification_time: ZipDateTime, data: &[u8]) -> Result<(), Box<dyn std::error::Error>>
{
    let (mut entry, config) = archive.new_file(filename)
    .last_modified(modification_time)
    .compression_method(rawzip::CompressionMethod::Terse)
    .start()?;

    let encoder = pklib::implode::ImplodeWriter::new(&mut entry, pklib::CompressionMode::Binary, pklib::DictionarySize::Size2K)?;

    let mut writer = config.wrap(encoder);

    std::io::copy(&mut &data[..], &mut writer)?;

    let (_, descriptor) = writer.finish()?;
    let compressed = entry.finish(descriptor)?;

    Ok::<(), Box<dyn std::error::Error>>(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    implode_test(args.directory, args.output_archive);

    Ok::<(), Box<dyn std::error::Error>>(())
}
