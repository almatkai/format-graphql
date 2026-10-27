//! Production adapter. Compiled by the host as a sibling of v2_sdk.
//! Library crate dependency: tezbar-format-graphql (the vendored v2 directory).
use super::v2_sdk as sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};
use tezbar_format_graphql::{format, Indentation, MAX_INPUT};

const SLUG: &str = "format-graphql";
static NONCE: OnceLock<String> = OnceLock::new();
static TOKEN: AtomicU64 = AtomicU64::new(1);
fn token() -> String {
    let nonce = NONCE.get_or_init(|| uuid::Uuid::new_v4().to_string());
    let counter = TOKEN
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .expect("native action token space exhausted");
    format!("format-graphql:{nonce}:{counter}")
}
#[derive(Clone, Serialize, Deserialize)]
struct Actions {
    copy: String,
    only: String,
    indent: String,
    retry: String,
}
impl Actions {
    fn new() -> Self {
        Self {
            copy: token(),
            only: token(),
            indent: token(),
            retry: token(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct CopyIntent {
    output: String,
    phase: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Replacement {
    id: String,
    base: String,
    output: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Session {
    actions: Actions,
    indent: String,
    message: String,
    intent: Option<CopyIntent>,
    replacement: Option<Replacement>,
}
fn save(state: &mut Value, session: &Session) {
    *state = serde_json::to_value(session).expect("serializable native session");
}
fn load(state: &Value) -> Result<Session, String> {
    serde_json::from_value(state.clone()).map_err(|_| "Reopen Format GraphQL.".into())
}
fn stored_indent(store: &Value) -> Result<Option<String>, String> {
    if !store.is_object() {
        return Err("Native indentation storage is invalid.".into());
    }
    match store.get("indent") {
        None => Ok(None),
        Some(Value::String(value)) if Indentation::from_value(value).is_ok() => {
            Ok(Some(value.clone()))
        }
        _ => Err("Native indentation storage is invalid. Repair it before saving.".into()),
    }
}
async fn persist(ctx: &CoreContext, value: &str) -> Result<(), String> {
    let value = value.to_owned();
    sdk::update_store(ctx, SLUG, move |store| {
        stored_indent(store)?;
        store["indent"] = json!(value);
        Ok(())
    })
    .await
    .map(|_| ())
    .map_err(|_| "Could not save indentation. Check native storage.".into())
}
fn acknowledged(value: &Value) -> bool {
    value.is_object()
        && value.get("ok") == Some(&Value::Bool(true))
        && value.get("error").is_none_or(Value::is_null)
        && value.get("success").is_none_or(|v| v == &Value::Bool(true))
}
fn view(session: &Session, replacement: Option<&Replacement>, initial: bool) -> Value {
    let mut input = json!({"type":"Form.TextArea","props":{"id":"input","title":"Input","placeholder":"Paste GraphQL here…"}});
    if initial {
        input["props"]["value"] = json!("");
    }
    if let Some(replacement) = replacement {
        input["props"]["value"] = json!(replacement.output);
        input["props"]["replacementBase"] = json!(replacement.base);
        input["props"]["replacementId"] = json!(replacement.id);
    }
    let mut dropdown = json!({"type":"Form.Dropdown","props":{"id":"indent","title":"Indentation","actionId":session.actions.indent},"children":[
        {"type":"Form.Dropdown.Item","props":{"title":"Tabs","value":"tab"}},
        {"type":"Form.Dropdown.Item","props":{"title":"Spaces: 2","value":"2"}},
        {"type":"Form.Dropdown.Item","props":{"title":"Spaces: 4","value":"4"}},
        {"type":"Form.Dropdown.Item","props":{"title":"Spaces: 8","value":"8"}}]});
    if initial {
        dropdown["props"]["value"] = json!(session.indent);
    }
    let mut actions = vec![
        json!({"id":session.actions.copy,"title":"Format and Copy","kind":"submit-form"}),
        json!({"id":session.actions.only,"title":"Only Format","kind":"submit-form"}),
    ];
    if session
        .intent
        .as_ref()
        .is_some_and(|i| i.phase == "uncertain")
    {
        actions.push(json!({"id":session.actions.retry,"title":"Retry Copy (May Copy Again)","kind":"action"}));
    }
    sdk::view(
        "Format GraphQL",
        json!({"type":"Form","children":[input,dropdown,{"type":"Form.Description","props":{"text":session.message}}]}),
        actions,
    )
}
async fn copy(ctx: &CoreContext, state: &mut Value, session: &mut Session) -> Value {
    // Save uncertain intent before awaiting the effect. Cancellation can drop this
    // future after the OS accepted a copy but before its reply reached us.
    let intent = session.intent.as_mut().expect("validated copy intent");
    intent.phase = "uncertain".into();
    let output = intent.output.clone();
    session.actions = Actions::new();
    session.message =
        "Copy was requested. If no acknowledgement arrives, retry deliberately.".into();
    save(state, session);
    let reply = sdk::copy(ctx, &output).await;
    if reply.as_ref().is_ok_and(acknowledged) {
        session.intent.as_mut().unwrap().phase = "acknowledged".into();
        session.message = "Formatted GraphQL. Copied to clipboard.".into();
    } else {
        session.message="GraphQL is formatted. Copy was not acknowledged; it may have completed. Use Retry Copy deliberately.".into();
    }
    save(state, session);
    view(session, None, false)
}
pub fn commands() -> &'static [&'static str] {
    &["index"]
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if command != "index" {
        return Err("Unknown Format GraphQL command.".into());
    }
    if operation == "extension:dispose-session" {
        *state = json!({});
        return Ok(sdk::unchanged());
    }
    if operation == "extension:run-command" {
        // There are no command arguments or preferences. NativeSessions supplies
        // them as argumentValues/preferences, never invented alternative keys.
        let stored = sdk::read_store(ctx, SLUG).await;
        let (indent, message) = match stored.and_then(|v| stored_indent(&v)) {
            Ok(value) => (
                value.unwrap_or_else(|| "tab".into()),
                "Paste a GraphQL document, then choose an action.".into(),
            ),
            Err(_) => (
                "tab".into(),
                "Could not read native indentation storage. Repair it before saving.".into(),
            ),
        };
        let session = Session {
            actions: Actions::new(),
            indent,
            message,
            intent: None,
            replacement: None,
        };
        save(state, &session);
        return Ok(view(&session, None, true));
    }
    let mut session = load(state)?;
    match operation {
        // These calls may receive cloned/discarded state. Return only committed
        // action/replacement IDs. The renderer consumes each replacement once.
        "extension:refresh-session" => {
            return Ok(view(&session, session.replacement.as_ref(), false))
        }
        "extension:search-text-changed" | "extension:load-more" => return Ok(sdk::unchanged()),
        "extension:invoke-action" => {}
        _ => return Err("Unsupported Format GraphQL operation.".into()),
    }
    let action = payload["actionId"].as_str().ok_or("Missing action ID.")?;
    if action == session.actions.retry {
        if session
            .intent
            .as_ref()
            .is_none_or(|i| i.phase != "uncertain")
        {
            return Err("No pending copy to retry.".into());
        }
        return Ok(copy(ctx, state, &mut session).await);
    }
    if action == session.actions.indent {
        let value = payload["formValues"]["value"]
            .as_str()
            .ok_or("Invalid indentation.")?;
        Indentation::from_value(value).map_err(|_| "Invalid indentation.")?;
        session.indent = value.into();
        save(state, &session);
        session.message = match persist(ctx, value).await {
            Ok(()) => "Indentation saved.".into(),
            Err(e) => e,
        };
        save(state, &session);
        return Ok(view(&session, None, false));
    }
    let do_copy = if action == session.actions.copy {
        true
    } else if action == session.actions.only {
        false
    } else {
        return Err("This action expired. Use the current form actions.".into());
    };
    let values = payload["formValues"]
        .as_object()
        .ok_or("Submit the complete form.")?;
    let input = values
        .get("input")
        .and_then(Value::as_str)
        .ok_or("Submit the complete form.")?;
    let indent = values
        .get("indent")
        .and_then(Value::as_str)
        .ok_or("Submit the complete form.")?;
    let indentation = Indentation::from_value(indent).map_err(|_| "Invalid indentation.")?;
    if input.len() > MAX_INPUT {
        return Err("GraphQL input limit exceeded.".into());
    }
    session.actions = Actions::new();
    session.intent = None;
    session.replacement = None;
    if input.is_empty() {
        session.message = "The item is required".into();
        save(state, &session);
        return Ok(view(&session, None, false));
    }
    let output = match format(input, indentation) {
        Ok(output) => output,
        Err(e) => {
            session.message = if e.0 == "Invalid GraphQL" {
                "Invalid GraphQL".into()
            } else {
                e.to_string()
            };
            save(state, &session);
            return Ok(view(&session, None, false));
        }
    };
    session.indent = indent.into();
    // Mint once per successful explicit format; refresh/copy/cancel/retry keep it.
    session.replacement = Some(Replacement {
        id: token(),
        base: input.into(),
        output: output.clone(),
    });
    // Persistence occurs before copy. Refuse to overwrite typed or JSON storage
    // corruption even when it appeared after the session was launched.
    save(state, &session);
    let storage_error = persist(ctx, indent).await.err();
    session.message = storage_error
        .clone()
        .unwrap_or_else(|| "Formatted GraphQL".into());
    if do_copy {
        session.intent = Some(CopyIntent {
            output: output.clone(),
            phase: "ready".into(),
        });
        // Copy feedback does not return another controlled input; the single
        // replacement is consumed once and guarded by submitted base bytes.
        let _ = copy(ctx, state, &mut session).await;
        if let Some(error) = storage_error {
            session.message.push(' ');
            session.message.push_str(&error);
            save(state, &session);
        }
    } else {
        save(state, &session);
    }
    Ok(view(&session, session.replacement.as_ref(), false))
}
