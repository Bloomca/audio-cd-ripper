## Audio CD ripper

This is a small CLI application to rip audio CDs into local files. Currently, it only supports [FLAC](https://xiph.org/flac/) (and probably will stay this way).

It uses [rust-cd-da-reader](https://github.com/Bloomca/rust-cd-da-reader) library underneath, so in theory it will be possible to support all major platforms (Windows, macOS and Linux), but right now it works only on Windows. It uses [MusicBrainz](https://musicbrainz.org/) for metadata tags and cover art, if possible. It does not support multi-CD releases.

To use it, you need to clone this repository and run the application, so you will need to have [Rust installed](https://www.rust-lang.org/tools/install). It should look like this:

```sh
git clone git@github.com:Bloomca/audio-cd-ripper.git
cd .\audio-cd-ripper\ # (on Windows)
cargo run
```

This will automatically try to detect a CD-ROM with an audio CD; if there are multiple choices, it will try the first one. The output will look something like this:

```powershell
PS C:\Users\myuser\projects\audio-cd-ripper> cargo run
Drive with a CD: \\.\E:
MusicBrainzId: 8QtEp4kVYQ9Aj4BtgduaXiTCqjE-
Found album Relationship of Command by At the Drive‐In from 2000-09-12, US release
There are 11 tracks total
Creating new folder for the album: C:\Users\myuser\projects\audio-cd-ripper\Relationship of Command
Writing a track #1: Arcarsenal
Successfully added metadata
Successfully wrote the track #1: Arcarsenal
Writing a track #2: Pattern Against User
Successfully added metadata
Successfully wrote the track #2: Pattern Against User
... # (cut tracks in-between for brevity)
Writing a track #11: Non‐Zero Possibility
Successfully added metadata
Successfully wrote the track #11: Non‐Zero Possibility
Cover art saved to: C:\Users\myuser\projects\audio-cd-ripper\Relationship of Command\folder.jpg
Successfully saved the album data
Success!
```

It fetches a cover art if possible, and adds important [Vorbis comment](https://en.wikipedia.org/wiki/Vorbis_comment) metadata for artist, album, track number, track title, etc. It should be recognized correctly by any major music players, like [Foobar2000](https://www.foobar2000.org/).

## Why build it?

Just for fun. I already built [a library](https://github.com/Bloomca/rust-cd-da-reader) to read the CD-DA data, so the only missing piece is to write a music player :)