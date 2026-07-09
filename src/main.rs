use clap::{Parser, Subcommand};
use pklib;
use rawzip::{
    ZipArchiveWriter,
    time::ZipDateTime,
};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

const AFTER_TEXT: &str = "\
Examples:
    imploder create directory/ out.zip
    imploder extract archive.zip directory/

please note that for `create` the directory contents are placed at the archive root
";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(after_help = AFTER_TEXT)]
struct Args {
    #[clap(subcommand)]
    subcommand: AppSubCommand,
}

#[derive(Subcommand)]
enum AppSubCommand {
    /// Create an archive from a directory: create <directory> <archive>
    Create {
        #[arg(value_name = "DIRECTORY")]
        directory: String,
        #[arg(value_name = "ARCHIVE")]
        archive: String,
    },
    /// Extract an archive into a directory: extract <archive> <directory>
    Extract {
        #[arg(value_name = "ARCHIVE")]
        archive: String,
        #[arg(value_name = "DIRECTORY")]
        directory: String,
    },
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
        let filename = file_path.unwrap().path().display().to_string();
        file_list.push(filename);
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
        let mod_time = ZipDateTime::from_unix(mod_time_unix.cast_signed());

        print!("imploding {}...", file);
        std::io::stdout().flush()?;
        let compressed_size = process_file(&mut archive, file.as_str(), mod_time, &buf)?;
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

fn explode(input_archive: String, output_dir: String) -> Result<(), Box<dyn std::error::Error>> {
    let drr = std::fs::create_dir(String::from("./") + &output_dir);

    match drr {
        Ok(_) => println!("Created dir {}", output_dir),
        Err(_) => println!("Seems like dir {} already exists", output_dir),
    }

    let path = Path::new(&input_archive);
    let archive_data = std::fs::read(path)?;

    let archive = rawzip::ZipArchive::from_slice(&archive_data)?;

    let entries = archive.entries();

    for entry in entries {
        let entry = entry?;
        let wayfinder = entry.wayfinder();

        let filename = entry.file_path();
        let normalized_filename = filename.try_normalize()?;
        let filename_ref = normalized_filename.as_ref();

        let local_entry = archive.get_entry(wayfinder)?;

        let mut output = Vec::new();

        println!("exploding {}...", filename_ref);

        let decompressor = pklib::explode::ExplodeReader::new(local_entry.data())?;

        let mut reader = local_entry.verifying_reader(decompressor);
        std::io::copy(&mut reader, &mut output)?;

        {
            let mut file = File::create(output_dir.clone() + "/" + filename_ref)?;
            file.write_all(output.as_slice())?;
            let modify_time = entry.last_modified();

            let _d = match modify_time {
                rawzip::time::ZipDateTimeKind::Utc(dt) => {
                    UNIX_EPOCH + Duration::from_secs(dt.to_unix() as u64)
                }
                _ => UNIX_EPOCH,
            };

            // due to that problem with modified_time when creating archives, i decided to add this check
            // if the modified time is not set for a file in the archive, _d becomes 1980-01-01
            // so instead of having extracted files have nothing as their modification date i decided to only
            // modify that time if the archive contains time info for those files
            if modify_time.year() > 1980 {
                file.set_modified(_d)?;
            }
        }
    }

    Ok::<(), Box<dyn std::error::Error>>(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match args.subcommand {
        AppSubCommand::Create { directory, archive } => {
            implode_test(directory, archive)?;
        }
        AppSubCommand::Extract { archive, directory } => {
            explode(archive, directory)?;
        }
    }

    Ok::<(), Box<dyn std::error::Error>>(())
}
