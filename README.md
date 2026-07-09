# imploder

This util creates and extracts zip archives with the DCL Implode compression and is used mainly for modding the game MDK 2.

Please note that this util intentionally doesn't add any info on modification date of the files when creating an archive. This is because setting "last_modified" adds a "UT extra field modtime" in addition to the regular "file last modified on (DOS date/time)" field. This means that files in an archive created by this util won't have any modification date written.

Here is some help:
```
Usage: imploder <COMMAND>

Commands:
  create   Create an archive from a directory: create <directory> <archive>
  extract  Extract an archive into a directory: extract <archive> <directory>
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Examples:
    imploder create directory/ out.zip
    imploder extract archive.zip directory/

please note that for `create` the directory contents are placed at the archive root
```
