## Audio CD ripper

This is a small CLI application to rip audio CDs into local files. Currently, it only supports [FLAC](https://xiph.org/flac/) (and probably will stay this way).

It uses [rust-cd-da-reader](https://github.com/Bloomca/rust-cd-da-reader) library underneath, so it works on every major platform (Windows, macOS and Linux). It uses [MusicBrainz](https://musicbrainz.org/) for metadata tags and cover art, if possible.

By default, it will try to automatically detect a CD drive with an audio CD in it and will create a folder for the album from where you run it, but you can specify both a specific disk (with a `--disk` argument) and the output folder (with a `--out` argument).

## Install from crates.io

You can install it directly from [crates.io](https://crates.io/crates/audio-cd-ripper):

```sh
cargo install audio-cd-ripper
```

Then run:

```sh
audio-cd-ripper --help
audio-cd-ripper
```

## Run from source

If you want to run from source, clone this repository and run the application (you will need to have [Rust installed](https://www.rust-lang.org/tools/install)):

```sh
git clone git@github.com:Bloomca/audio-cd-ripper.git
cd .\audio-cd-ripper\ # (on Windows)
cargo run
```

This will automatically try to detect a CD-ROM with an audio CD; if there are multiple choices, it will try the first one. The output will look something like this:

```sh
user@mac:~/projects/audo-cd-ripper % cargo run
Drive with a CD: default drive
MusicBrainzId: pT9QJuVCB.sW2mIKmcUC7LK9ChU-
Found album Honeymoon by Lana Del Rey from 2015-09-18, XE release
There are 14 tracks total
Creating new folder for the album: /Users/vsevolodzaikov/projects/personal/audio-cd-ripper/Honeymoon
Missing 14 track(s), starting rip

Writing track #1: Honeymoon
  Reading and encoding: [05:51 / 05:51] 100.0%
  Successfully added metadata
  Successfully wrote the track #1: Honeymoon

...

Writing track #14: Don’t Let Me Be Misunderstood
  Reading and encoding: [03:04 / 03:04] 100.0%
  Successfully added metadata
  Successfully wrote the track #14: Don’t Let Me Be Misunderstood
All tracks were written successfully
Cover art saved to: /Users/vsevolodzaikov/projects/personal/audio-cd-ripper/Honeymoon/folder.jpg
Successfully saved the album data
Success!
```

It fetches a cover art if possible, and adds important [Vorbis comment](https://en.wikipedia.org/wiki/Vorbis_comment) metadata for artist, album, track number, track title, etc. It should be recognized correctly by any major music players, like [Foobar2000](https://www.foobar2000.org/).

## Why build it?

Just for fun. I already built [a library](https://github.com/Bloomca/rust-cd-da-reader) to read the CD-DA data, so the only missing piece is to write a music player :)
