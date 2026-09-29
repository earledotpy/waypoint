// THROWAWAY PROTOTYPE (issue #42). Mounts one variant, picked by ?variant=.
// One stub is shared by all three, so the graph you build survives a switch.
import { useState } from "react";
import { useStub } from "./stub";
import { Switcher } from "./Switcher";
import { VariantA, nameA } from "./VariantA";
import { VariantB, nameB } from "./VariantB";
import { VariantC, nameC } from "./VariantC";

const variants = [
  { key: "A", name: nameA },
  { key: "B", name: nameB },
  { key: "C", name: nameC },
];

export default function Prototype() {
  const stub = useStub();
  const [variant, setVariant] = useState(
    new URLSearchParams(window.location.search).get("variant") ?? "A",
  );
  const change = (key: string) => {
    const url = new URL(window.location.href);
    url.searchParams.set("variant", key);
    window.history.replaceState(null, "", url);
    setVariant(key);
  };
  return (
    <div className="space-y-4 pb-20">
      <p role="note" className="border p-2">
        THROWAWAY PROTOTYPE for issue #42. Data is fake and lives in memory;
        reload to reset. Nothing here reaches the Rust side.
      </p>
      {variant === "A" && <VariantA stub={stub} />}
      {variant === "B" && <VariantB stub={stub} />}
      {variant === "C" && <VariantC stub={stub} />}
      <Switcher variants={variants} current={variant} onChange={change} />
    </div>
  );
}
