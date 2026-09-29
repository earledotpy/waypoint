// THROWAWAY PROTOTYPE (issue #42). Variant A — "Edges on each node".
// Every node row carries its own edges and its own small add form. The form
// lives on the node that gets gated (the dependent). No state history.
import { useState } from "react";
import type { EdgeKind, Stub, StubNode } from "./stub";

export const nameA = "Edges on each node, no history";

function NodeRow({ stub, node }: { stub: Stub; node: StubNode }) {
  const [from, setFrom] = useState("");
  const [kind, setKind] = useState<EdgeKind>("prerequisite");
  const [error, setError] = useState<string | null>(null);
  const incoming = stub.edges.filter((e) => e.to === node.id);
  const prereqs = incoming.filter((e) => e.kind === "prerequisite");
  const recs = incoming.filter((e) => e.kind === "recommendation");

  return (
    <li className="border p-2 space-y-1">
      <div>
        <strong>{node.title}</strong> · {node.state}
      </div>
      <div>
        Prerequisites:{" "}
        {prereqs.length === 0
          ? "none"
          : prereqs.map((e) => (
              <span key={e.id} className="mr-2">
                {stub.title(e.from)}{" "}
                <button className="border px-1" onClick={() => stub.removeEdge(e.id)}>
                  remove
                </button>
              </span>
            ))}
      </div>
      <div>
        Recommended first:{" "}
        {recs.length === 0
          ? "none"
          : recs.map((e) => (
              <span key={e.id} className="mr-2">
                {stub.title(e.from)}{" "}
                <button className="border px-1" onClick={() => stub.removeEdge(e.id)}>
                  remove
                </button>
              </span>
            ))}
      </div>
      <div>
        Add{" "}
        <select className="border" value={kind} onChange={(e) => setKind(e.target.value as EdgeKind)}>
          <option value="prerequisite">prerequisite</option>
          <option value="recommendation">recommendation</option>
        </select>{" "}
        <select className="border" value={from} onChange={(e) => setFrom(e.target.value)}>
          <option value="">(pick a node)</option>
          {stub.nodes.map((n) => (
            <option key={n.id} value={n.id}>
              {n.title}
            </option>
          ))}
        </select>{" "}
        <button
          className="border px-2"
          onClick={() => {
            const msg = stub.addEdge(from, node.id, kind);
            setError(msg);
            if (msg === null) setFrom("");
          }}
        >
          Add
        </button>
      </div>
      {error !== null && <p role="alert">{error}</p>}
    </li>
  );
}

export function VariantA({ stub }: { stub: Stub }) {
  return (
    <ul className="space-y-2">
      {stub.nodes.map((n) => (
        <NodeRow key={n.id} stub={stub} node={n} />
      ))}
    </ul>
  );
}
