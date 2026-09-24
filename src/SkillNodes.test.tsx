// These tests render `SkillNodes` in jsdom and replace the Rust side with
// `mockIPC`: every `invoke` goes to the function passed to `mockIPC`, which
// plays the part of `src-tauri/src/commands.rs`. So they check the screen and
// `api.ts` together, without starting Tauri.
import { fireEvent, render, screen, within } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { expect, test } from "vitest";
import type { SkillNode } from "./api";
import SkillNodes from "./SkillNodes";

// A node as the Rust side would send it, with only the fields a test cares
// about filled in differently.
function node(id: string, title: string, created_at: string): SkillNode {
  return {
    id,
    external_id: null,
    title,
    description: "",
    state: "available",
    created_at,
    retired_at: null,
  };
}

test("shows the nodes list_nodes returns", async () => {
  mockIPC((cmd) => {
    if (cmd === "list_nodes") {
      return [
        node("1", "Count to ten", "2026-09-23T09:00:00.000Z"),
        node("2", "Add single digits", "2026-09-23T10:00:00.000Z"),
      ];
    }
  });

  render(<SkillNodes />);

  // `findBy…` waits for the text to appear, because the list arrives after
  // the first render.
  const list = await screen.findByRole("list");
  const items = within(list).getAllByRole("listitem");
  // Shown in the order Rust returned them, which is oldest first.
  expect(items).toHaveLength(2);
  expect(items[0].textContent).toContain("Count to ten");
  expect(items[0].textContent).toContain("available");
  expect(items[0].querySelector("time")?.getAttribute("datetime")).toBe(
    "2026-09-23T09:00:00.000Z",
  );
  expect(items[1].textContent).toContain("Add single digits");
});

test("adds a created node to the list and clears the form", async () => {
  // What `create_node` was called with, so the test can check the argument
  // names match the Rust ones.
  let createArgs: unknown;
  mockIPC((cmd, args) => {
    if (cmd === "list_nodes") {
      return [];
    }
    if (cmd === "create_node") {
      createArgs = args;
      return node("1", "Read a Rust compiler error", "2026-09-24T09:00:00.000Z");
    }
  });

  render(<SkillNodes />);
  await screen.findByText("No skill nodes yet.");

  const title = screen.getByLabelText("Title") as HTMLInputElement;
  const description = screen.getByLabelText("Description (optional)") as HTMLTextAreaElement;
  fireEvent.change(title, { target: { value: "Read a Rust compiler error" } });
  fireEvent.change(description, { target: { value: "Find the line and the fix." } });
  fireEvent.click(screen.getByRole("button", { name: "Create node" }));

  expect(await screen.findByText("Read a Rust compiler error")).toBeTruthy();
  expect(screen.queryByText("No skill nodes yet.")).toBeNull();
  expect(createArgs).toEqual({
    title: "Read a Rust compiler error",
    description: "Find the line and the fix.",
  });
  expect(title.value).toBe("");
  expect(description.value).toBe("");
});

test("shows the error when create_node fails", async () => {
  mockIPC((cmd) => {
    if (cmd === "list_nodes") {
      return [];
    }
    if (cmd === "create_node") {
      // A Rust command that returns `Err(message)` makes `invoke` reject
      // with that message as a plain string.
      return Promise.reject("database error: disk I/O error");
    }
  });

  render(<SkillNodes />);
  await screen.findByText("No skill nodes yet.");

  const title = screen.getByLabelText("Title") as HTMLInputElement;
  fireEvent.change(title, { target: { value: "Read a Rust compiler error" } });
  fireEvent.click(screen.getByRole("button", { name: "Create node" }));

  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toBe("database error: disk I/O error");
  // The typed title is kept, so the user can fix it and try again, and
  // nothing was added to the list.
  expect(title.value).toBe("Read a Rust compiler error");
  expect(screen.getByText("No skill nodes yet.")).toBeTruthy();
});
