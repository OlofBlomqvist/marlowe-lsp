#![feature(start)]

mod codespan_lsp_local;
use codespan::FileId;
use codespan_lsp_local::{range_to_byte_span};
use regex::{Regex};
use std::{collections::HashMap, sync::Mutex, hash::Hash};
use serde_json::Value;
use tower_lsp::{ jsonrpc::{Result}, Client, LanguageServer, LspService, Server };
use tower_lsp::lsp_types::*;
use line_col::LineColLookup;

use lsp_types::{SemanticToken, Range};
use pest_derive::Parser;

pub mod sex {
    use super::*;
    #[derive(Parser)]
    #[grammar = "../sex.grammars"]
    pub struct SexParser;
}


#[derive(Debug)]
struct MyLSPServer {
    client: Client,
    state: Mutex<State>
}

#[derive(Debug)]
struct State {
    sources: HashMap<Url, FileId>,
    sexpression_asts: HashMap<Url, (Vec<(Range,sex::Rule,SemanticToken)>,ContractValidationResult)>,
    marlowe_asts:     HashMap<Url, (Vec<(Range,marlowe_lang::parsing::Rule,SemanticToken)>,ContractValidationResult)>,
    files: codespan::Files<String>,
    marlowe_parser_error: Option<(String,Range)>,
    sexpression_parser_error: Option<(String,Range)>
    
}

// TODO:

// Add support for get_diagnostics function to return 
// suggestions for auto_resolving things like when a value
// can be simplified from MulVal(2,2) into just Constant 4..

