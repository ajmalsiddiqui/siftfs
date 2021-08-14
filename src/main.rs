use std::env;

use siftfs::sift::SiftFilesystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!(
                "Usage: {} <DIR> <MOUNTPOINT>",
                env::args().nth(0).unwrap()
            );
            return Ok(());
        }
    };
    let mountpoint = match env::args().nth(2) {
        Some(path) => path,
        None => {
            println!(
                "Usage: {} <DIR> <MOUNTPOINT>",
                env::args().nth(0).unwrap()
            );
            return Ok(());
        }
    };

    const FILE_REGEX: &str = r"^([A-Za-z]+)-([A-Za-z]+)-([0-9]+)\.([a-z]+)$";
    const FILE_FORMAT_STRING: &str = "{} - {} ({}).{}";
    const FILE_FORMAT_STRING_ARGS: &str = "1,2,3,4";

    let siftfs = SiftFilesystem::new(
        &dir,
        FILE_REGEX,
        FILE_FORMAT_STRING,
        FILE_FORMAT_STRING_ARGS,
    );

    fuse::mount(siftfs, &mountpoint, &[])?;

    Ok(())
}
