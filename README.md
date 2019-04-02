## wikidot-to-html
[![Build Status](https://travis-ci.org/Nu-SCPTheme/wikidot-to-html.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/wikidot-to-html)

A Rust library and executable to convert Wikidot code into HTML. A reimplementation of the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from wikidot in a language that's not PHP.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).

### Compilation
```sh
$ cargo build --release
```

This will produce the binary `target/release/wikidot2html` and Rust library files.

### Testing
```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.
