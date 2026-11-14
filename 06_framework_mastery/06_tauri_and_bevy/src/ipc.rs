//! A simulation of Tauri's command layer, in plain Rust.
//!
//! In a Tauri app the user interface is a web page (HTML/JS) shown by the
//! system's webview, and the logic is Rust. They can't call each other's
//! functions directly: they live in different worlds and talk by sending
//! MESSAGES over IPC (inter-process communication). The page sends
//! `invoke("add_note", { title: "milk" })`; Tauri turns that into a JSON
//! message, checks the window is allowed to call `add_note`, deserializes
//! the arguments into your Rust types, calls your `#[tauri::command]`
//! function, and sends its serialized result back.
//!
//! This module does the same with `serde_json` strings, so you can see
//! every step.

use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What the page sends. `id` pairs the answer with the right JavaScript
/// promise; `window` says which window is asking (for permissions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvokeRequest {
    pub id: u64,
    pub window: String,
    pub cmd: String,
    #[serde(default)]
    pub args: Value,
}

/// What goes back: `{"id": 1, "ok": ...}` resolves the promise,
/// `{"id": 1, "error": "..."}` rejects it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvokeResponse {
    pub id: u64,
    #[serde(flatten)]
    pub body: ResponseBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseBody {
    Ok(Value),
    Error(String),
}

/// A message the Rust side pushes to the page, unasked:
/// `listen("notes-changed", handler)` in JavaScript receives it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMessage {
    pub event: String,
    pub payload: Value,
}

/// Values the app shares with every command: Tauri's `app.manage(value)`
/// and `State<'_, T>`. Commands may run on several threads at once, so
/// everything stored must be `Send + Sync` (use a `Mutex` for data that
/// changes).
#[derive(Default)]
pub struct States {
    values: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl States {
    pub fn manage<T: Send + Sync + 'static>(&mut self, value: T) {
        self.values.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// The managed `T`.
    ///
    /// # Errors
    ///
    /// If nothing of type `T` was managed: Tauri reports the same mistake
    /// when a command asks for `State<T>` that was never `manage`d.
    pub fn get<T: 'static>(&self) -> Result<&T, String> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
            .ok_or_else(|| format!("state {} is not managed", type_name::<T>()))
    }
}

/// Events waiting to be delivered to the page.
#[derive(Default)]
pub struct Events {
    queue: Mutex<Vec<EventMessage>>,
}

impl Events {
    /// Tauri's `app.emit("name", payload)`.
    pub fn emit(&self, event: &str, payload: impl Serialize) {
        let payload = serde_json::to_value(payload).unwrap_or(Value::Null);
        self.queue.lock().expect("events lock").push(EventMessage {
            event: event.to_string(),
            payload,
        });
    }

    /// Takes every waiting event (what the page's listeners receive).
    pub fn drain(&self) -> Vec<EventMessage> {
        std::mem::take(&mut *self.queue.lock().expect("events lock"))
    }
}

/// What a command can see besides its arguments.
pub struct Context<'a> {
    pub state: &'a States,
    pub events: &'a Events,
    /// The window that called (Tauri's `WebviewWindow` parameter).
    pub window: &'a str,
}

/// One security rule, as in a Tauri 2 capability file
/// (`src-tauri/capabilities/*.json`): these windows may use these commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    pub identifier: String,
    pub windows: Vec<String>,
    pub commands: Vec<String>,
}

