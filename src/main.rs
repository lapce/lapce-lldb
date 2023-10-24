use anyhow::Result;
use lapce_plugin::{
    psp_types::{
        lsp_types::{request::Initialize, InitializeParams, MessageType},
        Request,
    },
    register_plugin, LapcePlugin, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    PLUGIN_RPC.stderr(&format!("plugin returned with error: {e}"))
                }
            }
            _ => {}
        }
    }
}

fn initialize(params: InitializeParams) -> Result<()> {
    let server_path = params
        .initialization_options
        .as_ref()
        .and_then(|options| options.get("serverPath"))
        .and_then(|server_path| server_path.as_str())
        .and_then(|server_path| {
            if !server_path.is_empty() {
                Some(server_path)
            } else {
                None
            }
        });

    let program = match std::env::var("VOLT_OS").as_deref() {
        Ok("windows") => "where",
        _ => "which",
    };
    if let Some(server_path) = server_path {
        let exits = PLUGIN_RPC
            .execute_process(program.to_string(), vec![server_path.to_string()])
            .map(|r| r.success)
            .unwrap_or(false);
        if !exits {
            PLUGIN_RPC.window_show_message(
                MessageType::ERROR,
                format!("lldb path {server_path} couldn't be found, please check"),
            );
            return Ok(());
        }
        PLUGIN_RPC.register_debugger_type("lldb".to_string(), server_path.to_string(), None)?;
        return Ok(());
    }

    let exits = PLUGIN_RPC
        .execute_process(program.to_string(), vec!["lldb-vscode".to_string()])
        .map(|r| r.success)
        .unwrap_or(false);
    if !exits {
        PLUGIN_RPC.window_show_message(
            MessageType::ERROR,
            "lldb-vscode couldn't be found, please install or configure the path".to_string(),
        );
        return Ok(());
    }
    PLUGIN_RPC.register_debugger_type("lldb".to_string(), "lldb-vscode".to_string(), None)?;

    Ok(())
}
