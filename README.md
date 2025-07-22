# tauri-plugin-matrix-svelte (WIP)

A Tauri plugin that provides Matrix communication features through Svelte Rune stores.

This plugin is essentially a port of [Robrix](https://github.com/project-robius/robrix) code and architecture to a Tauri plugin. Instead of using Makepad, it uses Tauri and a Svelte/SvelteKit frontend. Most of the data is sent to frontend through Svelte Rune Stores directly, thanks to tauri-plugin-svelte, allowing instant reactivity.

## Installation

This project is still in progress, if you want to play with the plugin, please check the example project.

## Current supported features

This project is still under heavy development. The first goal is to port almost all features from the original Robrix project.

## Main Dependencies

- [tauri 2.0](https://tauri.app/) : allowing one codebase for 5 platforms
- [matrix-sdk / matrix-sdk-ui](https://github.com/matrix-org/matrix-rust-sdk) : awesome abstractions for building matrix clients
- [iota_stronghold](https://docs.rs/crate/iota_stronghold/latest): to store securely the matrix database passphrase
- [tauri-plugin-svelte](https://github.com/ferreira-tb/tauri-store/tree/main/packages/plugin-svelte): to communicate easily with Svelte frontend in a reactive way

## Special thanks

Huge thanks to [Kevin Boos](https://github.com/kevinaboos) and the [Robius](https://github.com/project-robius) team for the awesome Robrix project that inspired me for this different implementation of their Matrix client architecture.
Thanks to [Andrew Ferreira](https://github.com/ferreira-tb) for its handy tauri-plugin-svelte plugin.
Also thanks to [inKibra](https://github.com/inKibra/tauri-plugins/tree/main/packages/tauri-plugin-notifications) and[ flapili](https://github.com/flapili/tauri-plugin-fcm) for their examples on how so setup push notifications with Tauri.
Cudos to the whole Tauri team for their awesome platform to build on.
And of course thanks to the whole Matrix.org team and ecosystem !
