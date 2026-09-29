// THROWAWAY PROTOTYPE (issue #42). The floating variant switcher.
import { useEffect } from "react";

export function Switcher(props: {
  variants: { key: string; name: string }[];
  current: string;
  onChange: (key: string) => void;
}) {
  const index = props.variants.findIndex((v) => v.key === props.current);
  const step = (delta: number) => {
    const n = props.variants.length;
    props.onChange(props.variants[(index + delta + n) % n].key);
  };

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const t = e.target as HTMLElement;
      if (t.closest("input, textarea, select, [contenteditable]")) return;
      if (e.key === "ArrowLeft") step(-1);
      if (e.key === "ArrowRight") step(1);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const current = props.variants[index];
  return (
    <div
      style={{
        position: "fixed",
        bottom: 16,
        left: "50%",
        transform: "translateX(-50%)",
        background: "#111",
        color: "#fff",
        borderRadius: 999,
        padding: "6px 14px",
        display: "flex",
        gap: 12,
        boxShadow: "0 4px 12px rgba(0,0,0,.3)",
        fontFamily: "sans-serif",
        fontSize: 14,
      }}
    >
      <button onClick={() => step(-1)}>←</button>
      <span>
        PROTOTYPE · {current.key} — {current.name}
      </span>
      <button onClick={() => step(1)}>→</button>
    </div>
  );
}
