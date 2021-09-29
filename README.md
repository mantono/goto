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
```json
{
    "url": "https://github.com/",
    "title": "GitHub: Where the world builds software · GitHub",
    "tags": [ "source", "vcs", "github", "git", "control", "community" ]
}
```

So the anatomy of the file is
- URL (required)
- Title (optional)
- Tags (optional)

The bookmarks file is saved under the path `[OS_DATA_DIR]/[DOMAIN]/[HASH_OF_URL].json`. So for
exmaple the file above would for most Linux users be saved under
`~/.local/share/goto/github.com/09a8b930c8b79e7c313e5e741e1d59c39ae91bc1f10cdefa68b47bf77519be57.json`.
This means that any further attempt to save a bookmark for the exact sam URL would not create a new
bookmark, but rather merge with the existing one.

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
By default, the `git2` feature is enabled. If you want, [you may chose to exclude this feature](https://doc.rust-lang.org/cargo/reference/features.html#command-line-feature-options) at
build time if you don't want the ability to synchronize your bookmarks with git.
