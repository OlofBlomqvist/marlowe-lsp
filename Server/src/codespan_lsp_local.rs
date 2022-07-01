#![forbid(unsafe_code)]
use std::ops::Range;
use codespan_reporting::files::{Error, Files};
use lsp_types::{Position as LspPosition, Range as LspRange};

fn character_to_line_offset(line: &str, character: u32) -> Result<usize, Error> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();
            return Ok(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u32;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Ok(line_len)
    } else {
        Err(Error::ColumnTooLarge {
            given: character_offset as usize,
            max: line.len(),
        })
    }
}

pub fn position_to_byte_index<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    position: &LspPosition,
) -> Result<usize, Error>
where
    F: Files<'a> + ?Sized,
{
    let source = files.source(file_id)?;
    let source = source.as_ref();

    let line_span = files.line_range(file_id, position.line as usize).unwrap();
    let line_str = source.get(line_span.clone()).unwrap();

    let byte_offset = character_to_line_offset(line_str, position.character)?;

    Ok(line_span.start + byte_offset)
}

pub fn range_to_byte_span<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    range: &LspRange,
) -> Result<Range<usize>, Error>
where
    F: Files<'a> + ?Sized,
{
    Ok(position_to_byte_index(files, file_id, &range.start)?
        ..position_to_byte_index(files, file_id, &range.end)?)
}
