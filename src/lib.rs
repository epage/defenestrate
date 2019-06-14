#[macro_use]
extern crate serde_derive;

mod dict;

pub mod report;
pub mod tokens;

pub use crate::dict::*;

use std::fs::File;
use std::io::Read;

pub fn process_file(
    path: &std::path::Path,
    dictionary: &Dictionary,
    report: report::Report,
) -> Result<(), failure::Error> {
    let mut buffer = Vec::new();
    File::open(path)?.read_to_end(&mut buffer)?;
    for (line_idx, line) in grep_searcher::LineIter::new(b'\n', &buffer).enumerate() {
        let line_num = line_idx + 1;
        for token in tokens::Symbol::parse(line) {
            if let Ok(word) = std::str::from_utf8(token.token) {
                // Correct tokens as-is
                if let Some(correction) = dictionary.correct_str(word) {
                    let col_num = token.offset;
                    let msg = report::Message {
                        path,
                        line,
                        line_num,
                        col_num,
                        word,
                        correction,
                        non_exhaustive: (),
                    };
                    report(msg);
                }
            }
        }
    }

    Ok(())
}
