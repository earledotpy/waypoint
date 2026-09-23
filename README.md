# Waypoint

A local-first desktop app for tracking skill development through **evidence of doing**, not self-reported progress. It's a Tauri v2 + Rust + React rewrite of the ideas in [SkillTrace](https://github.com/earledotpy/skilltrace).

It is also a learning project. Agents build it, and the author observes, reviews and decides. Every change comes with explanations written to be learned from (design doc §12).

**Status:** milestone 1 (walking skeleton) in progress. The app opens an empty window.

- [Design document](docs/design-document.md)
- [Architecture & schema](docs/architecture-schema.md)

## Run it

You need Node 24.15 or newer, [rustup](https://rustup.rs/), and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform. rustup installs the pinned Rust version (see `rust-toolchain.toml`) by itself.

```sh
npm install
npm run tauri dev
```

The first build is slow, about 10 minutes, because it compiles Tauri and all its dependencies from scratch. Later builds reuse that work and take seconds.