/// A type-erased command: JSON in, JSON out.
type Handler = Box<dyn Fn(&Context<'_>, Value) -> Result<Value, String> + Send + Sync>;

/// The command registry, like `tauri::Builder` with `.manage(...)`,
/// `.invoke_handler(generate_handler![...])` and capability files.
#[derive(Default)]
pub struct Router {
    handlers: HashMap<&'static str, Handler>,
    capabilities: Vec<Capability>,
    states: States,
    events: Events,
}

impl Router {
    pub fn new() -> Self {
        Router::default()
    }

    /// Registers a command. `A` is the arguments struct (from JSON), `R` the
    /// result (to JSON). This generic wrapper is what `#[tauri::command]`
    /// generates for you: deserialize, call, serialize.
    pub fn command<A, R, F>(mut self, name: &'static str, handler: F) -> Self
    where
        A: DeserializeOwned,
        R: Serialize,
        F: Fn(&Context<'_>, A) -> Result<R, String> + Send + Sync + 'static,
    {
        let erased: Handler = Box::new(move |ctx, args| {
            let args: A = serde_json::from_value(args)
                .map_err(|e| format!("invalid arguments for {name}: {e}"))?;
            let result = handler(ctx, args)?;
            serde_json::to_value(result).map_err(|e| format!("cannot send the result: {e}"))
        });
        self.handlers.insert(name, erased);
        self
    }

    /// Shares a value with every command (`app.manage(value)`).
    pub fn manage<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        self.states.manage(value);
        self
    }

    /// Adds security rules from JSON, as Tauri reads capability files.
    ///
    /// # Errors
    ///
    /// If the JSON isn't a list of capabilities (unknown keys included).
    pub fn capabilities_json(mut self, json: &str) -> Result<Self, String> {
        let mut list: Vec<Capability> =
            serde_json::from_str(json).map_err(|e| format!("invalid capabilities: {e}"))?;
        self.capabilities.append(&mut list);
        Ok(self)
    }

    /// Whether some capability lets `window` call `cmd`. Deny by default:
    /// no rule, no access.
    pub fn allows(&self, window: &str, cmd: &str) -> bool {
        self.capabilities.iter().any(|c| {
            c.windows.iter().any(|w| w == window) && c.commands.iter().any(|allowed| allowed == cmd)
        })
    }

    /// Handles one request (already parsed).
    pub fn handle(&self, request: InvokeRequest) -> InvokeResponse {
        let body = match self.dispatch(&request) {
            Ok(value) => ResponseBody::Ok(value),
            Err(message) => ResponseBody::Error(message),
        };
        InvokeResponse {
            id: request.id,
            body,
        }
    }

    fn dispatch(&self, request: &InvokeRequest) -> Result<Value, String> {
        // Check permission BEFORE looking the command up, so a window that
        // isn't allowed can't even learn which commands exist.
        if !self.allows(&request.window, &request.cmd) {
            return Err(format!(
                "command {} not allowed for window {}",
                request.cmd, request.window
            ));
        }
        let handler = self
            .handlers
            .get(request.cmd.as_str())
            .ok_or_else(|| format!("command {} not found", request.cmd))?;
        let ctx = Context {
            state: &self.states,
            events: &self.events,
            window: &request.window,
        };
        handler(&ctx, request.args.clone())
    }

    /// Handles a raw JSON message from the page and returns the JSON reply:
    /// the whole IPC boundary in one function.
    pub fn handle_json(&self, raw: &str) -> String {
        let response = match serde_json::from_str::<InvokeRequest>(raw) {
            Ok(request) => self.handle(request),
            Err(e) => InvokeResponse {
                id: 0,
                body: ResponseBody::Error(format!("malformed message: {e}")),
            },
        };
        serde_json::to_string(&response).expect("a response always serializes")
    }

    /// Events emitted since the last call.
    pub fn take_events(&self) -> Vec<EventMessage> {
        self.events.drain()
    }

    /// The registered commands' names, sorted.
    pub fn command_names(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self.handlers.keys().copied().collect();
        names.sort();
        names
    }
}

/// The page's side: builds `invoke` messages with fresh ids, like
/// `@tauri-apps/api/core`'s `invoke(cmd, args)`.
pub struct Frontend {
    pub window: String,
    next_id: u64,
}

impl Frontend {
    pub fn new(window: &str) -> Self {
        Frontend {
            window: window.to_string(),
            next_id: 1,
        }
    }

    /// The JSON message for `invoke(cmd, args)`.
    pub fn invoke_message(&mut self, cmd: &str, args: Value) -> String {
        let request = InvokeRequest {
            id: self.next_id,
            window: self.window.clone(),
            cmd: cmd.to_string(),
            args,
        };
        self.next_id += 1;
        serde_json::to_string(&request).expect("a request always serializes")
    }

    /// Sends `invoke(cmd, args)` to `router` and waits for the result, like
    /// `await invoke(...)`: `Ok` resolves, `Err` rejects.
    pub fn invoke(&mut self, router: &Router, cmd: &str, args: Value) -> Result<Value, String> {
        let reply = router.handle_json(&self.invoke_message(cmd, args));
        let response: InvokeResponse = serde_json::from_str(&reply).map_err(|e| e.to_string())?;
        match response.body {
            ResponseBody::Ok(value) => Ok(value),
            ResponseBody::Error(message) => Err(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AddArgs {
        first_number: i64,
        second_number: i64,
    }

    fn router() -> Router {
        Router::new()
            .command("add", |_, args: AddArgs| {
                Ok(args.first_number + args.second_number)
            })
            .command("fail", |_, _: Value| -> Result<(), String> {
                Err("it went wrong".into())
            })
            .capabilities_json(
                r#"[{"identifier": "main", "windows": ["main"], "commands": ["add", "fail", "missing"]}]"#,
            )
            .unwrap()
    }

    #[test]
    fn a_round_trip_through_json() {
        let reply = router().handle_json(
            r#"{"id": 7, "window": "main", "cmd": "add", "args": {"firstNumber": 2, "secondNumber": 3}}"#,
        );
        assert_eq!(reply, r#"{"id":7,"ok":5}"#);
    }

    #[test]
    fn errors_reject_the_promise() {
        let mut page = Frontend::new("main");
        let router = router();
        assert_eq!(
            page.invoke(&router, "fail", json!({})),
            Err("it went wrong".into())
        );
        assert_eq!(
            page.invoke(&router, "missing", json!({})),
            Err("command missing not found".into())
        );
        let err = page
            .invoke(&router, "add", json!({"firstNumber": "two"}))
            .unwrap_err();
        assert!(
            err.starts_with("invalid arguments for add: invalid type: string \"two\""),
            "{err}"
        );
    }

    #[test]
    fn deny_by_default() {
        let router = router();
        let mut other = Frontend::new("popup");
        assert_eq!(
            other.invoke(&router, "add", json!({"firstNumber": 1, "secondNumber": 1})),
            Err("command add not allowed for window popup".into())
        );
        assert!(router.allows("main", "add"));
        assert!(!router.allows("main", "delete_everything"));
    }

    #[test]
    fn malformed_messages_get_an_error_reply() {
        let reply = router().handle_json("{not json");
        let response: InvokeResponse = serde_json::from_str(&reply).unwrap();
        assert_eq!(response.id, 0);
        assert!(
            matches!(response.body, ResponseBody::Error(m) if m.starts_with("malformed message"))
        );
    }

    #[test]
    fn state_and_events() {
        let router = Router::new()
            .manage(Mutex::new(0u32))
            .command("bump", |ctx, _: Value| {
                let counter = ctx.state.get::<Mutex<u32>>()?;
                let mut n = counter.lock().map_err(|e| e.to_string())?;
                *n += 1;
                ctx.events.emit("bumped", *n);
                Ok(*n)
            })
            .command("unmanaged", |ctx, _: Value| {
                ctx.state.get::<String>().cloned()
            })
            .capabilities_json(
                r#"[{"identifier": "c", "windows": ["main"], "commands": ["bump", "unmanaged"]}]"#,
            )
            .unwrap();
        let mut page = Frontend::new("main");
        page.invoke(&router, "bump", json!(null)).unwrap();
        assert_eq!(page.invoke(&router, "bump", json!(null)), Ok(json!(2)));
        let events = router.take_events();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[1],
            EventMessage {
                event: "bumped".into(),
                payload: json!(2)
            }
        );
        assert!(router.take_events().is_empty(), "drained");
        assert_eq!(
            page.invoke(&router, "unmanaged", json!(null)),
            Err("state alloc::string::String is not managed".into())
        );
    }

    #[test]
    fn capability_files_reject_unknown_keys() {
        let err = Router::new()
            .capabilities_json(
                r#"[{"identifier": "x", "windows": [], "commands": [], "allow_all": true}]"#,
            )
            .err()
            .unwrap();
        assert!(err.contains("unknown field `allow_all`"), "{err}");
    }

    #[test]
    fn requests_and_responses_have_a_stable_shape() {
        let mut page = Frontend::new("main");
        assert_eq!(
            page.invoke_message("greet", json!({"name": "Ama"})),
            r#"{"id":1,"window":"main","cmd":"greet","args":{"name":"Ama"}}"#
        );
        let error = InvokeResponse {
            id: 3,
            body: ResponseBody::Error("no".into()),
        };
        assert_eq!(
            serde_json::to_string(&error).unwrap(),
            r#"{"id":3,"error":"no"}"#
        );
    }
}
