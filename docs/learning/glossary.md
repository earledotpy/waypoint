# Glossary of programming words

This page explains the general programming words used in Waypoint's docs, in plain words. It assumes you know basic Python and nothing else. Waypoint's own words (node, evidence record, Locked, Available…) are in [`CONTEXT.md`](../../CONTEXT.md), not here.

Each entry starts with a plain explanation. Some then add **In Python**, the nearest thing you may already know, and **In Waypoint**, where you'll meet it in this code. When an entry uses another word from this page, it links to it. The [Rust and TypeScript](#rust-and-typescript) constructs that have their own concept note are listed [at the end](#words-with-their-own-concept-note).

**Adding a word.** When a PR uses a general programming word that isn't here, it adds an entry in the same PR, in alphabetical order. Follow the format above, and explain the word using only plain words or links to other entries on this page.

## A–C

### API

The list of requests one piece of software accepts from other code, and how to make them. It's like a restaurant menu: you order from the menu, and you don't need to know how the kitchen works.

**In Waypoint:** `src/api.ts` is the screen's menu of requests it can send to the Rust half of the app, such as `createNode`.

### Argument and parameter

A parameter is a named slot in a [function](#function)'s definition. An argument is the actual value you put in that slot when you call the function.

**In Python:** in `def greet(name):`, `name` is a parameter. In `greet("Ada")`, `"Ada"` is the argument.

### Async

Short for *asynchronous*. Async code starts a slow job, such as asking the [database](#database) something, and lets the rest of the program carry on instead of freezing while it waits. When the answer is ready, the code picks up where it left off. The value you wait for is usually a [promise](#promise).

**In Python:** `async def` and `await`, the same words.

**In Waypoint:** the screen's `handleSubmit` is async, so the window stays responsive while Rust creates the node. See the [Promises and `async` / `await`](concepts/promises-and-async-await.md) note.

### Build

Turning the files people write into the program that actually runs, and checking them on the way. For Rust, most of building is [compiling](#compile-and-compiler). For the screen, the TypeScript files are packed together into files the app's window can load.

### Class and object

A class is a blueprint for a kind of thing. An object is one thing made from it, carrying its own data ([fields](#field)) and its own [methods](#method).

**In Python:** `class Dog:` is the class, and `rex = Dog()` is an object.

**In Waypoint:** Rust has no classes. It gets the same effect from a `struct` for the data and an `impl` block for the methods. See the [`struct`](concepts/struct-and-derive.md) and [traits and `impl`](concepts/traits-and-impl.md) notes.

### Closure

A small [function](#function) written in place, without a name, usually handed straight to another function.

**In Python:** `lambda e: str(e)`.

**In Waypoint:** `.map_err(|e| e.to_string())` in a Tauri command. The `|e| e.to_string()` part is a closure: "given `e`, turn it into text".

### Column

See [Table, row and column](#table-row-and-column).

### Commit

A saved snapshot of all the project's files, recorded by git (the tool that keeps the project's history), with a message saying what changed. Each commit has an id, a long string of letters and digits (its *SHA*), so it can be pointed at forever.

### Compile and compiler

A compiler reads a whole program and translates it into instructions the computer can run directly, checking it as it goes. If a check fails, you get an error message and no program. *Compile time* means "while compiling, before the program ever runs". *Run time* means "while the program is running".

**In Python:** Python skips this step. It reads and runs your file as it goes, so a mistake usually only shows up when that line runs. Rust and TypeScript catch many mistakes at compile time instead, before anyone uses the app.

### Component

In [React](#react), one piece of the screen, written as a [function](#function). It takes some inputs and returns a description of what that piece should look like.

**In Waypoint:** `SkillNodes` in `src/SkillNodes.tsx` is the component for the whole screen.

### Connection

An open line to a [database](#database). You send [SQL](#sql) down it and get answers back. Opening one takes a little work, so a program usually opens one and reuses it.

**In Python:** `sqlite3.connect("waypoint.db")` returns a connection.

**In Waypoint:** the app opens one connection when it starts and shares it between all commands, behind a [lock](#lock-and-mutex).

### Constraint

A rule a [database](#database) table checks on every [row](#table-row-and-column), refusing any row that breaks it.

**In Waypoint:** `CHECK (length(trim(title)) > 0)` on the `skill_node` table refuses a blank title, even if the Rust code forgot to check.

### Crash and panic

A crash is a program stopping suddenly because something went wrong that it wasn't written to handle. In Rust, stopping on purpose like this is called a *panic*.

**In Python:** the nearest thing is an [exception](#error-and-exception) that nothing catches, so the program stops with a traceback.

**In Waypoint:** `.expect(…)` panics, with the message it's given, if the value before the dot is an error. The Tauri commands use it when taking the database lock, where carrying on would be worse than stopping.

### Crate

See [Package and crate](#package-and-crate).

## D–I

### Database

A program that stores data on disk, organised into [tables](#table-row-and-column), and answers questions about it.

**In Waypoint:** the database is [SQLite](#sqlite), one file on your computer.

### Dependency

A [library](#library) written by someone else that your project uses. A list file says which ones, and which versions.

**In Python:** what you `pip install` and list in `requirements.txt`.

**In Waypoint:** `Cargo.toml` lists the Rust dependencies, and `package.json` the TypeScript ones.

### Deserialize

See [Serialize and deserialize](#serialize-and-deserialize).

### Domain

The part of a program that holds the rules about its subject, here skills, evidence and reviews, rather than the screen or the storage.

**In Waypoint:** the `waypoint-domain` [crate](#package-and-crate). Every rule, and every write to the database, lives there.

### Error and exception

An error is a sign that something went wrong. Languages pass errors around in two different ways:

- An *exception* jumps out of the [function](#function), and out of the ones that called it, until some code catches it.
- An *error value* is handed back like any other [return value](#return-value), and the caller has to look at it.

**In Python:** exceptions. `raise ValueError(...)` throws one, and `try` / `except` catches it.

**In Waypoint:** Rust has no exceptions. A function that can fail returns a `Result`, which is either `Ok(value)` or `Err(error)`. See the [`Result` and `?`](concepts/result-and-question-mark.md) note. TypeScript has exceptions like Python's, using `throw` and `try` / `catch`.

### Field

One named piece of data inside a bigger value.

**In Python:** the attributes of an object: `node.title`.

**In Waypoint:** a `SkillNode` has the fields `id`, `title`, `description`, `state`, `created_at` and more.

### Frontend and backend

The frontend is the part of an app people see and click. The backend is the part behind it that does the work, such as storing data and applying rules.

**In Waypoint:** the frontend is the [React](#react) screen, written in TypeScript. The backend is the Rust code: the [Tauri](#tauri) commands, the [domain](#domain) and [SQLite](#sqlite). Unlike a website, both run inside one program on your own computer.

### Function

A named block of code you can run again and again. It takes [arguments](#argument-and-parameter) and hands back a [return value](#return-value).

**In Python:** `def`. In Rust it's `fn`. In TypeScript it's `function`, or the short form `(x) => …`.

### Function signature

A [function](#function)'s first line: its name, the [parameters](#argument-and-parameter) it takes and the [type](#type) of each, and the type of value it gives back. You can tell how to use a function from its signature alone.

**In Python:** with type hints, `def create_node(title: str) -> SkillNode:`.

**In Waypoint:** `fn create_node(conn: &Connection, title: &str, description: &str) -> Result<SkillNode, DomainError>`. The part after `->` is what it gives back.

### HTML

The language web pages are written in. It's text with tags such as `<p>` (a paragraph) and `<button>` that say what each part of the page is.

**In Waypoint:** the screen is a web page shown inside the app's window. [React](#react) writes its HTML with JSX: see the [JSX](concepts/jsx.md) note.

### Invariant

A rule about the data that must always be true, whatever happens. If an invariant is ever broken, the data is wrong.

**In Waypoint:** there are seven, I1–I7, listed in `docs/architecture-schema.md`. They're all enforced in the [domain](#domain), and each has a [test](#test-unit-test-and-assertion) named after it. For example, I5 says prerequisites never form a loop.

### IPC

Short for *inter-process communication*: two programs, or two halves of one, passing messages to each other.

**In Waypoint:** the screen asks Rust to do something by calling [Tauri](#tauri)'s `invoke`, and the message travels as [JSON](#json). See the [Tauri commands](concepts/tauri-command.md) note.

## J–P

### JSON

A plain-text format for data that almost every language can read and write:

```json
{ "title": "Read a Rust compiler error", "state": "available" }
```

**In Python:** the `json` module. A JSON object looks just like a dict.

**In Waypoint:** the screen and Rust send everything to each other as JSON, because it's the format both sides understand.

### Library

Code someone else wrote and packaged up for others to use.

**In Python:** anything you `import` that didn't come from your own files, such as `requests`.

### Literal

A value typed straight into the code, rather than worked out while the program runs: `42`, `"Create node"`, `'available'`.

### Lock and mutex

A lock makes sure only one part of a program uses something at a time. The others wait their turn. A *mutex* (short for "mutual exclusion") is a value wrapped in a lock: to get at the value, you must take the lock first.

**In Python:** `threading.Lock()`.

**In Waypoint:** the database [connection](#connection) is wrapped in a mutex, so two commands can never use it at the same moment. See the [Tauri managed state](concepts/tauri-managed-state.md) note.

### Macro

Code that writes code. Before [compiling](#compile-and-compiler), a macro expands into more code, saving you from typing it out.

**In Python:** there's nothing quite like it. Decorators (`@something` above a `def`) are the nearest cousin.

**In Waypoint:** in Rust a name ending in `!` is a macro, such as `generate_handler![…]`. `#[derive(…)]` is a related kind: see the [`struct` and `#[derive(…)]`](concepts/struct-and-derive.md) note.

### Markdown

A simple way to format plain text: `#` starts a heading, `*word*` is italic, `-` starts a bullet point. Every doc in this repo, this one included, is Markdown, and GitHub shows it formatted.

### Method

A [function](#function) that belongs to a [type](#type), called with a dot after a value.

**In Python:** `"  hi  ".strip()`.

**In Waypoint:** `title.trim()` in the domain's `create_node` removes spaces from both ends of the title.

### Migration

A numbered script that changes the [database](#database)'s shape, for example by creating a [table](#table-row-and-column). Migrations run once each, in number order, so every copy of the database ends up with the same shape.

**In Waypoint:** `crates/waypoint-domain/migrations/0001_create_skill_node.sql`. See the [SQLite migrations](concepts/sqlite-migrations.md) note.

### Module

A file, or folder, of code with its own name, that other code can use things from.

**In Python:** every `.py` file is a module.

**In Waypoint:** `crates/waypoint-domain/src/node.rs` is the `node` module of the `waypoint-domain` [crate](#package-and-crate).

### Mutex

See [Lock and mutex](#lock-and-mutex).

### Null and None

A special value that means "nothing here".

**In Python:** `None`. [SQL](#sql), [JSON](#json) and TypeScript call it `null`.

**In Waypoint:** Rust has no null. A value that might be missing has the type `Option`, which is either `Some(value)` or `None`, and Rust makes you deal with both. See the [`struct` and `Option`](concepts/struct-and-derive.md) note.

### Package and crate

A package is a bundle of code with a name and a version, which other projects can use as a [dependency](#dependency). In Rust, a package's code is called a *crate*.

**In Waypoint:** the Rust code is split into crates such as `waypoint-domain` and `waypoint-read`. See the [Cargo workspace](concepts/cargo-workspace.md) note.

### Panic

See [Crash and panic](#crash-and-panic).

### Permalink

A link to a file as it was at one exact [commit](#commit). It keeps showing the same lines even after the code changes. Walkthroughs use permalinks so their links stay right.

### Primary key

The [column](#table-row-and-column) whose value picks out exactly one [row](#table-row-and-column) of a [table](#table-row-and-column). No two rows may share it, and it's never empty. Other tables point at a row by storing its primary key; that pointer is a *foreign key* (see the [Foreign keys](concepts/foreign-keys.md) note).

**In Python:** the key of a dictionary: `nodes[id]` finds exactly one node.

**In Waypoint:** every table's primary key is a column called `id` holding a [UUID](#uuid).

### Promise

An IOU for a value that isn't ready yet. A slow [async](#async) function hands back a promise straight away. Later the promise either *resolves* (here's the value) or *rejects* (here's the [error](#error-and-exception)). `await` waits for that to happen.

**In Python:** the nearest thing is what calling an `async def` function gives you: something you have to `await` to get the value out.

**In Waypoint:** `createNode` in `src/api.ts` returns a promise of the new node. See the [Promises and `async` / `await`](concepts/promises-and-async-await.md) note.

## Q–Z

### Query

A question sent to a [database](#database) in [SQL](#sql), such as `SELECT title FROM skill_node`. People also use it loosely for any SQL statement, including ones that change data.

### React

The [library](#library) that draws Waypoint's screen. You describe what the screen should look like for the current data, using [components](#component), and React redraws it whenever that data changes. See the [React state and effects](concepts/react-state-and-effect.md) note.

### Render

To work out what the screen should look like and draw it. [React](#react) renders a [component](#component) again whenever its [state](#state) changes.

### Return value

What a [function](#function) hands back to the code that called it, when it finishes.

**In Python:** the value after `return`. In Rust, the last line of a function, written without a `;`, is handed back, so you'll often see no `return` at all.

### Row

See [Table, row and column](#table-row-and-column).

### Rust and TypeScript

The two programming languages Waypoint is written in, other than [SQL](#sql).

- **Rust** is used for everything behind the screen: the rules, and saving and reading data. A [compiler](#compile-and-compiler) checks it strictly before it runs, which catches many mistakes early.
- **TypeScript** is used for the screen. It is JavaScript, the language every web browser runs, with [types](#type) added so mistakes are caught before it runs.

**In Python:** both do the same jobs Python can do. The biggest difference you'll notice is that they check much more before the program runs.

### Schema

The shape of a [database](#database): which [tables](#table-row-and-column) it has, their columns, and their [constraints](#constraint).

**In Waypoint:** described in `docs/architecture-schema.md`, and built by [migrations](#migration).

### Serialize and deserialize

To serialize is to turn a value the program is holding into text, so it can be saved or sent somewhere else. To deserialize is to turn that text back into a value.

**In Python:** `json.dumps(data)` serializes and `json.loads(text)` deserializes.

**In Waypoint:** a `SkillNode` is serialized into [JSON](#json) to send it to the screen. `#[derive(Serialize)]` is what lets it be: see [ADR 0003](../adr/0003-serde-and-derive.md).

### SQL

The language for talking to a [database](#database). `SELECT` reads [rows](#table-row-and-column), `INSERT` adds them, `UPDATE` changes them and `DELETE` removes them.

### SQLite

A small [database](#database) that lives in a single file, with no separate program to install or run. Python has it built in, as the `sqlite3` module.

### State

Two different meanings in Waypoint's docs, and the sentence around the word tells you which:

1. **In code generally:** the values a program is remembering right now. *React state* is a value a [component](#component) remembers between [renders](#render), such as the text typed so far.
2. **In Waypoint's own words:** a node's state is where it stands: Locked, Available, In progress or Evidenced. That meaning is defined in [`CONTEXT.md`](../../CONTEXT.md).

### String

A piece of text.

**In Python:** a `str`, like `"hello"`.

**In Waypoint:** Rust has two kinds of text, `String` and `&str`. See the [borrowing in signatures](concepts/borrowing-in-signatures.md) note for the difference.

### Table, row and column

A [database](#database) table holds one kind of thing, laid out like a spreadsheet. Each row is one thing, and each column is one piece of information about every thing.

**In Waypoint:** in the `skill_node` table, each row is one node, and the columns include `title`, `state` and `created_at`.

### Tauri

The [library](#library) that turns Waypoint's web-page screen and its Rust code into one desktop app, and carries messages between the two halves. See the [Tauri commands](concepts/tauri-command.md) note.

### Test, unit test and assertion

A test is code that runs other code and checks the result. A unit test checks one small piece, usually one [function](#function), on its own. An assertion is the check itself: "this must equal that, or the test fails".

**In Python:** pytest, and the `assert` statement.

**In Waypoint:** both the Rust code and the screen have tests, and they all run on every PR. See the [Rust unit tests](concepts/rust-unit-tests.md) note.

### Thread

One line of work a program is doing. A program can run several threads at the same time. That's faster, but two threads might try to change the same thing at once, and a [lock](#lock-and-mutex) is what stops that.

**In Python:** the `threading` module.

### Timestamp

A recorded date and time.

**In Waypoint:** timestamps are stored as text like `2026-10-05T14:03:12.345Z`: year-month-day, `T`, then the time to the thousandth of a second. The `Z` means it's in UTC, the world's reference time zone, so the clock doesn't depend on where you are. This layout is called *ISO 8601*. See [ADR 0002](../adr/0002-table-conventions.md).

### Transaction

A group of [database](#database) changes that all happen, or none do. Think of a bank transfer: the money leaves one account and arrives in the other, never only half of that. Saving the whole group is called *committing* it (a different thing from a git [commit](#commit)), and undoing all of it is *rolling back*.

**In Python:** with `sqlite3`, `with conn:` commits the changes if the block finishes, and rolls them back if it raises an [exception](#error-and-exception).

### Type

What kind of value something is: a whole number, a [string](#string), a list, a `SkillNode`. The type decides what you can do with the value. A *type checker* is a tool that reads the code and makes sure every value is only used in ways its type allows.

**In Python:** `type(3)` is `int`. Python checks types only as the code runs, and type hints like `title: str` are optional notes, unless you run a separate checker such as mypy.

**In Waypoint:** Rust and TypeScript check every type before the program runs, while [compiling](#compile-and-compiler). A wrong type is an error you see straight away, not a crash later.

### UUID

Short for *universally unique identifier*: a long id, like `0192f4a8-7c3e-7b21-9d4f-3a6e5c1b2d09`, made so that no two are ever the same. Anyone can make one without asking a central counter for the next number.

**In Waypoint:** every node's id is a UUIDv7. Its first part is the time it was made, so ids sort in the order they were created. See [ADR 0002](../adr/0002-table-conventions.md).

### Variable

A name that holds a value.

**In Python:** `x = 3`. In Rust it's `let x = 3;`, and the value can't be changed later unless you write `let mut x = 3;`. In TypeScript it's `const x = 3;`, or `let` for one that changes.

## Words with their own concept note

These are Rust or TypeScript constructs rather than general programming words. Each has a concept note that explains it as it appears in this code.

| Word | Concept note |
|---|---|
| Borrowing, ownership, `&`, `&mut` | [Ownership and borrowing](concepts/borrowing.md) |
| `&str`, `&Connection` in a signature | [Borrowing in signatures](concepts/borrowing-in-signatures.md) |
| Cargo, workspace | [Cargo workspace](concepts/cargo-workspace.md) |
| CI, continuous integration | [Continuous integration](concepts/continuous-integration.md) |
| `enum`, variant, `match` | [Enums and `match`](concepts/enums-and-match.md) |
| Foreign key, `REFERENCES`, `PRAGMA foreign_keys` | [Foreign keys](concepts/foreign-keys.md) |
| JSX | [JSX](concepts/jsx.md) |
| `async`, `await`, promise | [Promises and `async` / `await`](concepts/promises-and-async-await.md) |
| `useState`, `useEffect` | [React state and effects](concepts/react-state-and-effect.md) |
| `Result`, `Ok`, `Err`, `?` | [`Result` and `?`](concepts/result-and-question-mark.md) |
| `#[test]` | [Rust unit tests](concepts/rust-unit-tests.md) |
| Migration | [SQLite migrations](concepts/sqlite-migrations.md) |
| `struct`, `Option`, `#[derive(…)]` | [`struct`, `Option` and `#[derive(…)]`](concepts/struct-and-derive.md) |
| Tauri command, `invoke` | [Tauri commands](concepts/tauri-command.md) |
| `State<'_, …>`, `app.manage` | [Tauri managed state](concepts/tauri-managed-state.md) |
| Trait, `impl`, `Display`, `From` | [Traits and `impl` blocks](concepts/traits-and-impl.md) |
