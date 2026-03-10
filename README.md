# 52Hz

A macOS menu bar break reminder app, inspired by DeskRest.

## Features

- Pomodoro-style work/break timer with configurable durations
- Fullscreen break overlay with media auto-pause
- Tick sound during work sessions
- Presence system -- virtual coworkers who share your work rhythm
- Internationalization (English default, Japanese)
- Menu bar only -- no Dock icon, no clutter

## Install

Download the latest DMG from [Releases](https://github.com/grad13/52Hz/releases).

Or build from source (see below).

## Build from Source

Prerequisites:

- Rust (stable)
- Node.js 20+
- Tauri CLI v2

```bash
cd code && npm install
npx tauri build --bundles app
```

The built app will be in `code/tauri/target/release/bundle/macos/`.

## Development

```bash
cd code && npm install
npx tauri dev
```

## Testing

Rust unit tests:

```bash
cd code/tauri && cargo test --lib
```

Frontend tests:

```bash
cd code && npx vitest run
```

Type checking:

```bash
cd code && npx svelte-check
```

## License

[BSL 1.1](LICENSE)
