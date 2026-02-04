# tauri-plugin-matrix-svelte

A Tauri plugin that provides a high level abstraction of the [Matrix client](https://matrix.org) API and objects.
It is compatible with both desktop and mobile devices.

This plugin is an adapter for the [matrix-ui-serializable](https://github.com/IT-ess/matrix-ui-serializable) library, that wraps the [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk). 
Most of the state data (Rooms list, Room data) is accessible to the frontend through a Svelte 5 Rune store, allowing easy and fine-grained reactivity of your view.

## Showcase
<div style="display: flex; justify-content: space-between; gap: 15px; margin: 0 -5px; max-width: 100%;">
  <img src="assets/room_list.png" alt="Room List" style="width: 32%; height: auto; max-width: 100%; margin: 0 5px;">
  <img src="assets/room.png" alt="Room" style="width: 32%; height: auto; max-width: 100%; margin: 0 5px;">
  <img src="assets/media_message.png" alt="Media message" style="width: 32%; height: auto; max-width: 100%; margin: 0 5px;">
</div>

You can try the [example client](https://github.com/IT-ess/tauri-plugin-matrix-svelte/tree/main/example/matrix-svelte-client) by installing the binaries (in the Github release), or by compiling the project locally.

## Current supported features

All features supported by [matrix-ui-serializable](https://github.com/IT-ess/matrix-ui-serializable?tab=readme-ov-file#Features).

## Usage

Even if this is a plugin, most of the logic stays tighly related to the example implementation. Thus, it is recommended to use the example client as a starting point.

### Requirements

- If you need to use OAuth authentication (that is the case for matrix.org), you'll need to configure an OAuth client. The example implementation use this preconfigured [website](https://github.com/IT-ess/oauth-redirect-deeplink), that uses deeplinks to pass the OAuth code upon redirect.
- A [Sygnal push notification gateway](https://github.com/matrix-org/sygnal) if you want to configure push notifications on mobile.

### Plugin configuration

**Required** configuration variables in your `tauri.conf.json` in the plugin part, for the `matrix-svelte` key.
- `android_sygnal_gateway_url`: Push gateway url for android
-	`ios_sygnal_gateway_url`: Push gateway url for iOS
-	`oauth_client_uri`: Client URI for OAuth 
-	`oauth_redirect_uri`: Redirect URI once the OAuth process is validated (must be the same host as redirect)

### Plugin requirements
This plugin works along two other plugins, [tauri-plugin-svelte](https://tb.dev.br/tauri-store/plugin-svelte/guide/getting-started) and [tauri-plugin-notifications](https://github.com/Choochmeque/tauri-plugin-notifications), that also must be initialized with default capabilities by your Tauri app before this plugin.

### Usage in Svelte

#### Stores
The `tauri-plugin-matrix-svelte-api` NPM package exposes the types and classes you need. 
Basically, you get four kind of classes / Rune stores : 
- `RoomsCollection`: that contains all the informations to implement the rooms list view of your client
- `RoomStore`: a store that contains the timeline and other info related to a currently opened room
- `ProfileStore`: a store that contains a Map of all known users profile (avatar, name...)
- `LoginStore`: a store that contains information about the logged in user

These stores must be instantiated upon webview creation, in the `hooks.client.ts`.

#### Commands and events
Command wrappers and event types are exposed by the NPM package.
The exposed commands cover the basic operations of a Matrix client.
A lot of requests are async, and should be submitted with the `submitAsyncRequest` command.

## Building

### Building the javascript bindings

- `pnpm install`
- `pnpm build`

### Building the Rust lib

- `cargo build`

## Main Dependencies

- [matrix-ui-serializable](https://github.com/IT-ess/matrix-ui-serializable) : high level abstraction of a Matrix client in Rust
- [tauri-plugin-svelte](https://github.com/ferreira-tb/tauri-store/tree/main/packages/plugin-svelte): to communicate easily with Svelte frontend in a reactive way
- [keyring-core](https://github.com/open-source-cooperative/keyring-core) : to store the Matrix session securely in the OS keychain

# Contributing
This project is opened to all kinds of contributions. I'm aware that the [documentation](https://docs.rs/tauri-plugin-matrix-svelte) isn't exhaustive and I do not have enough time to make it so. I can still [answer some questions](#chat-about-this-project) if needed !

## Possible improvements

As mentionned in [matrix-ui-serializable's README](https://github.com/IT-ess/matrix-ui-serializable?tab=readme-ov-file#possible-improvements), the main flaw of this plugin is the full serialization of the stores whenever the state_updaters are called. To avoid serialization, passing data directly to the frontend through Tauri's raw IPC API may be possible, but that would require extra work that is perfectly done by tauri-plugin-svelte right now.

## Chat about this project

Join this [Matrix room](https://matrix.to/#/#matrix-ui-serializable:matrix.org) if you have questions about this project !


# Special thanks to :

- To [Andrew Ferreira](https://github.com/ferreira-tb) for its handy tauri-plugin-svelte plugin.
- To the whole Tauri team for their awesome platform to build on.
- And of course to the whole Matrix team and ecosystem !
