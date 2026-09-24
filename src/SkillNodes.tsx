// The skill node screen: a form to create a node, and the list of nodes.
// See docs/learning/concepts/react-state-and-effect.md for `useState` and
// `useEffect`.
import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { createNode, listNodes } from "./api";
import type { SkillNode } from "./api";

function SkillNodes() {
  // Each `useState` is one value this screen remembers between renders.
  // Calling its setter stores the new value and draws the screen again.
  const [nodes, setNodes] = useState<SkillNode[]>([]);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  // The message from the last failed call, or `null` when there's nothing to
  // show.
  const [error, setError] = useState<string | null>(null);

  // Loads the list once, after the screen first appears. The empty `[]` at
  // the end means "run this on mount only".
  useEffect(() => {
    // If the screen goes away before the list arrives, `ignore` stops a late
    // answer from being stored. React's development mode mounts every
    // component twice to surface bugs like that, so this runs twice there.
    let ignore = false;
    listNodes()
      .then((loaded) => {
        if (!ignore) {
          setNodes(loaded);
        }
      })
      .catch((e) => {
        if (!ignore) {
          setError(String(e));
        }
      });
    return () => {
      ignore = true;
    };
  }, []);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    // Stops the browser's default of reloading the page on submit.
    event.preventDefault();
    try {
      const created = await createNode(title, description);
      // A new array with the node on the end. React only redraws when it's
      // handed a new value, so `nodes.push(created)` wouldn't show up.
      setNodes([...nodes, created]);
      setTitle("");
      setDescription("");
      setError(null);
    } catch (e) {
      // The Rust error arrives as a plain string, such as "A node needs a
      // title.". The inputs are left alone so the user can fix them.
      setError(String(e));
    }
  }

  return (
    <div className="space-y-6">
      {/* No `required` on the title: the domain decides what a valid title
          is, and its message is the one the user should see.
          The screen is deliberately unstyled. `border` is there only because
          Tailwind removes every border by default, which would leave the
          boxes invisible. */}
      <form onSubmit={handleSubmit} className="space-y-2">
        <div>
          <label htmlFor="node-title" className="block">
            Title
          </label>
          <input
            id="node-title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="w-full border"
          />
        </div>
        <div>
          <label htmlFor="node-description" className="block">
            Description (optional)
          </label>
          <textarea
            id="node-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full border"
          />
        </div>
        {error !== null && <p role="alert">{error}</p>}
        <button type="submit" className="border px-2">
          Create node
        </button>
      </form>

      {nodes.length === 0 ? (
        <p>No skill nodes yet.</p>
      ) : (
        <ul className="space-y-2">
          {/* `key` lets React tell the items apart when the list changes. */}
          {nodes.map((node) => (
            <li key={node.id}>
              <span>{node.title}</span> · <span>{node.state}</span> ·{" "}
              {/* The raw timestamp stays in `dateTime`, and the text shows
                  it in the user's own time zone and date format. */}
              <time dateTime={node.created_at}>
                {new Date(node.created_at).toLocaleString()}
              </time>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default SkillNodes;
