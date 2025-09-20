mod album_writer;
mod get_drive_letter;
mod music_brainz;
mod read_drive;

fn main() {
    let drives = get_drive_letter::get_drive_letter();

    match drives {
        Ok(drive_letters) => {
            if drive_letters.len() == 1 {
                read_drive(drive_letters.first().unwrap());
            } else if drive_letters.is_empty() {
                // TODO: allow to enter it manually
                println!("Did not find any drive letters");
                std::process::exit(1);
            } else {
                println!("Found multiple drives: {:#?}", drive_letters);
                println!("Trying to use the first one");
                read_drive(drive_letters.first().unwrap());
            }
        }
        Err(_) => {
            // TODO: allow to enter it manually
            println!("Did not find any drive letters");
            std::process::exit(1);
        }
    }
}

fn read_drive(letter: &str) {
    let result = read_drive::read_drive(letter);

    if result.is_err() {
        println!("Error while reading drive {}", letter);
    } else {
        println!("Success!");
    }
}
