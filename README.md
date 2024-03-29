# goto
## Example Usage
#### Add Bookmark
- `goto add crates.io` - Add bookmark for URL crates.io (protocol HTTPS is assumed unless specified)
- `goto add github.com git vcs` Add bookmark for github.com with tags "git" and "vcs"
#### Open Bookmarks
- `goto open rust` - Open the bookmark which matches the keywords best. If no match is bookmark is
found, the keywords will be used in a seach query instead with a search engine of choice
 (default is DuckDuckGo).

- `goto open rust crates` - Open best matching bookmark that conatins the tags _rust_ **and** _crates_.
#### Search & Edit Bookmarks
- `goto select -n 20 rust` - List the 20 first bookmarks with the tag "rust"
- `goto select -s 0.5 git` - List all bookmarks with the tag git and a matching score of at least 0.5

Editing a bookmark is then done by selecting it from the list and chosing the appropiate action.
## Bookmarks File
```yaml
url: https://github.com/
title: GitHub: Where the world builds software · GitHub
tags:
  - community
  - control
  - git
  - github
  - source
  - vcs
```

So the anatomy of the file is
- URL (required)
- Title (optional)
- Tags (optional)

The bookmarks file is saved under the path `[OS_DATA_DIR]/[DOMAIN]/[HASH_OF_URL].yaml`. So for
exmaple the file above would for most Linux users be saved under
`~/.local/share/goto/github.com/09a8b930c8b79e7c313e5e741e1d59c39ae91bc1f10cdefa68b47bf77519be57.yaml`.
This means that any further attempt to save a bookmark for the exact same URL would not create a new
bookmark, but rather merge with the existing one.

#### Deprecated JSON Support
An earlier version (0.2.0 and earlier) stored bookmark files in JSON instead of YAML.
Since version 0.3.0, YAML is used instead since it is more human readable and less prone to merge
conflicts, and when merge conflicts occur, they are easier to resolve. If you still have bookmarks
files in JSON, so you can migrate to the new YAML format by running `goto migrate` (requires feature
[migrate](README.md#migrate)).

## Building
To build and install run
```sh
cargo install --path .
```
or with make
```sh
make install
```

### Optional Features
#### migrate
Used for enabling the migrate subcommand (`goto migrate`) which migrates bookmarks files from the
old JSON format to the new YAML format.

#### git2
**Previously**, there was a feature called `git2` which would enable git synchronization for
bookmarks created by goto.

It did however never work satisfactory, and it did not have a good way of handling merge conflicts
either. This feature has been removed since of version 0.2.0, and it is recommended that any git
synchronization is now done manually.

To find were bookmarks are stored locally, run goto with the debug flag `goto --debug` and it will
the path to the goto data directory.