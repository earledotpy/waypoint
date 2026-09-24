// Runs before every test file (see `test.setupFiles` in vite.config.ts).
//
// The Tauri testing docs also add a `window.crypto` polyfill here, because
// `mockIPC` calls `window.crypto.getRandomValues`. jsdom 30 on Node 24
// already provides that function, so no polyfill is needed.
import { cleanup } from "@testing-library/react";
import { clearMocks } from "@tauri-apps/api/mocks";
import { afterEach } from "vitest";

// Each test starts from nothing: an empty page and no fake IPC handler left
// over from the test before it.
afterEach(() => {
  cleanup();
  clearMocks();
});
