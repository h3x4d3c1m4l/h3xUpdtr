# h3xUpdtr

A [WIP, but currently very much non existent] Rust based software updater for Windows applications.

The goal of this tool is to allow updating supporting applications on file level for faster downloads of updates (or downgrades or any other kind of version change).

## Design (basically some my thoughts, but not really organized)

When you request a version change, the updater should detect which files you already have, which have changed and which are unnecessary now. Only the changed and missing files will be downloaded and installed. This should benefit update times and data usage when updating large applications (e.g. [MomentoBooth](https://github.com/momentobooth/momentobooth)).

*Version definition*: A YAML file containing the file list of a specific version. The file should be named by the version. For every file it contains the relative file path, sha256 of compressed file (for checking download validity), sha256 of decompressed file for checking whether the current file needs updating and unpacked file size to display progress.

It should be up to the implementor of h3xUpdtr to determine what the naming scheme of version should be. I will most likely stick to `stable` and Git revs when using it.

Update files should be packed by something like zstandard, which has good comp/decomp speed and pretty good compression rate as well. Naming of the files should be the sha256 of the (original/uncompressed) file with concatenated the type of compression, for easy lookup.

## Features

- [ ] Actually do stuff at all
- [ ] GUI ([fltk](https://crates.io/crates/fltk))
- [ ] TUI ([Ratatui](https://crates.io/crates/ratatui))
- [ ] CLI (TBD)
