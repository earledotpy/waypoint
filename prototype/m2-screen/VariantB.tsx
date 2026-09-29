// THROWAWAY PROTOTYPE (issue #42). Variant B — "One sentence form".
// Three stacked sections: the node list with states, one form that reads as a
// sentence ("A is a prerequisite of B"), the list of every edge, and one log
// of every state change, newest first.
import { useState } from "react";
import type { EdgeKind, Stub } from "./stub";

export const nameB = "Sentence form, edge list, global log";

export function VariantB({ stub }: { stub: Stub }) {
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [kind, setKind] = useState<EdgeKind>("prerequisite");
  const [error, setError] = useState<string | null>(null);

  const nodeSelect = (value: string, set: (v: string) => void) => (
    <select className="border" value={value} onChange={(e) => set(e.target.value)}>
      <option value="">(pick a node)</option>
      {stub.nodes.map((n) => (
        <option key={n.id} value={n.id}>
          {n.title}
        </option>
      ))}
    </select>
  );

  return (
    <div className="space-y-6">
      <section>
        <h2>Nodes</h2>
        <ul>
          {stub.nodes.map((n) => (
            <li key={n.id}>
              {n.title} · <strong>{n.state}</strong>
            </li>
          ))}
        </ul>
      </section>

      <section className="space-y-1">
        <h2>Add an edge</h2>
        <div>
          {nodeSelect(from, setFrom)} is a{" "}
          <select className="border" value={kind} onChange={(e) => setKind(e.target.value as EdgeKind)}>
            <option value="prerequisite">prerequisite</option>
            <option value="recommendation">recommendation</option>
          </select>{" "}
          of {nodeSelect(to, setTo)}{" "}
          <button
            className="border px-2"
            onClick={() => {
              const msg = stub.addEdge(from, to, kind);
              setError(msg);
              if (msg === null) {
                setFrom("");
                setTo("");
              }
            }}
          >
            Add
          </button>
        </div>
        {error !== null && <p role="alert">{error}</p>}
      </section>

      <section>
        <h2>Edges</h2>
        {stub.edges.length === 0 ? (
          <p>No edges yet.</p>
        ) : (
          <ul>
            {stub.edges.map((e) => (
              <li key={e.id}>
                {stub.title(e.from)} is a {e.kind} of {stub.title(e.to)}{" "}
                <button className="border px-1" onClick={() => stub.removeEdge(e.id)}>
                  Remove
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section>
        <h2>State changes</h2>
        <ul>
          {[...stub.events].reverse().map((ev) => (
            <li key={ev.id}>
              {ev.at} · {stub.title(ev.nodeId)}: {ev.from ?? "(new)"} → {ev.to} ({ev.cause})
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