// Add support for ?party/?payee?/?contract/ etc.. holes
// such that we can autosuggest items to use,
// so that we can handle copy-pasted contracts from playground.

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
                    trigger_characters: Some(vec![String::from("\"")]),
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
                    token_list.0.to_vec(),
                    params.text_document_position_params.position,
                    |r| match r {
                        marlowe_lang::parsing::Rule::Notify |
                        marlowe_lang::parsing::Rule::Choice |
                        marlowe_lang::parsing::Rule::Deposit => String::from("Contracts in Marlowe run on a blockchain, but need to interact with the off-chain world. The parties to the contract, whom we also call the participants, can engage in various actions: they can be asked to deposit money, or to make a choice between various alternatives. A notification of an external value (also called an oracle value), such as the current price of a particular commodity, is the other possible form of input."),
                        marlowe_lang::parsing::Rule::Case => String::from("A When contract contains a collection of cases. Each case has the form Case action next where action is an Action and next a continuation (another contract). When a particular action happens, the state is updated accordingly and the contract will continue as the corresponding continuation next."),
                        marlowe_lang::parsing::Rule::Bound => String::from("A choice is made for a particular id with a list of bounds on the values that are acceptable. For example, [Bound 0 0, Bound 3 5] offers the choice of one of 0, 3, 4 and 5."),
                        marlowe_lang::parsing::Rule::Party |
                        marlowe_lang::parsing::Rule::PK |
                        marlowe_lang::parsing::Rule::Role => String::from("A Party is represented as either a public key hash or a role name. In order to progress a Marlowe contract, a party must provide an evidence. For PK party that would be a valid signature of a transaction signed by a private key of a public key that hashes to party’s PubKeyHash, similarly to Bitcoin’s Pay to Public Key Hash mechanism. For a Role party the evidence is spending a role token within the same transaction, usually to the same owner. So, Role parties will look like (Role \"alice\"), (Role \"bob\") and so on."),
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
                        marlowe_lang::parsing::Rule::TimeConstant => String::from("Timeout is the slot number after which the When will no longer accept any new events: Case branches will become unusable, and the contract will continue as specified by the timeout continuation. Timeouts accept templates, this means that instead of writing a specific slot number it is possible to fill Timeouts by using a template parameter that can be filled just before deploying or simulating the contract, for example: TimeParam \"maturityDate\""),
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
                    data: token_list.0.iter().map(|x|x.2).collect()
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
        

        let toks = {
            let mut state = self.state.lock().unwrap();
            match state.marlowe_asts.get_mut(&params.text_document_position_params.text_document.uri) {
                None => vec![],
                Some(semantic_tokens) => semantic_tokens.0.clone()
            }
            
        };
       
        let closest = 
            marlowe_lang::parsing::Rule::get_token_at_position(
                toks.to_vec(),params.text_document_position_params.position
            );
        
        match closest {
            Some((a,rule,_c)) => {
                {
                    self.client.log_message(MessageType::INFO, format!("highlighting selected '{rule:?}'") ).await;        
                }
                Ok(Some(vec![
                    DocumentHighlight { 
                        range: a,
                        kind: Some(DocumentHighlightKind::TEXT)
                    }])
                )
            }
            None => Ok(None)
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
            get_diagnostics(&mut state,&params.text_document.uri)
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
            get_diagnostics(&mut state,&params.text_document.uri)
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

    async fn completion(&self, completion_params: CompletionParams) -> Result<Option<CompletionResponse>> {
        
        // If we ever want to do anything more than basic Role name suggestions in here,
        // this code should be thrown out and replaced completely.

        let (source,col) = {
            let state = self.state.lock().unwrap();
            let id = *state.sources.get(&completion_params.text_document_position.text_document.uri).unwrap();
            let bindex = codespan_lsp_local::position_to_byte_index(
                &state.files, id, &completion_params.text_document_position.position).unwrap();
            let src = state.files.source(id).to_owned();
            (src,bindex)
        };

        if col < 10 { return Ok(None) }

        match source.get(col - 6 .. col) {
            None => return Ok(None),
            Some(prior) => 
                if prior != "Role \"" {
                    return Ok(None)
                }
        }

        let mut matches : Vec<CompletionItem> = 
            Regex::new("Role \".*\"")
                .unwrap()
                .find_iter(&source).map(|x|{
                    let s = x.as_str();
                    CompletionItem {
                        label: if let Some(xx) = s.get(6..s.len()-1) { xx.to_string() } else { s.to_string() }, 
                        kind: None, detail: None, 
                        documentation: None, deprecated: None, preselect: None, sort_text: None, 
                        filter_text: None, insert_text: None, insert_text_format: None, 
                        insert_text_mode: None, text_edit: None, additional_text_edits: None, 
                        command: None, commit_characters: None, data: None, tags: None }
                }
                ).collect();
        
        if matches.is_empty() {return Ok(None)}
        matches.sort_by_key(|x|x.label.clone());
        matches.dedup_by_key(|x|x.label.clone());
        Ok(Some(
            lsp_types::CompletionResponse::List(
                CompletionList { 
                    is_incomplete: false, 
                    items: matches
                }
            )
        ))
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
        source.clone(), |_rule,_range|{0} // we don't use output from this fn atm
    );

    let mar_vec = 
        match &marlowe_tokens {
            Ok(x) => x.0.to_vec(),
            Err(_) => vec![],
        };
        
    let sex_tokens = 
        sex::Rule::lsp_parse(
            source.clone(),  get_token_id(mar_vec)
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
                *state.marlowe_asts.get_mut(&url).unwrap() = (vec![],ContractValidationResult{items:vec![]});    
            } else {
                state.marlowe_asts.insert(url.clone(),(vec![],ContractValidationResult{items:vec![]}));    
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
                *state.sexpression_asts.get_mut(&url).unwrap() = (vec![],ContractValidationResult{items:vec![]}); 
            } else {
                state.sexpression_asts.insert(url.clone(),(vec![],ContractValidationResult{items:vec![]}));    
            }
        }
    }; 

}

fn get_diagnostics(state:&mut State,url:&Url) -> Vec<Diagnostic> {
    
    match &state.sexpression_parser_error {
        None => {},
        Some((msg,range)) => {
            return vec![
                Diagnostic { 
                    range: range.clone(), 
                    severity: None, 
                    code: Some(NumberOrString::String("S-Expression parser error".to_string())),  
                    code_description: None, 
                    source: None,
                    message: msg.to_string(), 
                    related_information: None,
                    tags: None,
                    data: None
                }    
            ];
        }
    };

    
    match &state.marlowe_parser_error {
        None => {},
        Some((msg,range)) => 
            return vec![Diagnostic { 
                range: range.clone(), 
                severity: None, 
                code: Some(NumberOrString::String("Marlowe parser error".to_string())), 
                code_description: None, 
                source: None, 
                message: msg.to_string(),
                related_information: None, 
                tags: None, 
                data: None 
            }]
    };
    
    match state.marlowe_asts.get(url) {
        Some(x) => {
            x.1.items.iter().map(|d|               
                Diagnostic { 
                    range: d.0, 
                    severity: Some(d.3), 
                    code: Some(NumberOrString::String("DIAGNOSTIC".to_string())), 
                    code_description: None, 
                    source: None, 
                    message: d.2.to_owned(),
                    related_information: None, 
                    tags: None, 
                    data: None 
                }   
            ).collect()
        }
        None => vec![]
    }
   
    
    

}





