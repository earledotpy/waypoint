// The only file that calls the Rust side. Every other file imports these
// functions, so the command names and argument names are written once, here,
// and match `src-tauri/src/commands.rs` exactly.
// See docs/learning/concepts/tauri-command.md.
import { invoke } from "@tauri-apps/api/core";

/**
 * One skill node, as `waypoint_read::SkillNode` sends it. The field names are
 * the Rust field names (and the SQL column names), snake_case included, so
 * `created_at` is the same word in all three languages.
 *
 * A Rust `Option<String>` arrives as a string or `null`.
 */
export type SkillNode = {
  id: string;
  external_id: string | null;
  title: string;
  description: string;
  state: string;
  created_at: string;
  retired_at: string | null;
};

/**
 * Adds a node and resolves to it as stored. If the Rust command returns an
 * `Err`, the promise rejects with its message, such as "A node needs a title.".
 */
export function createNode(title: string, description: string): Promise<SkillNode> {
  return invoke<SkillNode>("create_node", { title, description });
}

/** Every skill node, oldest first. */
export function listNodes(): Promise<SkillNode[]> {
  return invoke<SkillNode[]>("list_nodes");
}
