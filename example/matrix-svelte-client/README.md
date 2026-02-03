# matrix-svelte-client

An example frontend implementation over the [matrix-ui-serializable](https://github.com/IT-ess/matrix-ui-serializable) and tauri-plugin-matrix-svelte backend.

# Usage

## Running the app in dev mode

- Check out if you meet Tauri [prerequisites](https://tauri.app/start/prerequisites/)
- `pnpm install` (the plugin JS bindings must be built)
- [android] : add a keystore.properties file with your keystore data (see [here](https://tauri.app/distribute/sign/android/))
- `pnpm tauri dev` for desktop or `pnpm tauri [android|ios] dev` for mobile
