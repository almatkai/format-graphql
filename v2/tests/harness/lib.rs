//! Explicit Tauri/registry/dispatch stubs around unchanged pinned common/SDK/
//! NativeSessions and the exact production adapter. No OS calls are made.
extern crate self as tauri;
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit<T: serde::Serialize + Clone>(&self, event: &str, value: T) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit<T: serde::Serialize + Clone>(&self, _event: &str, _value: T) -> Result<(), String> {
        Ok(())
    }
}
pub mod native_core {
    pub mod common;
    pub mod extensions {
        pub mod format_graphql;
        #[cfg(not(test))]
        mod native;
        #[allow(dead_code)]
        mod v2_sdk;
        #[allow(dead_code)]
        mod registry {
            use super::super::common::{CoreContext, CoreResult};
            use serde_json::{json, Value};
            pub fn installed_command(
                ctx: &CoreContext,
                _extension: &str,
                _command: &str,
            ) -> Option<Value> {
                (!ctx.paths.data_dir.join("uninstalled").exists()).then(|| json!({"fixture":true}))
            }
            pub fn preferences(_extension: &Value, _command: Option<&str>) -> CoreResult {
                Ok(json!({}))
            }
        }
        #[allow(dead_code)]
        mod v2_runtime {
            use super::super::common::{CoreContext, CoreResult};
            use serde_json::Value;
            pub fn slug(id: &str) -> Option<&str> {
                let s = id
                    .strip_prefix("com.tezbar.native.")
                    .or_else(|| id.strip_prefix("raycast."))
                    .unwrap_or(id);
                (s == "format-graphql").then_some(s)
            }
            pub fn supports(id: &str, cmd: &str) -> bool {
                slug(id).is_some() && super::format_graphql::commands().contains(&cmd)
            }
            pub fn canonical_id(s: &str) -> String {
                format!("com.tezbar.native.{s}")
            }
            pub async fn handle(
                ctx: &CoreContext,
                _slug: &str,
                cmd: &str,
                op: &str,
                p: &Value,
                state: &mut Value,
            ) -> CoreResult {
                super::format_graphql::handle(ctx, cmd, op, p, state).await
            }
        }
        #[cfg(not(test))]
        #[derive(Clone)]
        pub struct Harness {
            pub ctx: super::common::CoreContext,
            native: native::NativeSessions,
            files: std::sync::Arc<std::sync::Mutex<()>>,
        }
        #[cfg(not(test))]
        impl Harness {
            pub fn new(ctx: super::common::CoreContext) -> Self {
                Self {
                    ctx,
                    native: native::NativeSessions::default(),
                    files: Default::default(),
                }
            }
            pub async fn call(&self, op: &str, p: serde_json::Value) -> serde_json::Value {
                self.native
                    .handle(&self.ctx, &self.files, op, &[p])
                    .await
                    .expect("native dispatch")
                    .expect("native result")
            }
            pub async fn clear(&self) {
                self.native.clear().await;
            }
            pub async fn clear_slug(&self) {
                self.native.clear_slug("format-graphql").await;
            }
        }
    }
}
