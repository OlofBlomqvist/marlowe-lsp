#![feature(start)]

mod codespan_lsp_local;
use codespan::FileId;
use codespan_lsp_local::{range_to_byte_span};

use std::{collections::HashMap, sync::Mutex};
use serde_json::Value;
use tower_lsp::{ jsonrpc::{Result}, Client, LanguageServer, LspService, Server };
use tower_lsp::lsp_types::*;

#[derive(Debug)]
struct MyLSPServer {
    client: Client,
    state: Mutex<State>
}

#[derive(Debug)]
struct State {
    sources: HashMap<Url, FileId>,
    sexpression_asts: HashMap<Url, Vec<(Range,sex::Rule,SemanticToken)>>,
    marlowe_asts: HashMap<Url, Vec<(Range,marlowe_lang::parsing::Rule,SemanticToken)>>,
    files: codespan::Files<String>,
    marlowe_parser_error: Option<(String,Range)>,
    sexpression_parser_error: Option<(String,Range)>
    
}

#[tower_lsp::async_trait]
impl LanguageServer for MyLSPServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(false),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                document_highlight_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions { 
                            text_document_registration_options: TextDocumentRegistrationOptions{ 
                                document_selector: None
                            }, 
                            semantic_tokens_options: SemanticTokensOptions{ 
                                work_done_progress_options: WorkDoneProgressOptions{ 
                                    work_done_progress: Some(false)
                                 },
                                legend: SemanticTokensLegend { 
                                    token_types: vec![
                                        SemanticTokenType::VARIABLE,
                                        SemanticTokenType::STRING,
                                        SemanticTokenType::NUMBER ,
                                        SemanticTokenType::STRUCT
                                    ], 
                                    token_modifiers: vec![
                                        SemanticTokenModifier::STATIC
                                    ]
                                }, 
                                range: Some(false), 
                                full: Some(SemanticTokensFullOptions::Bool(true)) 
                        },
                            static_registration_options: StaticRegistrationOptions::default()
                        }
                    )
                ),
                hover_provider: Some(HoverProviderCapability::Simple(true)), 
                
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        
        let state = self.state.lock().unwrap();
        match state.marlowe_asts.get(&params.text_document_position_params.text_document.uri) {
            Some(token_list) => {
                let closest = marlowe_lang::parsing::Rule::get_token_info_at_position(
                    token_list.to_vec(),
                    params.text_document_position_params.position,
                    |r| match r {
                        marlowe_lang::parsing::Rule::Notify |
                        marlowe_lang::parsing::Rule::Choice |
                        marlowe_lang::parsing::Rule::Deposit => String::from("Contracts in Marlowe run on a blockchain, but need to interact with the off-chain world. The parties to the contract, whom we also call the participants, can engage in various actions: they can be asked to deposit money, or to make a choice between various alternatives. A notification of an external value (also called an oracle value), such as the current price of a particular commodity, is the other possible form of input."),
                        marlowe_lang::parsing::Rule::Case => String::from("A When contract contains a collection of cases. Each case has the form Case action next where action is an Action and next a continuation (another contract). When a particular action happens, the state is updated accordingly and the contract will continue as the corresponding continuation next."),
                        marlowe_lang::parsing::Rule::Bound => String::from("A choice is made for a particular id with a list of bounds on the values that are acceptable. For example, [Bound 0 0, Bound 3 5] offers the choice of one of 0, 3, 4 and 5."),
                        marlowe_lang::parsing::Rule::Party |
                        marlowe_lang::parsing::Rule::PK |
                        marlowe_lang::parsing::Rule::Role => String::from("A Party is represented as either a public key hash or a role name. In order to progress a Marlowe contract, a party must provide an evidence. For PK party that would be a valid signature of a transaction signed by a private key of a public key that hashes to party’s PubKeyHash, similarly to Bitcoin’s Pay to Public Key Hash mechanism. For a Role party the evidence is spending a role token within the same transaction, usually to the same owner.\n
                        So, Role parties will look like (Role \"alice\"), (Role \"bob\") and so on."),
                        marlowe_lang::parsing::Rule::ChoiceId => String::from("Choices – of integers – are identified by ChoiceId which combines a name for the choice with the Party who had made the choice"),
                        marlowe_lang::parsing::Rule::TimeIntervalStart |
                        marlowe_lang::parsing::Rule::ConstantParam |
                        marlowe_lang::parsing::Rule::Constant |
                        marlowe_lang::parsing::Rule::MulValue |
                        marlowe_lang::parsing::Rule::DivValue |
                        marlowe_lang::parsing::Rule::SubValue |
                        marlowe_lang::parsing::Rule::TimeIntervalEnd => String::from("A Value encompasses Ada, fungible tokens (think currencies), non-fungible tokens (a custom token that is not interchangeable with other tokens), and more exotic mixed cases."),
                        marlowe_lang::parsing::Rule::TimeInterval |
                        marlowe_lang::parsing::Rule::TimeParam |
                        marlowe_lang::parsing::Rule::TimeConstant => String::from("Timeout is the slot number after which the When will no longer accept any new events: Case branches will become unusable, and the contract will continue as specified by the timeout continuation.
                        Timeouts accept templates, this means that instead of writing a specific slot number it is possible to fill Timeouts by using a template parameter that can be filled just before deploying or simulating the contract, for example: TimeParam \"maturityDate\""),
                        marlowe_lang::parsing::Rule::Close |
                        marlowe_lang::parsing::Rule::Pay |
                        marlowe_lang::parsing::Rule::Let |
                        marlowe_lang::parsing::Rule::If |
                        marlowe_lang::parsing::Rule::Assert |
                        marlowe_lang::parsing::Rule::When => String::from("Marlowe has six ways of building contracts. Five of these – Pay, Let, If, When and Assert – build a complex contract from simpler contracts, and the sixth, Close, is a simple contract. At each step of execution, as well as returning a new state and continuation contract, it is possible that effects – payments – and warnings can be generated too."),
                        marlowe_lang::parsing::Rule::Token => 
                            String::from("A Marlowe Account holds amounts of multiple currencies and/or fungible and non-fungible tokens. A concrete amount is indexed by a Token, which is a pair of CurrencySymbol and TokenName."),
                        _ => format!("{r:?}")
                    }
                );
                match closest {
                    Some(v) => {
                        Ok(
                            Some(
                                Hover { 
                                    contents: HoverContents::Markup(
                                            MarkupContent {
                                                kind: MarkupKind::PlainText,
                                                value: v
                                            }
                                    ),
                                    range: Some(
                                        Range::new(
                                            params.text_document_position_params.position,
                                            params.text_document_position_params.position,
                                        )
                                    )
                                }
                            )
                        )
                    },
                    None => Ok(None),
                }
            },
            None => {
                Ok(None)
            },
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        
        let state = self.state.lock().unwrap();

        match state.sexpression_asts.get(&params.text_document.uri) {
            Some(token_list) => {
                Ok(Some(SemanticTokensResult::Tokens(SemanticTokens{
                    result_id: Some("FULL".into()),
                    data: token_list.iter().map(|x|x.2).collect()
                })))
            },
            None => {
                Ok(None)
            },
        }
    }


    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        
        self.client
            .log_message(MessageType::INFO, "highlighting")
            .await;        

        let mut state = self.state.lock().unwrap();

        let toks = 
            state.marlowe_asts.get_mut(
                &params.text_document_position_params.text_document.uri);

        match toks {
            Some(tokens) => {
                        
                let closest = 
                    marlowe_lang::parsing::Rule::get_token_at_position(
                        tokens.to_vec(),params.text_document_position_params.position
                    );
                match closest {
                    Some((a,_b,_c)) => {
                        Ok(Some(vec![
                            DocumentHighlight { 
                                range: a,
                                kind: Some(DocumentHighlightKind::TEXT)
                            }]))
                    }
                    None => Ok(None)
                }
                
            },
            None => Ok(None),
        }

    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {  Ok(()) }
    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {}
    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {}
    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {}

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;
        
        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        
        let result = {   
            let mut state = self.state.lock().unwrap();
            get_or_insert_document(&mut state, &params.text_document);
            get_diagnostics(&mut state)
        };
        self.client.publish_diagnostics(
            params.text_document.uri.clone(), 
            result,
            None
        ).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let result = {
            let mut state = self.state.lock().unwrap();
            update_document(&mut state, &params.text_document.uri, params.content_changes);
            get_diagnostics(&mut state)
        };
        self.client.publish_diagnostics(
            params.text_document.uri, 
            result, 
            None).await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut state = self.state.lock().unwrap();
        state.marlowe_asts.remove(&params.text_document.uri);
    }

    async fn completion(&self, _completion_params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(None)
    }
}