fn get_range(x:pest::iterators::Pair<marlowe_lang::parsing::Rule>) -> Range {
    let span = x.as_span();
    let start_pos = span.start_pos().line_col();
    let end_pos = span.end_pos().line_col();
    Range {
        start : Position { line: start_pos.0 as u32 -1 , character: start_pos.1 as u32 - 1  },
        end : Position { line: end_pos.0 as u32 -1 , character: end_pos.1 as u32 - 1},
    }
}

#[derive(Debug)]
struct ContractValidationResult {
    items : Vec<(Range,String,String,DiagnosticSeverity)>
}

#[derive(Clone,Default,Debug)]
struct TokenType {
    currency_symbol : String ,
    token_name : String
}

#[derive(Clone,Default,Debug)]
struct AccountInfo {
    token_amounts : HashMap<TokenType,i64>
}

#[derive(Clone)]
enum VariableAssignment {
    Constant(i64),
    VariablePointer(String)
}

#[derive(Clone)]
struct NodeContext {
    defined_roles : Vec<String>,
    highest_timeout : Option<i64>,
    known_accounts : HashMap<String,AccountInfo>,
    let_assigns : HashMap<String,VariableAssignment>
}

fn parse_a_value() {
    // todo take a marlowe value and recursively process it.
    // return option type (since we cannot know the value should it be based on runtime variabled).
    // return also option string with warning should the thing be possibly to simplify.
}

fn parse_a_let_assignment() {
    // todo take a marlowe value and recursively process it.
    // return option type (since we cannot know the value should it be based on runtime variabled).
    // return also option string with warning should the thing be possibly to simplify.
}

// TODO:

// - - 'The contract makes a payment from account "xxxx" before a deposit has been made. (PayBeforeDeposit)'
// - - - ^ This includes the case where deposits have actually been made but another token was used.
// - - - ^ So for this check we must have a context that separates the deposit values by token type.

// - - 'The contract makes a payment of ₳ 0.000002 from account "xxx" but the account only has ₳ 0.000001. (PartialPayment)'
// - - 'The contract makes a payment of 21  from account "xxx" but the account only has 1. (PartialPayment)'
// - - 'The contract makes a payment of 21 ABC from account "xxx" but the account only has 1 ABC. (PartialPayment)'
// - - - ^ The PartialPayment validations must also take the token type into account.

// - - 'The value "(MulValue (Constant 10) (Constant 10))" can be simplified to "(Constant 100)" (SimplifiableValue).
// - - - ^ This just means that a value consist entierly out of static values.

// - - 'The contract uses a ValueId that has not been defined in a Let, so (Constant 0) will be used. (UndefinedUse).
// - - - ^ Add LET assignments to context and validate in UseVal values!

// - - Find continuations of type close where we end up implicitly refunding assets

// - - Find bool statements that will always evalutate to true/false causing sub-contracts
//     to be unreachable.

