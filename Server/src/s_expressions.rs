use lsp_types::{SemanticToken, Range};
use pest_derive::Parser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "../sexpression.pest"]
pub struct SexParser;
   
pub fn parse(sample:&str,f: fn(Rule) -> u32) -> Result<Vec<(Range,Rule,lsp_types::SemanticToken)>,(String,lsp_types::Range)> {
    
    match SexParser::parse(
            Rule::expressions, 
            sample.into()
    ) {
        Ok(p) => { 
                
                let mut previous_range : Option<lsp_types::Range> = None;
                let mut last_line_start : usize = 1;
                let mut last_line_end: usize = 1;
                let mut last_start: usize = 1;
                let mut last_end: usize = 1;

                let data = 
                    p.flatten().map(|x|{
                        let span = x.as_span();
                        let start_pos = span.start_pos();
                        let end_pos = span.end_pos();
                        let (start_line,start_col) = start_pos.line_col();
                        let (end_line,end_col) = end_pos.line_col();
                        let range = lsp_types::Range {
                            start: lsp_types::Position::new(start_line as u32,start_col as u32),
                            end:   lsp_types::Position::new(end_line as u32,end_col as u32),
                        };
                        let mut corrected_start = u32::try_from(start_pos.line_col().1).unwrap();
                        if start_pos.line_col().0 == last_line_start {
                            corrected_start = corrected_start - (last_start as u32)
                        } else {
                            corrected_start = corrected_start - 1;
                        }                        
                        let this_line_start = start_pos.line_col().0;
                        let calculated_length = end_pos.pos() - start_pos.pos();
                        let token = SemanticToken { 
                            // `deltaLine`: token line number, relative to the previous token
                            // `deltaStart`: token start character, relative to the previous token 
                            //  (relative to 0 or the previous token's start if they are on the same line)
                            // `length`: the length of the token. A token cannot be multiline.
                            // `tokenType`: will be looked up in `SemanticTokensLegend.tokenTypes`
                            // `tokenModifiers`: each set bit will be looked up in `SemanticTokensLegend.tokenModifiers`
                            delta_line: (this_line_start - last_line_start) as u32,
                            delta_start: corrected_start ,
                            length: calculated_length as u32,
                            token_type: f(x.as_rule()), 
                            token_modifiers_bitset: 0 
                        };

                        (last_line_end,last_end) = end_pos.line_col();
                        (last_line_start,last_start) = start_pos.line_col();
                        previous_range = Some(range);
                        (range,x.as_rule(),token)
                    }).collect();

            Ok(data)

        },
        Err(x) => {
            
            let error_message = format!("{x:#}");
            match x.line_col {
                pest::error::LineColLocation::Span(start,end) => {
                    Err((
                        error_message,
                        lsp_types::Range {
                            start: lsp_types::Position::new(
                                start.0 as u32 - 1,start.1 as u32),
                            end: lsp_types::Position::new(
                                end.0 as u32 - 1,end.1 as u32)
                        }))
                }
                pest::error::LineColLocation::Pos(position) =>
                    Err((
                        error_message,
                        lsp_types::Range {
                            start: lsp_types::Position::new(position.0 as u32 - 1,position.1 as u32),
                            end: lsp_types::Position::new(position.0 as u32 - 1,position.1 as u32)
                        }))
                }
            }
        }
    }


pub fn get_token_at_position(tokens:Vec<(Range,Rule,lsp_types::SemanticToken)>,position:lsp_types::Position) -> Option<(Range,Rule,SemanticToken)> {
    
    let line = position.line + 1;
    let char = position.character + 1;

    let mut currently_closest : Option<(Range,Rule,SemanticToken)> = None;
    
    let mut filtered = 
        tokens.iter().filter(|(range,_rule,_token)|{    
            if range.start.line > line || (range.start.line == line && range.start.character > char) {
                return false
            }
            true
        });

    while let Some(current) = filtered.next() {
        match &currently_closest {
            Some(currently_closest_item) => {
                
                let previous_start = currently_closest_item.0.start;
                let previous_end = currently_closest_item.0.end;
                
                let start_pos = current.0.start;
                let end_pos = current.0.end;

                if start_pos >= previous_start || end_pos <= previous_end {
                    currently_closest = Some(*current)
                }

            },
            None => {
                currently_closest = Some(*current)
            },
        }
        

    }
    
    match currently_closest {
        Some(v) => {
            Some(v)
        },
        None => None ,
    }
}

pub fn get_token_info_at_position(p:Vec<(Range,Rule,lsp_types::SemanticToken)>,position:lsp_types::Position) -> Option<String> {
    match get_token_at_position(p,position) {
            Some(ooh) => Some(format!("{:?}",ooh.1)),
            None => None
    }    
}



