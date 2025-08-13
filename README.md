# tauri-plugin-matrix-svelte

A Tauri plugin that provides Matrix communication features through Svelte Rune stores.

This plugin is an adapter for the [matrix-ui-serializable](https://github.com/IT-ess/matrix-ui-serializable) library, that provides high abstractions of the client state. Most of the state data is sent to frontend through Svelte Rune Stores directly, thanks to tauri-plugin-svelte, allowing instant reactivity.

## Current supported features

Check [here](https://github.com/IT-ess/matrix-ui-serializable?tab=readme-ov-file#Features).

## Installation

This project is still in progress, if you want to play with the plugin, please check the [matrix-svelte-client](github.com/IT-ess/tauri-plugin-matrix-svelte/tree/main/example/matrix-svelte-client) example provided in this same repo.

## Building

### Building the javascript bindings

- `pnpm install`
- `pnpm build`

### Building the Rust lib

- `cargo build`

## Main Dependencies

- [tauri 2.0](https://tauri.app/) : allowing one codebase for 5 platforms
- [matrix-ui-serializable](https://github.com/IT-ess/matrix-ui-serializable) : high level abstraction of a Matrix client in Rust
- [iota_stronghold](https://docs.rs/crate/iota_stronghold/latest): to store securely the matrix database passphrase
- [tauri-plugin-svelte](https://github.com/ferreira-tb/tauri-store/tree/main/packages/plugin-svelte): to communicate easily with Svelte frontend in a reactive way

## Special thanks to :

- To [Andrew Ferreira](https://github.com/ferreira-tb) for its handy tauri-plugin-svelte plugin.
- To [inKibra](https://github.com/inKibra/tauri-plugins/tree/main/packages/tauri-plugin-notifications) and [flapili](https://github.com/flapili/tauri-plugin-fcm) for their examples on how so setup push notifications with Tauri.
- To the whole Tauri team for their awesome platform to build on.
- And of course to the whole Matrix team and ecosystem !
