# JSX

## In one line

JSX is the HTML-like syntax inside a `.tsx` file. Each tag is really a function call that builds a description of the screen, and `{…}` drops a TypeScript expression into it.

## Where it appears here

- `src/SkillNodes.tsx`, the `return (…)` at the end of `SkillNodes`: the form, the error message and the list.
- `src/App.tsx`, `App`: `<SkillNodes />` inside the page layout.
- `src/SkillNodes.test.tsx`: `render(<SkillNodes />)` builds the screen for a test.

## What it does

A browser doesn't understand JSX. Before the code runs, Vite rewrites every tag into a function call. (`"jsx": "react-jsx"` in `tsconfig.json` tells `tsc` to expect that same rewrite when it type-checks.) So

```tsx
<p role="alert">{error}</p>
```

becomes, roughly,

```ts
jsx("p", { role: "alert", children: error })
```

That call returns a plain object that says "a `p` with these attributes and this content". A component returns a tree of these objects, and React turns the tree into real page elements. When state changes, it builds a new tree and updates only what differs. So JSX is ordinary code, and `tsc` checks it: a misspelt attribute or a wrong type is a compile error, not a broken page. A file must end in `.tsx`, not `.ts`, for JSX to be allowed.

### The pieces in `SkillNodes.tsx`

- **`{…}` holds one expression.** `{error}`, `{node.title}` and `value={title}` put a value in place. Text is escaped automatically, so a title containing `<b>` shows the characters, not bold text. Only expressions are allowed, not statements, so there's no `if` or `for` inside JSX. The next three items are how you get round that.
- **`cond && <p …>`** shows the `<p>` only when `cond` is true. `&&` gives back its left side if that's falsy, and its right side otherwise. React draws nothing for `false`, `null` or `undefined`. So `{error !== null && <p role="alert">{error}</p>}` is the alert when there's an error, and nothing otherwise. One trap: a falsy *number* is drawn, so `{count && …}` shows `0` when `count` is 0. Comparing first, as `error !== null` does, avoids that.
- **`cond ? a : b`** is JavaScript's inline `if`/`else` (Python's `a if cond else b`). It picks "No skill nodes yet." or the list.
- **`.map(…)`** turns each node into an `<li>`, like a Python list comprehension. Each item needs a `key` that stays the same for the same node, so React can tell items apart when the list changes.
- **`className` and `htmlFor`** stand in for HTML's `class` and `for`. JSX attributes become JavaScript object keys, and `class` and `for` are reserved words in JavaScript.
- **`onChange={(e) => setTitle(e.target.value)}`** passes a function for React to call on each keystroke. Writing `onChange={setTitle(…)}` would call it once, during the render.
- **`<SkillNodes />`** is a tag for your own component. A capital first letter tells JSX it's a component to call, not an HTML element.
- **`{/* … */}`** is a comment inside JSX: a JavaScript comment wrapped in `{}`.

## Python comparison

The nearest Python idea is a Jinja template, as in Flask:

```jinja
{% if error %}<p role="alert">{{ error }}</p>{% endif %}
<ul>{% for node in nodes %}<li>{{ node.title }}</li>{% endfor %}</ul>
```

Both mix markup with values, and both escape text by default.

Where it breaks:

- **It's code, not text.** A Jinja template is a string that's filled in and sent as HTML. JSX compiles to function calls that produce objects, and the type checker reads it like any other code.
- **No template statements.** Jinja has `{% if %}` and `{% for %}`. JSX has only expressions, so it uses `&&`, `? :` and `.map`. Python's `and` works exactly like JavaScript's `&&` here: `x and y` gives back `x` if it's falsy, and `y` otherwise.
- **It runs again on every change.** A template is rendered once per request. The JSX in a component is rebuilt on every render, and React updates only the parts that changed.

## Why this code uses it

It's how React components are normally written, and the Tauri `react-ts` template starts with it (ADR 0001 §5). The alternative is calling `jsx(…)` or `React.createElement(…)` by hand, which is legal and does the same thing, but a nested form becomes very hard to read.

## See also

- [React state and effects](react-state-and-effect.md): what makes a component render again.
- React documentation, [Writing Markup with JSX](https://react.dev/learn/writing-markup-with-jsx) and [JavaScript in JSX with Curly Braces](https://react.dev/learn/javascript-in-jsx-with-curly-braces)
