# goto
## Example Usage
#### Open/List Bookmarks
- `goto open rust` - List all bookmarks with the tag _rust_, or which contains rust in the URL or title. If only one bookmark is found it will be opened by the browser straight away. If several bookmarks are found, they will be listed and the user will be asked to chose which one to open in the browser. If no bookmarks is found, the keyword will be used in a seach query instead with a search engine of choice (default is DuckDuckGo).

- `goto open rust crates` - List all bookmarks that conatins the tags _rust_ **and** _crates_.
#### Search Bookmarks
#### Add Bookmark
`goto add crates.io`

#### Edit Bookmark
goto edit crates.io

## Bookmarks File
```
https://crates.io
crates.io: Rust Package Registry
rust crates packages registry
```

So the anatomy of the file is
- URL
- Title
- tag(s)