#[decurse::decurse]
fn recursively_validate_contract(mut pairs:pest::iterators::Pairs<'static,marlowe_lang::parsing::Rule>,context:NodeContext) -> ContractValidationResult {

    let mut result = ContractValidationResult {
        items: vec![]
    };

    while let Some(p) = pairs.next() {
        match p.as_rule() {
            
            marlowe_lang::parsing::Rule::ChoiceId => { 
                let s = p.into_inner().next().unwrap().into_inner().next().unwrap();
                let ss = s.as_str().to_string();
                if !context.defined_roles.contains(&ss) {
                    result.items.push((get_range(s),ss.clone(),format!("The choice '{}' does not seem to be valid here.. The choices that have been previously defined in this context are: {:?}. \nThe contract uses a ChoiceId that has not been input by a When, so (Constant 0) will be used.",ss,context.defined_roles),DiagnosticSeverity::WARNING));
                }
            }

            marlowe_lang::parsing::Rule::Case => {
                
                let mut case = p.into_inner();
                let mut this_case_context = context.clone();  
                while let Some(x) = case.next() {
                    let mut action = x.into_inner();
                    while let Some(inner_action) = action.next() {
                        match inner_action.as_rule() {
                            // todo: update the context account information
                            marlowe_lang::parsing::Rule::Deposit => {
                                let mut pairs = inner_action.into_inner();
                                let by = pairs.next().unwrap();
                                let into_account_of = pairs.next().unwrap();
                                let currency = pairs.next().unwrap();
                                let amount = pairs.next().unwrap();
                                // todo: handle adding to context
                                // if the amount is a parameter, we cannot know anything now
                                // let mut hmm = this_case_context.known_accounts.entry("BANKEN".to_owned()).or_default();
                                // let v = hmm.token_amounts.entry("ADA".to_string()).or_default();
                                // *v += 100;

                                for x in recursively_validate_contract(by.into_inner(), this_case_context.clone()).items {
                                    result.items.push(x)
                                } 
                                for x in recursively_validate_contract(into_account_of.into_inner(), this_case_context.clone()).items {
                                    result.items.push(x)
                                } 

                            }
                            marlowe_lang::parsing::Rule::Choice => {
                                let mut choice = inner_action.into_inner();
                                while let Some(x) = choice.next() {
                                    match x.as_rule() {
                                        marlowe_lang::parsing::Rule::ChoiceId => {
                                            let s = x.into_inner().next().unwrap().into_inner().next().unwrap();
                                            let ss = s.as_str().to_string();
                                            if ! context.defined_roles.contains(&ss) {
                                                this_case_context.defined_roles.push(ss);
                                            }
                                        },
                                        marlowe_lang::parsing::Rule::ArrayOfBounds => {}
                                        _ => unreachable!()
                                    }
                                }
                            }
                            _ => {
                                for x in recursively_validate_contract(inner_action.into_inner(), this_case_context.clone()).items {
                                    result.items.push(x)
                                } 
                            }
                        }
                    }
                }
            }
            // todo: validate against the context account info
            marlowe_lang::parsing::Rule::Pay => {
                let mut pay_contract = p.into_inner();
                let party = pay_contract.next().unwrap().into_inner();
                let payee = pay_contract.next().unwrap().into_inner();
                let _currency = pay_contract.next().unwrap().into_inner().next().unwrap();
                let _amount = pay_contract.next().unwrap().into_inner().next().unwrap();
                let continuation_contract = pay_contract.next().unwrap().into_inner();
                let sub_context = context.clone();

                //let acc = sub_context.known_accounts.get("BANKEN").unwrap();
                
                // result.items.push((
                //     get_range(amount.clone()),
                //     String::new(),
                //     format!("This is: (currency: {} , amount: {}). CONTEXT NOW IS: {:?}",currency.as_str(),amount.as_str(), wooo)
                //     ,DiagnosticSeverity::INFORMATION
                // ));

                // validate party against outer context
                for x in recursively_validate_contract(party, context.clone()).items {
                    result.items.push(x)
                }
                
                // validate payee against outer context
                for x in recursively_validate_contract(payee, context.clone()).items {
                    result.items.push(x)
                }

                // validate sub-contract using the updated inner context
                for x in recursively_validate_contract(continuation_contract, sub_context.clone()).items {
                    result.items.push(x)
                }
            }
            marlowe_lang::parsing::Rule::When => {

                let mut when_contract = p.into_inner();
                let cases = when_contract.next().unwrap().into_inner();
                let time_out = when_contract.next().unwrap().into_inner().next().unwrap();
                let continuation_contract = when_contract.next().unwrap().into_inner();
                let mut sub_context = context.clone();
                match time_out.as_rule() {
                    marlowe_lang::parsing::Rule::TimeConstant => {
                        match sub_context.highest_timeout {
                            Some(highest_seen_so_far) => {
                                let inner_value = time_out.as_str().parse::<i64>().unwrap();
                                if inner_value > highest_seen_so_far {
                                    sub_context.highest_timeout = Some(inner_value);
                                    //result.push((get_range(time_out),String::new(),String::from("VERY GOOD"),DiagnosticSeverity::INFORMATION))
                                } else {
                                    result.items.push((get_range(time_out),String::new(),format!("Expected a timeout greater than {}",highest_seen_so_far),DiagnosticSeverity::WARNING))
                                }
                            },
                            None => sub_context.highest_timeout = Some(
                                time_out.as_str().parse::<i64>().unwrap()
                            ),
                        }
                    },
                    _ => {}
                }
                for x in recursively_validate_contract(cases, sub_context.clone()).items {
                    result.items.push(x)
                }                
                
                for x in recursively_validate_contract(continuation_contract, sub_context.clone()).items {
                    result.items.push(x)
                }
            },
            _ => {
                for x in recursively_validate_contract(p.into_inner(), context.clone()).items {
                    result.items.push(x)
                }
            }
        }
    }
    result
}


