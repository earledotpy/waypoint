# Promises and `async` / `await`

## In one line

A `Promise` is a value that stands for an answer that hasn't arrived yet. You wait for it either with `await` inside an `async` function, or by handing it callbacks with `.then(…)` and `.catch(…)`.

## Where it appears here

- `src/api.ts`, `createNode` and `listNodes`: each returns the `Promise` that `invoke` gives back.
- `src/SkillNodes.tsx`, `handleSubmit`: an `async` function that `await`s `createNode` inside `try` / `catch`.
- `src/SkillNodes.tsx`, the `useEffect`: waits for `listNodes()` with `.then(…)` and `.catch(…)` instead.

## What it does

Asking Rust for something takes time, and the page mustn't freeze while it waits. So `invoke` doesn't return the node. It returns straight away with a `Promise`, and the Rust work carries on in the background. A promise ends in one of two ways:

- it **resolves** with a value (the Rust command returned `Ok(node)`), or
- it **rejects** with an error (the command returned `Err(message)`, see [Tauri commands](tauri-command.md)).

`createNode` and `listNodes` aren't `async` themselves. They just pass `invoke`'s promise back to the caller. The type `Promise<SkillNode>` says "a promise that resolves to a `SkillNode`". TypeScript takes that on trust: nothing checks at runtime that Rust really sent a `SkillNode`.

### Two ways to wait

**`await`**, in `handleSubmit`:

```tsx
async function handleSubmit(event) {
  try {
    const created = await createNode(title, description);
    setNodes([...nodes, created]);
  } catch (e) {
    setError(String(e));
  }
}
```

`await` pauses this function until the promise settles, then hands back its value, so the code reads top to bottom. If the promise rejects, the `await` line throws, and `catch` receives the rejection. Here that's the plain string Rust sent. TypeScript types `e` as `unknown`, because anything can be thrown, so `String(e)` turns it into text either way. `await` is only allowed inside a function marked `async`, and an `async` function always returns a promise of its own.

**`.then` / `.catch`**, in the `useEffect`:

```tsx
listNodes()
  .then((loaded) => { if (!ignore) setNodes(loaded); })
  .catch((e) => { if (!ignore) setError(String(e)); });
```

`.then(f)` means "when it resolves, call `f` with the value". `.catch(g)` means "if it rejects, call `g` with the error". The line itself doesn't wait. The code after it runs at once, and `f` or `g` runs later. `(loaded) => { … }` is an *arrow function*, a function with no name written in place, like Python's `lambda` but able to hold several statements.

### Why the effect doesn't use `await`

The function given to `useEffect` may return only one thing: a cleanup function (see [React state and effects](react-state-and-effect.md)). An `async` function always returns a promise, so `useEffect(async () => …)` would hand React a promise where it expects a cleanup, and React warns about it. `.then` keeps the effect an ordinary function that returns its cleanup. `handleSubmit` has no such limit: React ignores whatever an event handler returns, so it can be `async`.

A promise that rejects with no `.catch` and no `try` around its `await` is an *unhandled rejection*. The error only goes to the console, and the user sees nothing. That's why both places catch.

## Python comparison

`async def` and `await` look the same in Python, and `try` / `except` around an `await` works just like here. The nearest thing to a `Promise` is an `asyncio.Task`, and `.then(f)` is close to `task.add_done_callback(f)`.

Where it breaks:

- **Work starts immediately.** Calling a Python `async def` function only creates a coroutine, and nothing runs until something awaits it or schedules it. Calling `createNode(…)` starts the Rust call at once, whether or not anyone ever awaits the promise.
- **The event loop is always there.** Python needs `asyncio.run(…)` to start a loop. The webview runs one all the time, so any function can create a promise, and only `await` needs an `async` function.
- **Anything can be thrown.** Python only raises exceptions. JavaScript can throw or reject with any value, and Tauri rejects with a bare string. `except Exception as e` has no equivalent, so `catch (e)` gets `unknown` and the code has to deal with it.

## Why this code uses it

`invoke` returns a promise, so every call to Rust is asynchronous whether we like it or not. `handleSubmit` uses `await` because straight-line code is easiest to read. The effect uses `.then` because React doesn't allow an `async` effect function. The other common pattern, an `async` function defined inside the effect and then called, works too, but it adds one more function to follow.

## See also

- [Tauri commands](tauri-command.md): how a Rust `Result` becomes a resolved or rejected promise.
- [React state and effects](react-state-and-effect.md): the effect and its cleanup.
- MDN, [Using promises](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Using_promises)