fn get_or_insert_document(state: &mut State, document: &TextDocumentItem) -> FileId {
    if let Some(id) = state.sources.get(&document.uri) {
        *id
    } else {

        let id = state
            .files
            .add(document.uri.to_string(), document.text.clone());

        state.sources.insert(document.uri.clone(), id);
        
        update_asts(
            document.text.clone(), 
            state, 
            document.uri.clone()
        );
        id

    }
}

fn update_document(
    state: &mut State,
    url: &Url,
    changes: Vec<TextDocumentContentChangeEvent>,
) -> FileId {
    let id = *state.sources.get(&url).unwrap();
    let mut source = state.files.source(id).to_owned();
    for change in changes {
        if let (None, None) = (change.range, change.range_length) {
            source = change.text;
        } else if let Some(range) = change.range {
            let span = range_to_byte_span(
                &state.files, 
                id, 
                &range
            ).unwrap_or_default();
            let range = (span.start)..(span.end);
            source.replace_range(range, &change.text);
        }
    }
    state.files.update(id, source.clone());
    update_asts(source,state,url.clone());
    id
}


// uses marlowe token rule in combination with s expression to create final verdict 
// on which semantictoken type to use for a specific range
fn get_token_id(mar_vec:Vec<(Range, marlowe_lang::parsing::Rule, SemanticToken)>) -> impl Fn(sex::Rule,Range) -> u32 {
    move |rule:sex::Rule,range:Range| {
        let marlowe_match = 
            mar_vec.iter().find(|x|x.0==range);
        let default_func = |rule| match rule {
            sex::Rule::string => 1,
            sex::Rule::number => 2,
            sex::Rule::ident => 0,
            _ => 99
        };
        if let Some(x) = marlowe_match {
            match x.1 {
                marlowe_lang::parsing::Rule::Case => 3,
                _ => default_func(rule)
            }
        } else { default_func(rule) }
    }
}

