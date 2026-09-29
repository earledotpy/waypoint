// THROWAWAY PROTOTYPE (issue #42). Variant C — "Pick a node".
// A list of nodes on the left; clicking one opens its detail on the right:
// what it needs, what it unlocks, an add form scoped to it, and its own
// state history.
import { useState } from "react";
import type { EdgeKind, Stub, StubEdge } from "./stub";

export const nameC = "Pick a node, detail panel, per-node history";

export function VariantC({ stub }: { stub: Stub }) {
  const [selected, setSelected] = useState(stub.nodes[0]?.id ?? "");
  const [other, setOther] = useState("");
  const [kind, setKind] = useState<EdgeKind>("prerequisite");
  const [error, setError] = useState<string | null>(null);
  const node = stub.nodes.find((n) => n.id === selected);

  const edgeList = (label: string, list: StubEdge[], show: (e: StubEdge) => string) => (
    <div>
      {label}:{" "}
      {list.length === 0 ? (
        "none"
      ) : (
        <ul className="ml-4">
          {list.map((e) => (
            <li key={e.id}>
              {show(e)}{" "}
              <button className="border px-1" onClick={() => stub.removeEdge(e.id)}>
                remove
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );

  return (
    <div className="flex gap-6">
      <ul className="w-1/3 border p-2">
        {stub.nodes.map((n) => (
          <li key={n.id}>
            <button
              className={n.id === selected ? "underline" : ""}
              onClick={() => {
                setSelected(n.id);
                setError(null);
              }}
            >
              {n.title}
            </button>{" "}
            · {n.state}
          </li>
        ))}
      </ul>

      {node && (
        <div className="w-2/3 space-y-3">
          <h2>
            {node.title} · <strong>{node.state}</strong>
          </h2>
          {edgeList(
            "Needs first (prerequisites)",
            stub.edges.filter((e) => e.to === node.id && e.kind === "prerequisite"),
            (e) => stub.title(e.from),
          )}
          {edgeList(
            "Recommended first",
            stub.edges.filter((e) => e.to === node.id && e.kind === "recommendation"),
            (e) => stub.title(e.from),
          )}
          {edgeList(
            "Unlocks (it is a prerequisite of)",
            stub.edges.filter((e) => e.from === node.id && e.kind === "prerequisite"),
            (e) => stub.title(e.to),
          )}
          {edgeList(
            "Recommended before",
            stub.edges.filter((e) => e.from === node.id && e.kind === "recommendation"),
            (e) => stub.title(e.to),
          )}

          <div>
            Add a{" "}
            <select className="border" value={kind} onChange={(e) => setKind(e.target.value as EdgeKind)}>
              <option value="prerequisite">prerequisite</option>
              <option value="recommendation">recommendation</option>
            </select>{" "}
            for this node:{" "}
            <select className="border" value={other} onChange={(e) => setOther(e.target.value)}>
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
                const msg = stub.addEdge(other, node.id, kind);
                setError(msg);
                if (msg === null) setOther("");
              }}
            >
              Add
            </button>
            {error !== null && <p role="alert">{error}</p>}
          </div>

          <div>
            History:
            <ul className="ml-4">
              {stub.events
                .filter((ev) => ev.nodeId === node.id)
                .map((ev) => (
                  <li key={ev.id}>
                    {ev.at} · {ev.from ?? "(new)"} → {ev.to} ({ev.cause})
                  </li>
                ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
