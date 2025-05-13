import { invoke } from "@tauri-apps/api/core";
import { RoomsCollection } from "./stores/rooms-collection.svelte";

export async function ping(value: string): Promise<string | null> {
  return await invoke<{ value?: string }>("plugin:matrix-svelte|ping", {
    payload: {
      value,
    },
  }).then((r) => (r.value ? r.value : null));
}

export { RoomsCollection };
