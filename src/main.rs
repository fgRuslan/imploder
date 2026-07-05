use std::{fs::File, io::{Read, Write}};
use flate2;
use pklib;
use rawzip::{ZipArchiveWriter, time::ZipDateTime};

fn implode_test() -> Result<(), Box<dyn std::error::Error>> {
    let data = b"Hello, world!";
    let modification_time = ZipDateTime::from_unix(1783257786);

    let mut output = Vec::new();
    let mut archive = rawzip::ZipArchiveWriter::new(&mut output);

    let (mut entry, config) = archive.new_file("test.txt")
    .last_modified(modification_time)
    .compression_method(rawzip::CompressionMethod::Terse)
    .start()?;

    let encoder = pklib::implode::ImplodeWriter::new(&mut entry, pklib::CompressionMode::Binary, pklib::DictionarySize::Size2K)?;

    let mut writer = config.wrap(encoder);

    std::io::copy(&mut &data[..], &mut writer)?;

    let (_, descriptor) = writer.finish()?;
    let compressed = entry.finish(descriptor)?;
    archive.finish()?;

    println!("{:?}", output);

    {
        let mut file = File::create("output.zip")?;
        file.write_all(output.as_slice())?;
    }

    Ok::<(), Box<dyn std::error::Error>>(())
}

fn process_file(mut archive: ZipArchiveWriter<&mut Vec<u8>>, filename: &str, modification_time: ZipDateTime, data) -> Result<(), Box<dyn std::error::Error>>
{
    let (mut entry, config) = archive.new_file(filename)
    .last_modified(modification_time)
    .compression_method(rawzip::CompressionMethod::Terse)
    .start()?;

    let encoder = pklib::implode::ImplodeWriter::new(&mut entry, pklib::CompressionMode::Binary, pklib::DictionarySize::Size2K)?;

    let mut writer = config.wrap(encoder);

    std::io::copy(&mut &data[..], &mut writer)?;

    let (_, descriptor) = writer.finish()?;

    Ok::<(), Box<dyn std::error::Error>>(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    implode_test();

    Ok::<(), Box<dyn std::error::Error>>(())
}
