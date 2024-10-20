mod bind;

use std::error::Error;

use bind::log::IscLog;
use bind::mem::IscMem;
use bind::parser::IscParser;
use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId};
use lsp_types::notification::DidSaveTextDocument;
use lsp_types::notification::Notification as _;
use lsp_types::notification::PublishDiagnostics;
use lsp_types::request::Request as _;
use lsp_types::Diagnostic;
use lsp_types::Position;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::Range;
use lsp_types::{
    request::DocumentDiagnosticRequest, DiagnosticOptions, DiagnosticServerCapabilities,
    InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("starting generic LSP server");
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            ..Default::default()
        })),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = connection.initialize(server_capabilities).unwrap();
    eprintln!("server initialized");

    main_loop(connection, initialization_params)?;

    io_threads.join()?;
    eprintln!("shutting down server");

    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting example main loop");

    let mem = IscMem::new();
    let log = IscLog::new(&mem);

    let parser = IscParser::new(&mem, &log).unwrap();
    for msg in &connection.receiver {
        match msg {
            Message::Notification(req) => match req.method.as_str() {
                DidSaveTextDocument::METHOD => {
                    eprintln!("notification: {req:?}");

                    let params = notification::<DidSaveTextDocument>(req)
                        .expect("failed to parse notification");

                    let path = params.text_document.uri.to_string().replace("file://", "");

                    let msg = match parser.parse_file(path.as_ref()) {
                        Ok(obj) => {
                            if !obj.check(&log, &mem) {
                                eprint!("{}", log);
                            }

                            log.to_string()
                        }
                        Err(_) => log.to_string(),
                    };

                    let mut diagnostics: Vec<Diagnostic> = vec![];
                    msg.split("\n").for_each(|line| {
                        let strs = line.splitn(3, ":").collect::<Vec<&str>>();
                        if strs.len() < 3 {
                            return;
                        }

                        let position: u32 = strs[1].parse().expect("failed to parse position");
                        let message = strs[2].to_string();

                        let diagnostic = lsp_types::Diagnostic {
                            range: Range {
                                start: Position {
                                    line: position - 1,
                                    character: 0,
                                },
                                end: Position {
                                    line: position - 1,
                                    character: u32::max_value(),
                                },
                            },
                            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                            message,
                            ..Default::default()
                        };

                        diagnostics.push(diagnostic);
                    });

                    let notification = Notification::new(
                        PublishDiagnostics::METHOD.to_string(),
                        PublishDiagnosticsParams {
                            uri: params.text_document.uri,
                            diagnostics,
                            version: None,
                        },
                    );

                    match connection.sender.send(Message::Notification(notification)) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("failed to send notification: {}", err);
                        }
                    }
                }
                _ => {
                    eprintln!("received an unknown notification method: {}", req.method);
                }
            },
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                match req.method.as_str() {
                    DocumentDiagnosticRequest::METHOD => {
                        let (_, params) = request::<DocumentDiagnosticRequest>(req)
                            .expect("failed to parse request");
                        eprintln!("got DocumentDiagnostic request: {params:?}");
                    }
                    _ => {
                        eprintln!("received an unknown request method: {}", req.method);
                    }
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
        }
    }

    Ok(())
}

fn request<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn notification<R>(req: Notification) -> Result<R::Params, ExtractError<Notification>>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