fn update_asts(source:String,state:&mut State,url:Url)  {
    
    let marlowe_tokens = marlowe_lang::parsing::Rule::lsp_parse(
        &source, |_rule,_range|{0} // we don't use output from this fn atm
    );

    let mar_vec = 
        match &marlowe_tokens {
            Ok(x) => x.to_vec(),
            Err(_) => vec![],
        };

    let sex_tokens = 
        sex::Rule::lsp_parse(
            &source,  get_token_id(mar_vec)
        );

    match marlowe_tokens {
        Ok(tokens) => {
            //println!("Marlowe parser succeeded");
            state.marlowe_parser_error = None;
            if state.marlowe_asts.contains_key(&url) {
                *state.marlowe_asts.get_mut(&url).unwrap() = tokens;    
            } else {
                state.marlowe_asts.insert(url.clone(),tokens);    
            }
            
        },
        Err((e,r)) => {
            //println!("Marlowe parser failed.. error was: \n{e:#}");
            state.marlowe_parser_error = Some((e,r));
            if state.marlowe_asts.contains_key(&url) {
                *state.marlowe_asts.get_mut(&url).unwrap() = vec![];    
            } else {
                state.marlowe_asts.insert(url.clone(),vec![]);    
            }



        }
    };  

    match sex_tokens {
        Ok(tokens) => {
            //println!("S-expression parser succeeded");
            state.sexpression_parser_error = None;
            if state.sexpression_asts.contains_key(&url) {
                *state.sexpression_asts.get_mut(&url).unwrap() = tokens;    
            } else {
                state.sexpression_asts.insert(url.clone(),tokens);    
            }
            
        },
        Err((e,r)) => {
            //println!("S-expression parser failed.. error was: \n{e:#}");
            state.sexpression_parser_error = Some((e,r));
            if state.sexpression_asts.contains_key(&url) {
                *state.sexpression_asts.get_mut(&url).unwrap() = vec![];    
            } else {
                state.sexpression_asts.insert(url.clone(),vec![]);    
            }
        }
    }; 

}

fn get_diagnostics(state:&mut State) -> Vec<Diagnostic> {
    
    let sex_diags = match &state.sexpression_parser_error {
        Some((msg,range)) => Some(Diagnostic { 
            range: range.clone(), 
            severity: None, 
            code: Some(NumberOrString::String("S-Expression parser error".to_string())),  
            code_description: None, 
            source: None,
            message: msg.to_string(), 
            related_information: None,
            tags: None,
            data: None
        }),
        None => None,
    };

    if let Some(d) = sex_diags {
        return vec![d]
    }
    
    let marlowe_diags = match &state.marlowe_parser_error {
        Some((msg,range)) => {
            //println!("GOT DIAG ERROR FOR MARLOWE WITH RANGE: {range:?}");
            Some(Diagnostic { 
                range: range.clone(), 
                severity: None, 
                code: Some(NumberOrString::String("Marlowe parser error".to_string())), 
                code_description: None, 
                source: None, 
                message: msg.to_string(),
                related_information: None, 
                tags: None, 
                data: None 
            })},
        None => None,
    };
    
    if let Some(d) = marlowe_diags {
        return vec![d]
    }

    vec![]
}


