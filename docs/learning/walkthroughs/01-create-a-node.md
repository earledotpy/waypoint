# Walkthrough 01 — Create a node

Milestone: 1 (walking skeleton)
Pinned to: [`d0271bf`](https://github.com/earledotpy/waypoint/commit/d0271bf09f7868b7a2b55f6686f798892981be91), the merge of "Create and list nodes from the React screen" (#33). Every code link below is a permalink to this commit, so the line numbers stay right even after the code moves on.

## The action

The user types a title, say "Read a Rust compiler error", into **Title (required)**, leaves **Description (optional)** empty, and clicks **Create node**.

What they see: the node appears at the bottom of the list as `Read a Rust compiler error · available · <the time it was made>`, and both boxes empty out, ready for the next one.

If the title is blank, or only spaces, nothing is added. The text "A node needs a title." appears above the button, and whatever they typed stays in the boxes so they can fix it. That failure path has [its own section](#when-the-title-is-blank) below.

## The path

The call goes down from React, through Tauri and the domain, to SQLite, and the stored node comes back up the same way.

| Layer | File | Function | What happens |
|---|---|---|---|
| React | `src/SkillNodes.tsx` | [`handleSubmit`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L42-L58) | Stops the page reloading, then calls `createNode` with what's in the two boxes and waits. |
| React → Rust | `src/api.ts` | [`createNode`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/api.ts#L28-L30) | Calls `invoke("create_node", { title, description })`, which sends the arguments to Rust as JSON. |
| Tauri command | `src-tauri/src/commands.rs` | [`create_node`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/commands.rs#L26-L40) | Locks the one database connection, calls the domain, and turns an error into its message. |
| Tauri state | `src-tauri/src/lib.rs` | [`Db`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/lib.rs#L17-L25), put there by [`run`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/lib.rs#L52-L83) | Where the connection lives. `run` opened it at startup and handed it to Tauri; the command's `db.lock()` borrows it. |
| Domain | `crates/waypoint-domain/src/node.rs` | [`create_node`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L11-L60) | Trims the title, rejects a blank one, makes a new id, and inserts the row. |
| SQLite | `crates/waypoint-domain/migrations/0001_create_skill_node.sql` | [the `skill_node` table](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/migrations/0001_create_skill_node.sql#L5-L13) | `INSERT … RETURNING` stores the row, after its `CHECK`s pass, and hands back the row as stored. |
| Back to Rust | `crates/waypoint-read/src/node.rs` | [`SkillNode::from_row`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-read/src/node.rs#L36-L54) | Turns the returned row into a [`SkillNode`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-read/src/node.rs#L8-L33) struct. |
| Back to React | `src-tauri/src/commands.rs` → `src/SkillNodes.tsx` | [`create_node`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/commands.rs#L26-L40) → [`handleSubmit`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L42-L58) | Tauri turns `Ok(node)` into JSON. `await` gets it as a plain object, `setNodes` stores a new list, and React draws the screen again. |

## Step by step

### 1. React: the click

[`SkillNodes`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L9-L117) is the whole screen. It remembers four values with `useState`: the list of `nodes`, the `title` and `description` being typed, and the last `error` (see [React state and effects](../concepts/react-state-and-effect.md)). Every keystroke in a box calls `setTitle` or `setDescription`, so by the time the button is clicked, `title` already holds "Read a Rust compiler error".

The button sits inside a `<form onSubmit={handleSubmit}>` ([JSX](../concepts/jsx.md)), so clicking it runs [`handleSubmit`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L42-L58). Its first line, `event.preventDefault()`, stops the browser's old habit of reloading the page on submit. Then:

```ts
const created = await createNode(title, description);
```

`handleSubmit` is an `async` function, so `await` pauses it here until Rust answers, without freezing the screen (see [Promises and `async` / `await`](../concepts/promises-and-async-await.md)).

### 2. `api.ts`: crossing into Rust

[`createNode`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/api.ts#L28-L30) is one line:

```ts
return invoke<SkillNode>("create_node", { title, description });
```

`invoke` comes from Tauri's JavaScript library. It turns `{ title, description }` into JSON and hands it, with the name `"create_node"`, to the Rust half of the same program. There is no network and no port: the page and Rust are one app. `api.ts` is the only file that calls `invoke`, so the command name and argument names are written once, right next to the [`SkillNode`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/api.ts#L14-L22) type that describes what comes back.

On the Rust side, the list in [`run`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/lib.rs#L75-L78)'s `generate_handler![…]` is what lets Tauri find `create_node` by its name. The details are in [Tauri commands](../concepts/tauri-command.md).

### 3. Tauri command: lock, call, translate

[`create_node`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/commands.rs#L26-L40) in `commands.rs` receives three arguments. `title` and `description` are read from the JSON by name. `db: State<'_, Db>` isn't in the JSON at all: Tauri fills it in from the value `run` registered at startup.

That value is [`Db`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/lib.rs#L17-L25), which is just another name for `Mutex<Connection>`: the one SQLite connection, wrapped in a lock. [`run`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/lib.rs#L52-L83) opened the database once, before any command could arrive, and gave it to Tauri with `app.manage(db)` (see [Tauri managed state](../concepts/tauri-managed-state.md)). The command's first line takes the lock:

```rust
let conn = db.lock().expect(POISONED);
```

While `conn` exists, no other command can use the connection. When the function ends, `conn` goes away and the lock is released on its own. (`.expect` only fires if an earlier command crashed while holding the lock. The [`POISONED`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/commands.rs#L18-L24) comment explains why stopping then is the safe choice.)

The second line does the real work, and it's the whole rest of the command:

```rust
waypoint_domain::create_node(&conn, &title, &description).map_err(|e| e.to_string())
```

The `&` hands the domain a borrowed look at the connection and the text, rather than giving them away (see [Borrowing in signatures](../concepts/borrowing-in-signatures.md)). The domain returns a `Result<SkillNode, DomainError>` (see [`Result` and `?`](../concepts/result-and-question-mark.md)). `.map_err` leaves an `Ok` alone and turns an `Err(DomainError)` into `Err(String)`, its message. It has to: the page can only receive things that become JSON, and a plain message is what it shows the user (see "Errors must turn into JSON too" in [Tauri commands](../concepts/tauri-command.md)).

The command has no rules of its own. It doesn't check the title. That's deliberate: commands stay thin, and every rule lives in the domain ([ADR 0001 §2](../../adr/0001-crate-layout-write-hiding-and-frontend-tooling.md#2-who-may-depend-on-whom)).

### 4. Domain: the rules

[`create_node`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L11-L60) in `waypoint-domain` is where the decisions are made. It's the only crate allowed to write to SQLite. In order:

1. **Trim.** `title.trim()` drops spaces from both ends, and `description.trim_end()` from the end only, because leading spaces mean something in Markdown (four of them make a code block).
2. **Reject a blank title.** If the trimmed title is empty, return `Err(DomainError::EmptyTitle)` straight away, before SQLite is touched. (More on this in [When the title is blank](#when-the-title-is-blank).)
3. **Make an id.** `Uuid::now_v7()` makes a new UUIDv7. Its first part is the current time, so ids sort in the order nodes were made ([ADR 0002 §1](../../adr/0002-table-conventions.md#1-ids-are-uuidv7-stored-as-text)).
4. **Insert and read back in one statement.** [`conn.query_row(…)`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L52-L58) runs the SQL shown in section 5 below with the id, title and description as `?1`, `?2`, `?3`. Passing them as parameters, not pasting them into the SQL text, means a title like `'); DROP TABLE` is stored as a title and never run as SQL.
5. **Pass on a failure.** The `?` at the end of `query_row(…)?` means "if SQLite failed, return that error now". A `rusqlite::Error` becomes a `DomainError::Database` on the way out, thanks to the [`From` impl](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/error.rs#L55-L59) in `error.rs` (see [Traits and `impl` blocks](../concepts/traits-and-impl.md)). If it worked, `Ok(node)` is returned.

### 5. SQLite: the row

The statement is:

```sql
INSERT INTO skill_node (id, external_id, title, description, state, created_at)
VALUES (?1, NULL, ?2, ?3, 'available', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
RETURNING id, external_id, title, description, state, created_at, retired_at
```

It writes into [the `skill_node` table](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/migrations/0001_create_skill_node.sql#L5-L13), which migration 0001 created when the app opened the database (see [SQLite migrations](../concepts/sqlite-migrations.md)). Three things to notice:

- `state` is always `'available'` for now. There are no prerequisites until milestone 2, so nothing can be Locked yet.
- SQLite, not Rust, supplies `created_at`, so every timestamp in the database has exactly the same format ([ADR 0002 §2](../../adr/0002-table-conventions.md#2-timestamps-are-iso-8601-utc-text-with-milliseconds)).
- Before the row is stored, SQLite checks the table's `CHECK` constraints on [`title`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/migrations/0001_create_skill_node.sql#L8) and [`state`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/migrations/0001_create_skill_node.sql#L10). On this path both pass, because the domain already made sure of it.

`RETURNING` hands back the row exactly as it was stored. So the caller gets what's really in the database, including the `created_at` SQLite chose, not what Rust meant to write.

### 6. Back up: row → struct → JSON → screen

The returned row goes to [`SkillNode::from_row`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-read/src/node.rs#L36-L54), which copies each column into a field of the [`SkillNode`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-read/src/node.rs#L8-L33) struct (see [`struct`, `Option` and `#[derive(…)]`](../concepts/struct-and-derive.md)). `SkillNode` lives in `waypoint-read`, not the domain, so crates that may only read can use the same type ([ADR 0001 §2](../../adr/0001-crate-layout-write-hiding-and-frontend-tooling.md#2-who-may-depend-on-whom)). `list_nodes` uses the same `from_row`, so a node looks the same however it was read.

The domain returns `Ok(node)`, the command passes it through `.map_err` untouched, and Tauri turns it into JSON. It can, because `SkillNode` has `#[derive(Serialize)]` ([ADR 0003](../../adr/0003-serde-and-derive.md)). The field names stay snake_case all the way, so `created_at` is the same word in SQL, Rust and TypeScript.

Back in [`handleSubmit`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L42-L58), `await` finishes and `created` holds the node as a plain object. Then:

```ts
setNodes([...nodes, created]);
setTitle("");
setDescription("");
setError(null);
```

`[...nodes, created]` is a *new* list with the node on the end. React only redraws when it's handed a new value, so changing the old list in place wouldn't show ([React state and effects](../concepts/react-state-and-effect.md)). The setters make React run `SkillNodes` again, and this time [`nodes.map(…)`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L103-L112) draws one more `<li>`, showing the title, the state and the time. Note that the screen does **not** call `list_nodes` again: the node it shows is the one `RETURNING` handed back.

### When the title is blank

Now the user types three spaces and clicks **Create node**. Steps 1 to 3 are the same: `handleSubmit` → `createNode("   ", "")` → `invoke` → the command locks the connection and calls the domain. The difference starts in the domain.

1. **Domain.** In [`create_node`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L17-L27), `"   ".trim()` is `""`, so `title.is_empty()` is true and the function returns `Err(DomainError::EmptyTitle)`. No id is made and SQLite is never asked.
2. **The error type.** [`DomainError`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/error.rs#L13-L25) is an enum with one variant per thing that can go wrong (see [Enums and `match`](../concepts/enums-and-match.md)). Its [`fmt`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/error.rs#L30-L38) (the `Display` impl, Rust's `__str__`) turns `EmptyTitle` into "A node needs a title.", a sentence written for the person typing.
3. **Tauri command.** [`create_node`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src-tauri/src/commands.rs#L26-L40)'s `.map_err(|e| e.to_string())` turns that into `Err("A node needs a title.")`. Tauri sends an `Err` back as a *rejected* promise, with the string inside.
4. **React.** In [`handleSubmit`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L42-L58), the rejected promise makes `await` throw, so the code jumps to `catch (e)`, like Python's `except`. `setError(String(e))` stores the message. The title and description are not cleared, so the user can fix them.
5. **The screen.** React draws again, and [`{error !== null && <p role="alert">{error}</p>}`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L92) now shows the paragraph. `role="alert"` tells screen readers to read it out as soon as it appears.

Why is a blank title rejected in the domain *and* by SQLite's `CHECK (length(trim(title)) > 0)`?

- **The domain is the real guard.** It runs first, it gives an error the user can act on, and every rule is meant to live there (`AGENTS.md`: invariants live in `waypoint-domain`). Without it, SQLite would still refuse the row, but the user would see "database error: CHECK constraint failed: …", which is a developer's message. The comment [above the check](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L22-L24) says exactly this.
- **The `CHECK` is a backstop.** It catches a bug that slips past the domain: a future write path that forgets to trim, say, or a curriculum import. It can't give a nice message, but it stops a bad row being stored at all ([ADR 0002 §4](../../adr/0002-table-conventions.md#4-enums-get-a-check-constraint-as-a-backstop)).

The HTML input deliberately has no `required` attribute. The browser would otherwise block the submit with its own message, and the domain's message is the one the user should see (the [comment in the JSX](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.tsx#L62-L68) explains).

The tests that pin this path down: [`create_node_rejects_a_blank_title`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L78-L90) (domain) and [`empty_title_message_is_a_sentence_for_the_user`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/error.rs#L71-L76) (the message); see [Rust unit tests](../concepts/rust-unit-tests.md). On the screen side, ["shows the error when create_node fails"](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/src/SkillNodes.test.tsx#L84-L109) fakes a failing command with a different message, so it covers what the screen does with *any* error: it shows the alert and keeps the input.

## Invariants on the way

**None of I1–I7 is enforced on this path yet.** Milestone 1 builds only the `skill_node` table, and the things I1–I7 guard (edges, evidence records, the state-change log, advisory annotations) don't exist. So there is no test named `i1_…` to `i7_…` anywhere in the code yet (design doc §12.3). Two gaps are worth knowing about, because this path will change when they close.

**The I7 gap.** I7 says every change to a node's state, *creation included*, writes exactly one `node_state_event` row in the same transaction. Creating a node gives it its first state, `available`, so this path should write one. It doesn't, because the `node_state_event` table arrives in milestone 2. The [`TODO(milestone 2): I7` comment](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L43-L46) in `create_node` marks the spot. Milestone 2's migration will backfill one creation event (`from_state` NULL, `actor` `human`) for every node made before it, so no node ends up without its history (architecture doc §2, I7 in §4).

**The `CHECK` backstop on `state`.** The architecture wants state written only from a Rust enum, so the compiler rejects a made-up state ([ADR 0002 §4](../../adr/0002-table-conventions.md#4-enums-get-a-check-constraint-as-a-backstop)). That enum, `NodeState`, arrives with the state machine in milestone 2. Until then `SkillNode.state` is a plain `String`, and `create_node` writes `'available'` as a literal in its SQL (see the [`TODO(milestone 2): ADR 0002 §4` comment](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/node.rs#L38-L41)). A typo like `'availabel'` would compile fine. What stops it is the [`CHECK` on `state`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/migrations/0001_create_skill_node.sql#L10), which allows only the four real states. Its test is [`skill_node_rejects_unknown_state`](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/src/db.rs#L121-L137).

(I4, "a node's id is never reused", is only noted in a [comment on the `id` column](https://github.com/earledotpy/waypoint/blob/d0271bf09f7868b7a2b55f6686f798892981be91/crates/waypoint-domain/migrations/0001_create_skill_node.sql#L6) for now. Every node gets a fresh UUIDv7, but there's no retire or delete yet, so nothing is there to guard.)

## Concepts used

Every concept note milestone 1 wrote, in the order this walkthrough meets them:

- [React state and effects](../concepts/react-state-and-effect.md): `useState`, and why a new list makes the screen redraw.
- [JSX](../concepts/jsx.md): the HTML-like markup in `SkillNodes`, the `onSubmit`, and the `error !== null && …` alert.
- [Promises and `async` / `await`](../concepts/promises-and-async-await.md): `await createNode(…)`, and how a rejected promise lands in `catch`.
- [Tauri commands](../concepts/tauri-command.md): `invoke`, `#[tauri::command]`, `generate_handler!`, and why the command uses `.map_err` to send its error as a string.
- [Tauri managed state](../concepts/tauri-managed-state.md): `Db`, `app.manage` and `State<'_, Db>`.
- [Ownership and borrowing (`&`, `&mut`)](../concepts/borrowing.md): what `&conn` means.
- [Borrowing in signatures (`&Connection`, `&str`)](../concepts/borrowing-in-signatures.md): why the domain's `create_node` takes `&Connection` and `&str`.
- [`Result` and `?`](../concepts/result-and-question-mark.md): `Ok` and `Err`, and the `?` after `query_row`.
- [Enums and `match`](../concepts/enums-and-match.md): `DomainError` and its variants.
- [Traits and `impl` blocks](../concepts/traits-and-impl.md): `Display`, and the `From` impl that lets `?` convert errors.
- [`struct`, `Option` and `#[derive(…)]`](../concepts/struct-and-derive.md): `SkillNode`, its `Option` fields, and `#[derive(Serialize)]`.
- [SQLite migrations](../concepts/sqlite-migrations.md): where the `skill_node` table and its `CHECK`s come from.
- [Rust unit tests](../concepts/rust-unit-tests.md): the `#[test]` functions linked above.
- [Cargo workspace](../concepts/cargo-workspace.md): why the code is split into `waypoint-domain`, `waypoint-read` and `src-tauri`.
- [Continuous integration](../concepts/continuous-integration.md): the checks that ran on every PR in this path before it merged.

**Optional further reading:** [The Rust Book, chapter 3, "Common Programming Concepts"](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html) covers variables, types, functions and control flow. It isn't needed for this walkthrough.

## Check yourself

Answer each question in your own words before opening its model answer. You can use this walkthrough, the concept notes and the [glossary](../glossary.md), but nothing else.

### 1. Follow the click all the way down and back

When you click **Create node**, your request travels from the screen, through the Rust code, into the database, and the new node travels back up to the screen. List every stop it makes along the way, in order.

**A complete answer:** for each stop, both on the way down and on the way back up, names the file *and* the function (or, for the database, the table) that handles it, and says in a few words what happens there.

<details>
<summary>Model answer</summary>

On the way down:

1. `src/SkillNodes.tsx`, `handleSubmit`: stops the page reloading, then asks for the node to be made and waits.
2. `src/api.ts`, `createNode`: sends the title and description to Rust with `invoke("create_node", …)`, written as JSON.
3. `src-tauri/src/commands.rs`, `create_node` (the Tauri command): takes the [lock](../glossary.md#lock-and-mutex) on the one database [connection](../glossary.md#connection), then calls the domain.
4. `src-tauri/src/lib.rs`, `Db` and `run`: not a step the request passes through, but where the connection the command locks comes from. `run` opened it at startup.
5. `crates/waypoint-domain/src/node.rs`, `create_node` (the domain): trims the title, refuses a blank one, makes a new id, and runs the `INSERT`.
6. The `skill_node` table, made by `crates/waypoint-domain/migrations/0001_create_skill_node.sql`: checks its rules, stores the row, and hands it back with `RETURNING`.

On the way back up:

7. `crates/waypoint-read/src/node.rs`, `SkillNode::from_row`: copies the returned row into a `SkillNode`.
8. `src-tauri/src/commands.rs`, `create_node`: passes the `SkillNode` back, and Tauri turns it into JSON.
9. `src/SkillNodes.tsx`, `handleSubmit`: the `await` finishes, `setNodes` stores a new list with the node on the end, and the screen is drawn again with one more line.

See [The path](#the-path) and [Step by step](#step-by-step).

</details>

### 2. A title that's only spaces

You type three spaces as the title and click **Create node**. Which part of the app says no, and what do you see on the screen? The database has its own rule against blank titles, so why does the app check earlier as well?

**A complete answer:** names the file and function that refuses the title, quotes the message you see, says what happens to what you typed, and says what you would see instead if only the database checked.

<details>
<summary>Model answer</summary>

The domain refuses it: `create_node` in `crates/waypoint-domain/src/node.rs`. It trims the title to nothing, sees it's empty, and returns the `EmptyTitle` error before the database is asked anything. On screen, "A node needs a title." appears above the button, and the three spaces stay in the box so you can fix them.

The domain checks first because it can give a message written for the person typing. If only the database checked, its rule (a `CHECK` [constraint](../glossary.md#constraint)) would still refuse the row, but you would see a developer's message like "database error: CHECK constraint failed: …". The database rule is a backstop that catches a future bug, such as a new way of adding nodes that forgets to check.

See [When the title is blank](#when-the-title-is-blank).

</details>

### 3. Turning a Rust error into words for the screen

When the domain refuses something, it hands back its own Rust error value, a `DomainError`. The screen can't receive that kind of value as it is. Where is it turned into plain text, and why does it have to be?

**A complete answer:** names the file and the function, quotes the piece of code that does the turning, and says what stops the `DomainError` itself from being sent.

<details>
<summary>Model answer</summary>

In `src-tauri/src/commands.rs`, in the Tauri command `create_node`, the piece `.map_err(|e| e.to_string())` does it. It leaves a success alone, and turns an error into its message, such as "A node needs a title.".

It has to, because everything sent to the screen must be turned into [JSON](../glossary.md#json) first, and `DomainError` can't be: it isn't marked with `Serialize`, the label that lets a Rust value become JSON (see [serialize](../glossary.md#serialize-and-deserialize)). Its text message can, and that message is exactly what the screen shows the user.

See step 3, [Tauri command: lock, call, translate](#3-tauri-command-lock-call-translate), and "Errors must turn into JSON too" in the [Tauri commands](../concepts/tauri-command.md) note.

</details>

### 4. Where the new line on the screen comes from

After a node is created, it appears in the list straight away, but the screen never asks for the whole list again. So where does the node it shows come from, and why can you trust that it matches what's saved in the database?

**A complete answer:** says what the database hands back after saving, which function passes it up to the screen, and why that copy is the true one.

<details>
<summary>Model answer</summary>

It comes from the database itself. The `INSERT` ends with `RETURNING`, which hands back the row exactly as it was saved. That row becomes a `SkillNode`, travels back up through the command, and `handleSubmit` adds it to the list with `setNodes([...nodes, created])`.

You can trust it because it's what was really stored, not what the Rust code meant to store. That includes the creation time, which the database chose itself.

See [5. SQLite: the row](#5-sqlite-the-row) and [6. Back up](#6-back-up-row--struct--json--screen).

</details>

### 5. The history line that isn't written yet

One of Waypoint's [invariants](../glossary.md#invariant) says that every time a node's state changes, including when the node is first created, the app also writes a line to a history table. Creating a node today doesn't write that line. Which invariant is it, why is the line missing, and what stops a misspelled state such as `'availabel'` from being saved today?

**A complete answer:** gives the invariant's number, says when the history table arrives and how the code marks the gap, and names the rule that refuses a misspelled state along with the test that proves it.

<details>
<summary>Model answer</summary>

It's I7. The history table, `node_state_event`, only arrives in milestone 2, so there's nowhere to write the line yet. A `TODO(milestone 2): I7` comment in the domain's `create_node` marks the spot, and milestone 2 will add the missing line for every node made before then.

What stops `'availabel'` today is the `CHECK` constraint on the `state` column of the `skill_node` table, which only allows the four real states. Its test is `skill_node_rejects_unknown_state`. (In milestone 2, a Rust [enum](../concepts/enums-and-match.md) will stop the misspelling even earlier, while compiling.)

See [Invariants on the way](#invariants-on-the-way).

</details>
