// THROWAWAY PROTOTYPE for "What milestone 2's screen shows" (issue #42).
// Never merge this. It lives on the prototype/42-m2-screen branch only.
//
// An in-memory stand-in for the Rust side, so the three variants can be
// clicked in a plain browser tab with no database. It copies the rules the
// map settled (self-edge, duplicate and cycle refusals; Locked <-> Available
// arithmetic; one state event per change), roughly.
import { useState } from "react";

export type NodeState = "Locked" | "Available" | "In progress" | "Evidenced";
export type EdgeKind = "prerequisite" | "recommendation";

export type StubNode = { id: string; title: string; state: NodeState };
export type StubEdge = { id: string; from: string; to: string; kind: EdgeKind };
export type StubEvent = {
  id: number;
  nodeId: string;
  from: NodeState | null;
  to: NodeState;
  cause: string;
  at: string;
};

let counter = 0;
function nextId(prefix: string) {
  counter += 1;
  return `${prefix}${counter}`;
}
function now() {
  return new Date().toLocaleTimeString();
}

const seedNodes: StubNode[] = [
  { id: "n1", title: "Write a for loop", state: "Available" },
  { id: "n2", title: "Use a dict", state: "Available" },
  { id: "n3", title: "Write a function", state: "Available" },
  { id: "n4", title: "Read a stack trace", state: "Available" },
];
const seedEvents: StubEvent[] = seedNodes.map((n, i) => ({
  id: i + 1,
  nodeId: n.id,
  from: null,
  to: n.state,
  cause: "created",
  at: "earlier",
}));

export function useStub() {
  const [nodes, setNodes] = useState<StubNode[]>(seedNodes);
  const [edges, setEdges] = useState<StubEdge[]>([]);
  const [events, setEvents] = useState<StubEvent[]>(seedEvents);

  const title = (id: string) => nodes.find((n) => n.id === id)?.title ?? id;

  function reaches(allEdges: StubEdge[], start: string, target: string) {
    const toVisit = [start];
    const seen = new Set<string>();
    while (toVisit.length > 0) {
      const current = toVisit.pop()!;
      if (current === target) return true;
      if (seen.has(current)) continue;
      seen.add(current);
      for (const e of allEdges) {
        if (e.kind === "prerequisite" && e.from === current) toVisit.push(e.to);
      }
    }
    return false;
  }

  function hasUnmetPrereq(allEdges: StubEdge[], nodeId: string) {
    return allEdges.some(
      (e) =>
        e.kind === "prerequisite" &&
        e.to === nodeId &&
        nodes.find((n) => n.id === e.from)?.state !== "Evidenced",
    );
  }

  function setState(nodeId: string, to: NodeState, cause: string) {
    const node = nodes.find((n) => n.id === nodeId)!;
    setNodes((ns) => ns.map((n) => (n.id === nodeId ? { ...n, state: to } : n)));
    setEvents((es) => [
      ...es,
      { id: es.length + 1, nodeId, from: node.state, to, cause, at: now() },
    ]);
  }

  // Returns an error message, or null when the edge was added.
  function addEdge(from: string, to: string, kind: EdgeKind): string | null {
    if (!from || !to) return "Pick both nodes.";
    if (from === to) {
      return kind === "prerequisite"
        ? "A node can't be its own prerequisite."
        : "A node can't recommend itself.";
    }
    if (edges.some((e) => e.from === from && e.to === to && e.kind === kind)) {
      return `"${title(from)}" is already a ${kind} of "${title(to)}".`;
    }
    if (kind === "prerequisite" && reaches(edges, to, from)) {
      return `"${title(from)}" already depends on "${title(to)}", so it can't also be its prerequisite.`;
    }
    const edge: StubEdge = { id: nextId("e"), from, to, kind };
    const after = [...edges, edge];
    setEdges(after);
    const target = nodes.find((n) => n.id === to)!;
    if (kind === "prerequisite" && target.state === "Available" && hasUnmetPrereq(after, to)) {
      setState(to, "Locked", `prerequisite "${title(from)}" added`);
    }
    return null;
  }

  function removeEdge(edgeId: string) {
    const edge = edges.find((e) => e.id === edgeId)!;
    const after = edges.filter((e) => e.id !== edgeId);
    setEdges(after);
    const target = nodes.find((n) => n.id === edge.to)!;
    if (edge.kind === "prerequisite" && target.state === "Locked" && !hasUnmetPrereq(after, edge.to)) {
      setState(edge.to, "Available", `prerequisite "${title(edge.from)}" removed`);
    }
  }

  function createNode(newTitle: string) {
    const node: StubNode = { id: nextId("n"), title: newTitle, state: "Available" };
    setNodes((ns) => [...ns, node]);
    setEvents((es) => [
      ...es,
      { id: es.length + 1, nodeId: node.id, from: null, to: "Available", cause: "created", at: now() },
    ]);
  }

  return { nodes, edges, events, title, addEdge, removeEdge, createNode };
}

export type Stub = ReturnType<typeof useStub>;