// We do multiple passes (sexpress+marlowe) for parsing because it was easier to do
// than switch from pest.rs which does not support token streaming..
trait LSParse<T> {
    fn lsp_parse(sample:String, f: impl Fn(T,Range) -> u32) ->
        std::result::Result<
            (Vec<(Range,T,lsp_types::SemanticToken)>,ContractValidationResult),
            (String,lsp_types::Range)>;
    fn get_token_at_position(tokens:Vec<(Range,T,lsp_types::SemanticToken)>,position:lsp_types::Position) -> Option<(Range,T,SemanticToken)>;
    fn get_token_info_at_position(p:Vec<(Range,T,lsp_types::SemanticToken)>,position:lsp_types::Position, f:fn(T)->String) -> Option<String>;
}

use pest::{Parser};
#[macro_export]
#[doc(hidden)]
macro_rules! Impl_LSPARSE_For {
    
    ($rule_type:ty,$parser_type:ty,$top_type:expr,$test:expr) => {
        
        impl LSParse<$rule_type> for $rule_type {
            
            fn lsp_parse(sample:String,f: impl Fn($rule_type,Range) -> u32) -> 
                std::result::Result<
                    (Vec<(Range,$rule_type,lsp_types::SemanticToken)>,ContractValidationResult), (String,lsp_types::Range)
                > {
                let boxed = Box::new(sample.clone());
                let lookup = LineColLookup::new(&sample);
                match <$parser_type>::parse(
                    $top_type,
                    Box::leak(boxed)
                ) {
                    Ok(p) => { 
                        
                        let mut previous_range : Option<lsp_types::Range> = None;
                        let mut last_line_start : usize = 1;
                        let mut last_line_end: usize = 1;
                        let mut last_start: usize = 1;
                        let mut last_end: usize = 1;
                        
                        let data = 
                            p.clone().flatten().map(|x|{
                                let span = x.as_span();
                                let start_pos = span.start();
                                let end_pos = span.end();
                                
                                let (start_line,start_col) = lookup.get(start_pos);
                                let (end_line,end_col) = lookup.get(end_pos);
                                
                                let range = lsp_types::Range {
                                    start: lsp_types::Position::new(start_line as u32,start_col as u32),
                                    end:   lsp_types::Position::new(end_line as u32,end_col as u32),
                                };
                                let mut corrected_start = start_col as usize;
                                if start_line == last_line_start {
                                    corrected_start = corrected_start - last_start;
                                    
                                } else {
                                    corrected_start = corrected_start - 1;
                                }       
                                let corrected_line = (start_line - last_line_start);
                                let calculated_length = span.as_str().len();

                                let token = SemanticToken { 
                                    // `deltaLine`: token line number, relative to the previous token
                                    // `deltaStart`: token start character, relative to the previous token 
                                    //  (relative to 0 or the previous token's start if they are on the same line)
                                    // `length`: the length of the token. A token cannot be multiline.
                                    // `tokenType`: will be looked up in `SemanticTokensLegend.tokenTypes`
                                    // `tokenModifiers`: each set bit will be looked up in `SemanticTokensLegend.tokenModifiers`
                                    delta_line: corrected_line as u32,
                                    delta_start: corrected_start as u32 ,
                                    length: calculated_length as u32,
                                    token_type: f(x.as_rule(),range), 
                                    token_modifiers_bitset: 0 
                                };
        
                                (last_line_end,last_end) = (end_line,end_col);
                                (last_line_start,last_start) = (start_line,start_col);
                                previous_range = Some(range);
                                (range,x.as_rule(),token)
                            }).collect();

                        let validation_result = $test(p);
                        Ok((data,validation_result))
                       
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
    sex::Rule::expressions,
    |_| ContractValidationResult { items: vec![] }
);

Impl_LSPARSE_For!(
    marlowe_lang::parsing::Rule,
    marlowe_lang::parsing::MarloweParser,
    marlowe_lang::parsing::Rule::Contract,
    |x| {
        recursively_validate_contract(x, NodeContext { 
            defined_roles: vec![], 
            highest_timeout: None , 
            known_accounts: HashMap::new(),
            let_assigns : HashMap::new()
        })
    }
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

