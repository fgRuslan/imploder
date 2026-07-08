use clap::Parser;
use pklib;
use rawzip::{ZipArchiveWriter, time::ZipDateTime};
use std::{
    fs::File,
    io::{Read, Write},
    time::UNIX_EPOCH,
};

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
    output_archive: String,
}

#[test]
fn test_large_file_implode() {
    let data = vec![0u8; 10_000_000];
    let compressed = pklib::implode_bytes(
        &data,
        pklib::CompressionMode::Binary,
        pklib::DictionarySize::Size4K,
    )
    .unwrap();
    let decompressed = pklib::explode_bytes(&compressed).unwrap();
    assert_eq!(data.len(), decompressed.len());
    assert_eq!(data, decompressed);
}

fn implode_test(
    input_dir: String,
    output_archive: String,
) -> Result<(), Box<dyn std::error::Error>> {
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
        let source_byte_count = curr_file.read_to_end(&mut buf)?;

        file = String::from(file.split('\\').last().unwrap());

        let mod_time_unix = curr_file
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let mod_time2 = ZipDateTime::from_unix(mod_time_unix.cast_signed());

        print!("imploding {}...", file);
        let compressed_size = process_file(&mut archive, &file.as_str(), mod_time2, &buf)?;
        println!("{} -> {} bytes", source_byte_count, compressed_size);
    }

    archive.finish()?;

    println!("writing the archive to disk...");

    {
        let mut file = File::create(output_archive)?;
        file.write_all(output.as_slice())?;
    }

    println!("done!");

    Ok::<(), Box<dyn std::error::Error>>(())
}

fn process_file(
    archive: &mut ZipArchiveWriter<&mut Vec<u8>>,
    filename: &str,
    modification_time: ZipDateTime,
    data: &[u8],
) -> Result<u64, Box<dyn std::error::Error>> {
    let (mut entry, config) = archive
        .new_file(filename)
        //setting last_modified sets the "UT extra field modtime" which is fine for pkzip but breaks mdk2
        //.last_modified(modification_time)
        .compression_method(rawzip::CompressionMethod::Terse)
        .start()?;

    let encoder = pklib::implode::ImplodeWriter::new(
        &mut entry,
        pklib::CompressionMode::Binary,
        pklib::DictionarySize::Size4K,
    )?;

    let mut writer = config.wrap(encoder);

    std::io::copy(&mut &data[..], &mut writer)?;

    let (_, descriptor) = writer.finish()?;
    let compressed = entry.finish(descriptor)?;

    Ok::<u64, Box<dyn std::error::Error>>(compressed)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    implode_test(args.directory, args.output_archive)?;

    Ok::<(), Box<dyn std::error::Error>>(())
}
