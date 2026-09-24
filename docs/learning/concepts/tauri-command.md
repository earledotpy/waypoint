# Tauri commands

## In one line

`#[tauri::command]` marks a Rust function that the web page can call with `invoke("name", { args })`. The arguments go over as JSON, and the function's `Result` comes back as JSON: `Ok` resolves the JavaScript promise and `Err` rejects it.

## Where it appears here

- `src-tauri/src/commands.rs`, `create_node` and `list_nodes`: the two commands. Each locks the connection, calls one function in `waypoint_domain` or `waypoint_read`, and turns the error into a string.
- `src-tauri/src/lib.rs`, `run`: `.invoke_handler(tauri::generate_handler![…])` lists the commands the page may call.
- `src/api.ts`, `createNode` and `listNodes`: the only calls to `invoke` in the frontend.
- `src/SkillNodes.test.tsx`: replaces the Rust side with `mockIPC`, so the page can be tested without Tauri.

## What it does

A Tauri app is one program with two halves: Rust, and a webview (a built-in browser engine) that shows the React page. A command is how the page asks Rust to do something.

Follow one call, `createNode("Read a Rust compiler error", "")` in `api.ts`:

1. **TypeScript** runs `invoke("create_node", { title, description })`. `invoke` turns the object into JSON, `{"title": "Read a Rust compiler error", "description": ""}`, and hands it, with the name `"create_node"`, to the Rust half of the same program. It returns a `Promise` straight away.
2. **`generate_handler!`** builds one function that looks at the name and picks the Rust function to run. Only the commands listed in it can be called. A command missing from the list still compiles, and `invoke` then rejects at runtime with "Command create_node not found". The list must be in one `generate_handler!` call: a second `.invoke_handler(…)` replaces the first, it doesn't add to it.
3. **`#[tauri::command]`** is a macro. It writes the code that turns that JSON into the Rust arguments, matching them by name. `title: String` reads the `"title"` key. An argument whose type is `State<'_, Db>` isn't read from the JSON at all. Tauri fills it in from managed state (see [Tauri managed state](tauri-managed-state.md)).
4. **The function runs** like any Rust function. `create_node` returns a `Result<SkillNode, String>`.
5. **The result goes back as JSON.** `Ok(node)` is turned into JSON by serde (`SkillNode` derives `Serialize`, see [Structs and `#[derive]`](struct-and-derive.md)), and the promise resolves to a plain object with the same field names. `Err(message)` makes the promise reject with the message string. That's why `SkillNodes.tsx` catches the error and shows it with `String(e)`: the "exception" is just the text. (For promises themselves, see [Promises and `async` / `await`](promises-and-async-await.md).)

Some details that bite:

- **Argument names.** Tauri expects the JavaScript keys in camelCase. A Rust argument `due_at` would be sent as `{ dueAt: … }`. Both arguments here are one word, so the names are identical on both sides. Field names *inside* a returned struct are not converted, which is why `SkillNode` arrives with `created_at`.
- **Errors must turn into JSON too.** `DomainError` doesn't implement `Serialize`, so a command can't return it. `.map_err(|e| e.to_string())` swaps it for its `Display` text, which is what the user sees. The one error written for users, `EmptyTitle`, reads "A node needs a title.".
- **Which thread.** A command without `async` runs on the main thread, the one that also draws the window, so a slow command freezes the app. Both commands here run one small SQLite query, so that's fine for now.
- **Types aren't shared.** Nothing checks that `api.ts`'s `SkillNode` type matches the Rust struct, or that `"create_node"` is spelled right. That's why `api.ts` is the only file that calls `invoke`: the names are written once, next to a comment pointing at `commands.rs`.

## Python comparison

The nearest Python idea is a Flask or FastAPI route:

```python
@app.post("/create_node")
def create_node(title: str, description: str) -> SkillNode:
    ...
```

The decorator registers a function under a name, the framework turns JSON into its arguments, and the return value goes back as JSON. `#[tauri::command]` plus `generate_handler!` does the same job, and `invoke("create_node", …)` plays the part of `fetch("/create_node", …)`.

Where it breaks:

- **There's no network.** A Flask route listens on a port, and anything that can reach the port can call it. A Tauri command is called by the app's own webview, inside the same program, and no port is opened. There are no URLs, HTTP verbs or status codes to design.
- **Registering is explicit.** Flask's decorator registers the route by itself. In Tauri the attribute only prepares the function, and nothing is callable until it's listed in `generate_handler!`.
- **Errors are values.** A Python route that raises gets turned into a 500 response. A Tauri command returns `Err`, and the frontend gets exactly the text inside it (see [`Result` and `?`](result-and-question-mark.md)). A Rust *panic* in a command, such as the poisoned-lock `.expect`, isn't turned into a nice message.

## Why this code uses it

Commands are the only door between the UI and the domain (architecture doc §1: "Tauri command invocations (typed, one call per intent)"). The UI never touches SQLite. It asks for one thing at a time, and the domain decides whether it's allowed. Keeping each command thin (ADR 0001 §2) means every rule, such as "a title can't be blank", lives in `waypoint-domain` and is tested there, and the commands themselves have no logic that needs a test of its own.

## See also

- [Tauri managed state](tauri-managed-state.md): where `State<'_, Db>` comes from.
- [React state and effects](react-state-and-effect.md): the screen that calls these commands.
- [ADR 0001 §2](../../adr/0001-crate-layout-write-hiding-and-frontend-tooling.md#2-who-may-depend-on-whom): which crates a command may call.
- Tauri documentation, [Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/)
