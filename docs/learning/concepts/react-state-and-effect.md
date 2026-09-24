# React state and effects

## In one line

`useState` gives a component a value that it remembers between renders and that redraws the screen when it changes. `useEffect` runs code after the screen has been drawn, such as loading data.

## Where it appears here

- `src/SkillNodes.tsx`, `SkillNodes`: four `useState` values (`nodes`, `title`, `description`, `error`) and one `useEffect` that loads the list with `listNodes()` when the screen first appears.
- `src/SkillNodes.test.tsx`: the tests wait with `findBy…` for the effect's list to arrive.

## What it does

A React component is a function that returns what the screen should look like. React calls it again whenever something changes. That's a *render*. Local variables start fresh on every call, so a component can't remember anything in them.

### `useState`

```tsx
const [title, setTitle] = useState("");
```

`useState("")` asks React to keep one value for this component, starting at `""`. It returns the current value and a function to change it. Calling `setTitle("Count to ten")` stores the new value and tells React to render again, and on that render `title` is `"Count to ten"`.

The form's inputs work this way. Each keystroke calls `setTitle(e.target.value)`, and the `<input value={title}>` shows whatever `title` now is. So after a successful create, `setTitle("")` is all it takes to clear the box. React calls this a *controlled input*: the state is the source of truth, not the input box.

React only redraws when it gets a **new** value. `nodes.push(created)` would change the old array in place, and React wouldn't notice. So `handleSubmit` builds a new array, `[...nodes, created]` (like Python's `[*nodes, created]`), and passes that to `setNodes`.

### `useEffect`

```tsx
useEffect(() => {
  let ignore = false;
  listNodes().then((loaded) => {
    if (!ignore) setNodes(loaded);
  });
  return () => {
    ignore = true;
  };
}, []);
```

(Trimmed: the real code also has a `.catch` that shows a failed load in the error message.)

The function inside runs *after* React has put the render on screen, so the page appears at once, showing "No skill nodes yet.", and the list fills in when Rust answers. Loading the list can't happen inside the component function itself, because that runs on every render and would ask Rust again after every keystroke.

The `[]` at the end is the *dependency list*: the effect runs again only when a value in it changes. An empty list means "only when the component first appears" (*mounts*).

The function the effect returns is its *cleanup*. React calls it when the component goes away. Here it sets `ignore`, so an answer that arrives too late isn't stored. In development, `<React.StrictMode>` (in `main.tsx`) mounts every component, removes it and mounts it again, on purpose, to show up effects that forget to clean up. So in `npm run tauri dev`, `list_nodes` is called twice and the first answer is ignored. A built app calls it once.

## Python comparison

Picture a variable that redraws the page whenever you assign to it, plus a function that runs once right after the page first appears:

```python
class SkillNodes(Widget):
    nodes = reactive([])            # assigning to self.nodes redraws the screen

    def on_mount(self):             # runs after the first draw
        self.nodes = api.list_nodes()
```

That's close to how Textual (a Python terminal UI library) works, and `on_mount` is the nearest thing to `useEffect(…, [])`.

Where it breaks:

- **The whole function runs again.** A Python object keeps its attributes between redraws. A React component is a plain function that React calls on every render, and only values kept with `useState` survive.
- **Setting doesn't change the variable now.** After `setTitle("")`, `title` still holds the old text until the next render. The new value shows up only when React calls the component again.
- **Changing in place is invisible.** In Python, `self.nodes.append(x)` is the natural way to add to a list. React checks whether it was handed a *different* value, and a list changed in place is still the same list, so nothing redraws. Always pass a new value.

## Why this code uses it

Issue #19 asks for plain React: no router and no state library. `useState` and `useEffect` are the two hooks that ship with React and cover this screen: remember the list and the form, and load the list once. A state library such as Redux or Zustand earns its place when many screens share the same data, and Waypoint has one screen so far.

## See also

- [Tauri commands](tauri-command.md): what `listNodes` and `createNode` do on the Rust side.
- [Promises and `async` / `await`](promises-and-async-await.md): the `.then` in the effect, and why the effect isn't `async`.
- [JSX](jsx.md): the markup the component returns.
- React documentation, [State: A Component's Memory](https://react.dev/learn/state-a-components-memory) and [Synchronizing with Effects](https://react.dev/learn/synchronizing-with-effects)
