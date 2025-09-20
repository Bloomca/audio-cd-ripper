mod get_drive_letter;
mod read_drive;
mod music_brainz;
mod album_writer;

fn main() {
    let drives = get_drive_letter::get_drive_letter();

    match drives {
        Ok(drive_letters) => {
            if drive_letters.len() == 1 {
                read_drive(drive_letters.get(0).unwrap());
            } else if drive_letters.len() == 0 {
                // TODO: allow to enter it manually
                println!("Did not find any drive letters");
                std::process::exit(1);
            } else {
                println!("Found multiple drives: {:#?}", drive_letters);
                println!("Trying to use the first one");
                read_drive(drive_letters.get(0).unwrap());
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
