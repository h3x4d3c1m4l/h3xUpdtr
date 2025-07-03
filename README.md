# h3xUpdtr

A Rust-based software updater for (primarily) Windows applications.

The goal of this tool is to enable file-level updates for supported applications, allowing for faster downloads of updates, downgrades, or any other kind of version change.

⚠️ **Warning:** This software is in a very early, pre-release stage. Use at your own risk. I cannot be held responsible for accidentally overwritten files.

## Design

When you request a version change, the updater detects which files you already have, which files have changed, and which files are no longer needed. Only the changed or missing files are downloaded and installed. This approach reduces update time and data usage, especially for large applications (e.g., [MomentoBooth](https://github.com/momentobooth/momentobooth)).

**Version definition:**
A YAML file describes each version by listing the files it contains. The YAML should be named after the version identifier. For every file, it includes:

* The relative file path
* The SHA256 hash of the compressed file (to verify download integrity)
* The SHA256 hash of the decompressed file (to check if an update is needed)
* The uncompressed file size

It is up to the implementer to decide on the version naming scheme. I personally stick to `stable` and Git revision hashes.

Update files are currently compressed with Brotli, as it offers fast compression/decompression and decent compression ratios. The naming scheme of the update files consist of the actual SHA256 of the original (uncompressed) file, to allow easy lookup of the right file.

## Features

### Functionality

* [x] Create versions
* [x] Switch between versions
  * [ ] Detect and remove obsolete files
* [ ] Verify local files

### File Store Support

* [x] S3-compatible storage
* [ ] Azure Blob Storage
* [ ] FTP(S)
* [ ] SFTP, SCP

### Interface

* [x] CLI ([clap](https://crates.io/crates/clap), [console](https://crates.io/crates/console), [indicatif](https://crates.io/crates/indicatif))
* [ ] GUI ([fltk](https://crates.io/crates/fltk))
* [ ] TUI ([Ratatui](https://crates.io/crates/ratatui))

## Limitations

* Empty folders are not supported.
* Symlinks are ignored when creating versions and overwritten when switching versions.
