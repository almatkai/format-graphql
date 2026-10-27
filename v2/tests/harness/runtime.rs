use format_graphql_host_harness::native_core::{
    common::{CoreContext, CorePaths},
    extensions::{format_graphql, Harness},
};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
struct Fixture {
    _dir: tempfile::TempDir,
    ctx: CoreContext,
    h: Harness,
    copies: Arc<Mutex<Vec<Vec<Value>>>>,
}
impl Fixture {
    fn new(reply: Result<Value, String>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        let ctx = CoreContext::new(
            CorePaths {
                data_dir: p.join("data"),
                config_dir: p.join("config"),
                resource_dir: p.join("resource"),
                temp_dir: p.join("tmp"),
                home_dir: p.into(),
            },
            None,
        );
        let copies = Arc::new(Mutex::new(vec![]));
        let record = copies.clone();
        ctx.set_router(Arc::new(move |op, args| {
            assert_eq!(op, "clipboard:write");
            record.lock().unwrap().push(args);
            let reply = reply.clone();
            Box::pin(async move { reply })
        }))
        .unwrap();
        Self {
            h: Harness::new(ctx.clone()),
            ctx,
            copies,
            _dir: dir,
        }
    }
    async fn launch(&self) -> Value {
        self.h
            .call(
                "extension:run-command",
                json!({"extensionId":"com.tezbar.native.format-graphql","commandName":"index"}),
            )
            .await
    }
    async fn submit(&self, form: &Value, index: usize, input: &str, indent: &str) -> Value {
        self.h.call("extension:invoke-action",json!({"sessionId":form["sessionId"],"actionId":form["actions"][index]["id"],"formValues":{"input":input,"indent":indent}})).await
    }
    fn store(&self) -> std::path::PathBuf {
        self.ctx
            .paths
            .data_dir
            .join("native-extensions-v2/format-graphql/storage.json")
    }
}
fn message(v: &Value) -> &str {
    v["root"]["children"][2]["props"]["text"].as_str().unwrap()
}
#[tokio::test]
async fn real_sessions_actions_drafts_and_stale_ids() {
    let f = Fixture::new(Ok(json!({"ok":true})));
    let form = f.launch().await;
    assert_eq!(form["root"]["children"][1]["props"]["value"], "tab");
    assert_eq!(form["actions"][0]["title"], "Format and Copy");
    assert_eq!(form["actions"][1]["kind"], "submit-form");
    let only = f.submit(&form, 1, "{a(b:1)}", "4").await;
    assert!(f.copies.lock().unwrap().is_empty());
    assert_eq!(
        only["root"]["children"][0]["props"]["value"],
        "{\n    a(b: 1)\n}\n"
    );
    assert_eq!(
        only["root"]["children"][0]["props"]["replacementBase"],
        "{a(b:1)}"
    );
    assert_eq!(f.submit(&form, 0, "{forged}", "2").await["ok"], false);
    let refresh =
        f.h.call(
            "extension:refresh-session",
            json!({"sessionId":form["sessionId"]}),
        )
        .await;
    assert_eq!(refresh["actions"], only["actions"]);
    let invalid = f.submit(&only, 0, "{secret broken", "2").await;
    assert_eq!(message(&invalid), "Invalid GraphQL");
    assert!(invalid["root"]["children"][0]["props"]
        .get("value")
        .is_none());
    assert!(f.copies.lock().unwrap().is_empty());
    let empty = f.submit(&invalid, 0, "", "2").await;
    assert_eq!(message(&empty), "The item is required");
    let copied = f.submit(&empty, 0, "{a}", "2").await;
    assert!(message(&copied).contains("Copied"));
    assert_eq!(*f.copies.lock().unwrap(), vec![vec![json!("{\n  a\n}\n")]]);
    let other = f.launch().await;
    assert_eq!(f.h.call("extension:invoke-action",json!({"sessionId":other["sessionId"],"actionId":copied["actions"][0]["id"],"formValues":{"input":"{a}","indent":"2"}})).await["ok"],false);
    assert_eq!(
        f.h.call(
            "extension:dispose-session",
            json!({"sessionId":form["sessionId"]})
        )
        .await,
        true
    );
    assert_eq!(f.submit(&copied, 0, "{a}", "2").await["ok"], false);
    f.h.clear_slug().await;
}
#[tokio::test]
async fn strict_acknowledgements_private_errors_and_explicit_retry() {
    for reply in [
        Ok(Value::Null),
        Ok(json!(true)),
        Ok(json!(false)),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"error":"private secret"})),
        Ok(json!({"ok":true,"error":false})),
        Ok(json!({"ok":true,"success":null})),
        Ok(json!({"success":true})),
        Err("private token /home/person".into()),
    ] {
        let f = Fixture::new(reply);
        let form = f.launch().await;
        let result = f.submit(&form, 0, "{a}", "2").await;
        assert!(message(&result).contains("not acknowledged"));
        assert!(!message(&result).contains("private"));
        assert_eq!(result["actions"].as_array().unwrap().len(), 3);
        for op in [
            "extension:refresh-session",
            "extension:search-text-changed",
            "extension:load-more",
        ] {
            f.h.call(op, json!({"sessionId":form["sessionId"]})).await;
        }
        assert_eq!(f.copies.lock().unwrap().len(), 1);
        let retry =
            f.h.call(
                "extension:invoke-action",
                json!({"sessionId":form["sessionId"],"actionId":result["actions"][2]["id"]}),
            )
            .await;
        assert_eq!(f.copies.lock().unwrap().len(), 2);
        assert_ne!(result["actions"], retry["actions"]);
    }
    for reply in [
        json!({"ok":true}),
        json!({"ok":true,"error":null,"success":true}),
    ] {
        let f = Fixture::new(Ok(reply));
        let result = f.submit(&f.launch().await, 0, "{a}", "2").await;
        assert!(message(&result).contains("Copied"));
        assert_eq!(result["actions"].as_array().unwrap().len(), 2);
    }
}
#[tokio::test]
async fn storage_preserves_unknowns_and_rejects_mid_session_corruption() {
    let f = Fixture::new(Ok(json!({"ok":true})));
    let form = f.launch().await;
    std::fs::create_dir_all(f.store().parent().unwrap()).unwrap();
    std::fs::write(f.store(), r#"{"unknown":{"keep":true},"indent":"4"}"#).unwrap();
    let change=f.h.call("extension:invoke-action",json!({"sessionId":form["sessionId"],"actionId":form["root"]["children"][1]["props"]["actionId"],"formValues":{"value":"8"}})).await;
    assert!(change["root"]["children"][0]["props"]
        .get("value")
        .is_none());
    assert!(change["root"]["children"][1]["props"]
        .get("value")
        .is_none());
    let store: Value = serde_json::from_slice(&std::fs::read(f.store()).unwrap()).unwrap();
    assert_eq!(store, json!({"unknown":{"keep":true},"indent":"8"}));
    assert_eq!(
        f.launch().await["root"]["children"][1]["props"]["value"],
        "8"
    );
    for bytes in [
        "{bad",
        r#"{"indent":3,"future":true}"#,
        r#"{"indent":"16"}"#,
        "[]",
    ] {
        std::fs::write(f.store(), bytes).unwrap();
        let fresh = f.launch().await;
        let r = f.submit(&fresh, 1, "{a}", "2").await;
        assert!(message(&r).contains("storage"));
        assert_eq!(std::fs::read_to_string(f.store()).unwrap(), bytes);
        // Reopen provides a new valid action token, never resets malformed storage.
        let fresh = f.launch().await;
        assert!(message(&fresh).contains("storage"));
        let r = f.submit(&fresh, 1, "{a}", "2").await;
        assert!(message(&r).contains("storage"));
        assert_eq!(std::fs::read_to_string(f.store()).unwrap(), bytes);
    }
}
#[tokio::test]
async fn cloned_refresh_search_and_forged_tokens_do_not_revive() {
    let f = Fixture::new(Ok(json!({"ok":true})));
    let mut state = json!({});
    let p = json!({});
    let form = format_graphql::handle(&f.ctx, "index", "extension:run-command", &p, &mut state)
        .await
        .unwrap();
    let committed = state.clone();
    let mut clone = state.clone();
    let refreshed =
        format_graphql::handle(&f.ctx, "index", "extension:refresh-session", &p, &mut clone)
            .await
            .unwrap();
    assert_eq!(clone, committed);
    assert_eq!(form["actions"], refreshed["actions"]);
    let returned = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:search-text-changed",
        &p,
        &mut clone,
    )
    .await
    .unwrap();
    assert_eq!(returned["mode"], "unchanged");
    assert_eq!(clone, committed);
    let mut discarded = json!({});
    let ghost =
        format_graphql::handle(&f.ctx, "index", "extension:run-command", &p, &mut discarded)
            .await
            .unwrap();
    assert_ne!(ghost["actions"], form["actions"]);
    let p = json!({"actionId":ghost["actions"][0]["id"],"formValues":{"input":"{a}","indent":"2"}});
    assert!(
        format_graphql::handle(&f.ctx, "index", "extension:invoke-action", &p, &mut state)
            .await
            .is_err()
    );
    for values in [
        Value::Null,
        json!({"input":"{a}"}),
        json!({"input":"{a}","indent":"2x"}),
        json!({"input":true,"indent":"2"}),
    ] {
        let p = json!({"actionId":form["actions"][0]["id"],"formValues":values});
        assert!(
            format_graphql::handle(&f.ctx, "index", "extension:invoke-action", &p, &mut state)
                .await
                .is_err()
        );
    }
    assert!(f.copies.lock().unwrap().is_empty());
}
#[tokio::test]
async fn cancellation_checkpoints_uncertain_intent_and_does_not_automatically_recopy() {
    let f = Fixture::new(Ok(json!({"ok":true})));
    let entered = Arc::new(tokio::sync::Notify::new());
    let notify = entered.clone();
    let count = Arc::new(AtomicUsize::new(0));
    let record = count.clone();
    let ctx = CoreContext::new(f.ctx.paths.clone(), None);
    ctx.set_router(Arc::new(move |_, _| {
        let notify = notify.clone();
        let record = record.clone();
        Box::pin(async move {
            record.fetch_add(1, Ordering::SeqCst);
            notify.notify_one();
            std::future::pending().await
        })
    }))
    .unwrap();
    let mut state = json!({});
    let form = format_graphql::handle(
        &ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    let p = json!({"actionId":form["actions"][0]["id"],"formValues":{"input":"{a}","indent":"2"}});
    {
        let pending =
            format_graphql::handle(&ctx, "index", "extension:invoke-action", &p, &mut state);
        tokio::pin!(pending);
        tokio::select! {_=&mut pending=>panic!("unexpected completion"),_=entered.notified()=>{}}
    }
    assert_eq!(state["intent"]["phase"], "uncertain");
    let r = format_graphql::handle(
        &ctx,
        "index",
        "extension:refresh-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(r["actions"].as_array().unwrap().len(), 3);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    let h = Harness::new(ctx);
    let form = h
        .call(
            "extension:run-command",
            json!({"extensionId":"raycast.format-graphql","commandName":"index"}),
        )
        .await;
    let h2 = h.clone();
    let p = json!({"sessionId":form["sessionId"],"actionId":form["actions"][0]["id"],"formValues":{"input":"{a}","indent":"2"}});
    let task = tokio::spawn(async move { h2.call("extension:invoke-action", p).await });
    entered.notified().await;
    tokio::time::timeout(Duration::from_secs(2), h.clear())
        .await
        .unwrap();
    assert_eq!(task.await.unwrap()["ok"], false);
}

fn replacement(v: &Value) -> &Value {
    &v["root"]["children"][0]["props"]
}

#[tokio::test]
async fn replacement_identity_trace() {
    let f = Fixture::new(Ok(Value::Null)); // uncertain copy permits deliberate retry
    let mut state = json!({});
    let initial = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    let submit = |form: &Value, index: usize| {
        json!({
            "actionId": form["actions"][index]["id"],
            "formValues": {"input":"{a}","indent":"2"}
        })
    };
    let first = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:invoke-action",
        &submit(&initial, 1),
        &mut state,
    )
    .await
    .unwrap();
    let first_id = replacement(&first)["replacementId"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(!first_id.is_empty());
    assert_eq!(state["replacement"]["id"], first_id);
    let committed = state.clone();
    let mut clone = state.clone();
    let refresh = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:refresh-session",
        &json!({}),
        &mut clone,
    )
    .await
    .unwrap();
    assert_eq!(
        clone, committed,
        "refresh clone must not mint or commit an identity"
    );
    assert_eq!(replacement(&refresh), replacement(&first));
    let indent = format_graphql::handle(
        &f.ctx, "index", "extension:invoke-action",
        &json!({"actionId":first["root"]["children"][1]["props"]["actionId"],"formValues":{"value":"4"}}),
        &mut state,
    ).await.unwrap();
    assert!(replacement(&indent).get("replacementId").is_none());
    assert_eq!(state["replacement"]["id"], first_id);
    let indent_refresh = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:refresh-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(replacement(&indent_refresh), replacement(&first));
    let copy = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:invoke-action",
        &submit(&indent, 0),
        &mut state,
    )
    .await
    .unwrap();
    let copy_id = replacement(&copy)["replacementId"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(copy_id, first_id);
    assert_eq!(state["replacement"]["id"], copy_id);
    assert_eq!(state["intent"]["phase"], "uncertain");
    let copy_refresh = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:refresh-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(replacement(&copy_refresh), replacement(&copy));
    let retry = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":copy["actions"][2]["id"]}),
        &mut state,
    )
    .await
    .unwrap();
    assert!(replacement(&retry).get("replacementId").is_none());
    assert_eq!(state["replacement"]["id"], copy_id);
    let retry_refresh = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:refresh-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(replacement(&retry_refresh), replacement(&copy));
    assert_eq!(f.copies.lock().unwrap().len(), 2);
    let second = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:invoke-action",
        &submit(&retry, 1),
        &mut state,
    )
    .await
    .unwrap();
    let second_id = replacement(&second)["replacementId"].as_str().unwrap();
    assert_ne!(second_id, first_id);
    assert_ne!(second_id, copy_id);
    assert_eq!(state["replacement"]["id"], second_id);
    for field in ["value", "replacementBase"] {
        assert_eq!(replacement(&first)[field], replacement(&copy)[field]);
        assert_eq!(replacement(&first)[field], replacement(&second)[field]);
    }

    // Cancel a real copy after its checkpoint; refresh/retry retain the identity.
    let entered = Arc::new(tokio::sync::Notify::new());
    let notify = entered.clone();
    let ctx = CoreContext::new(f.ctx.paths.clone(), None);
    ctx.set_router(Arc::new(move |_, _| {
        let notify = notify.clone();
        Box::pin(async move {
            notify.notify_one();
            std::future::pending().await
        })
    }))
    .unwrap();
    let p = submit(&second, 0);
    {
        let pending =
            format_graphql::handle(&ctx, "index", "extension:invoke-action", &p, &mut state);
        tokio::pin!(pending);
        tokio::select! {_=&mut pending=>panic!("unexpected completion"),_=entered.notified()=>{}}
    }
    assert_eq!(state["intent"]["phase"], "uncertain");
    let cancelled_id = state["replacement"]["id"].as_str().unwrap().to_owned();
    assert_ne!(cancelled_id, second_id);
    let cancelled_state = state.clone();
    let cancelled_refresh = format_graphql::handle(
        &ctx,
        "index",
        "extension:refresh-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(state, cancelled_state);
    assert_eq!(
        replacement(&cancelled_refresh)["replacementId"],
        cancelled_id
    );
    let cancelled_retry = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":cancelled_refresh["actions"][2]["id"]}),
        &mut state,
    )
    .await
    .unwrap();
    assert!(replacement(&cancelled_retry).get("replacementId").is_none());
    assert_eq!(state["replacement"]["id"], cancelled_id);
    let cancelled_retry_refresh = format_graphql::handle(
        &f.ctx,
        "index",
        "extension:refresh-session",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(
        replacement(&cancelled_retry_refresh),
        replacement(&cancelled_refresh)
    );
    if let Some(path) = std::env::var_os("FORMAT_GRAPHQL_RENDERER_TRACE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&json!({
                "initial":initial,"first":first,"refresh":refresh,"indent":indent,
                "indentRefresh":indent_refresh,"copy":copy,"copyRefresh":copy_refresh,
                "retry":retry,"retryRefresh":retry_refresh,"second":second,
                "cancelledRefresh":cancelled_refresh,"cancelledRetry":cancelled_retry,
                "cancelledRetryRefresh":cancelled_retry_refresh
            }))
            .unwrap(),
        )
        .unwrap();
    }
}