use lsp_types::{SemanticToken, Range};
use pest_derive::Parser;

pub mod sex {
    use super::*;
    #[derive(Parser)]
    #[grammar = "../sex.grammars"]
    pub struct SexParser;
}

// We do multiple passes (sexpress+marlowe) for parsing because it was easier to do
// than switch from pest.rs which does not support token streaming..
trait LSParse<T> {
    fn lsp_parse(sample:&str,f: impl Fn(T,Range) -> u32) -> std::result::Result<Vec<(Range,T,lsp_types::SemanticToken)>,(String,lsp_types::Range)>;
    fn get_token_at_position(tokens:Vec<(Range,T,lsp_types::SemanticToken)>,position:lsp_types::Position) -> Option<(Range,T,SemanticToken)>;
    fn get_token_info_at_position(p:Vec<(Range,T,lsp_types::SemanticToken)>,position:lsp_types::Position, f:fn(T)->String) -> Option<String>;
}

use pest::Parser;
#[macro_export]
#[doc(hidden)]
macro_rules! Impl_LSPARSE_For {
    
    ($rule_type:ty,$parser_type:ty,$top_type:expr) => {
        impl LSParse<$rule_type> for $rule_type {
            
            fn lsp_parse(sample:&str,f: impl Fn($rule_type,Range) -> u32) -> std::result::Result<Vec<(Range,$rule_type,lsp_types::SemanticToken)>,(String,lsp_types::Range)> {
                
                match <$parser_type>::parse(
                    $top_type,
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
                                        token_type: f(x.as_rule(),range), 
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
            
            fn get_token_at_position(tokens:Vec<(Range,$rule_type,lsp_types::SemanticToken)>,position:lsp_types::Position) -> Option<(Range,$rule_type,SemanticToken)> {
                let line = position.line + 1;
                let char = position.character + 1;
                let mut currently_closest : Option<(Range,$rule_type,SemanticToken)> = None;
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
                    None => None,
                    Some((a,b,c)) => {
                        Some((Range {
                            start: Position {
                                character: a.start.character - 1,
                                line: a.start.line - 1
                            },
                            end: Position {
                                character: a.end.character - 1,
                                line: a.end.line - 1
                            }
                        },b,c))
                    }
                }
            }
            
            fn get_token_info_at_position(p:Vec<(Range,$rule_type,lsp_types::SemanticToken)>,position:lsp_types::Position, f:fn($rule_type)->String) -> Option<String> {
                match Self::get_token_at_position(p,position) {
                        Some(ooh) => Some(f(ooh.1)),
                        None => None
                }    
            }
        
        }
    }
}



Impl_LSPARSE_For!(
    sex::Rule,
    sex::SexParser,
    sex::Rule::expressions
);

Impl_LSPARSE_For!(
    marlowe_lang::parsing::Rule,
    marlowe_lang::parsing::MarloweParser,
    marlowe_lang::parsing::Rule::Contract
);


use tokio::io::{stdin, stdout};
//use wasm_bindgen::prelude::*;

#[tokio::main]
//#[wasm_bindgen]
pub async fn main() {

    let (service, socket) = 
        LspService::build(|xx| {
            MyLSPServer { 
                client: xx,
                state: Mutex::new(
                    State {
                        files: codespan::Files::new(),
                        sources: HashMap::new(),
                        sexpression_asts: HashMap::new(),
                        marlowe_asts: HashMap::new(),
                        marlowe_parser_error: None,
                        sexpression_parser_error: None
                    } 
                )
            }
        }).finish();

    
    let stdin = stdin();
    let stdout = stdout();

    let server = Server::new(stdin, stdout, socket);

    server.serve(service).await;
    // loop {
        
    //     let listener = TcpListener::bind(format!("127.0.0.1:8080")).await.unwrap();
    //     println!("Starting lsp service listener ... {:?}",listener);
    //     let (stream, _) = listener.accept().await.unwrap();
    //     let (read, write) = tokio::io::split(stream);
        
    //     let (service, socket) = LspService::new(|client| 
    //         MyLSPServer { client, state: Mutex::new(State {
    //             files: codespan::Files::new(),
    //             sources: HashMap::new(),
    //             sexpression_asts: HashMap::new(),
    //             marlowe_asts: HashMap::new(),
    //             marlowe_parser_error: None,
    //             sexpression_parser_error: None
    //         })
    //     });
    //     println!("Client has connected!");
    //     Server::new(read, write, socket).serve(service).await;
    // }

}

