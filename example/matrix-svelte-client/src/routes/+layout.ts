// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)

import { RoomsCollection } from "tauri-plugin-matrix-svelte-api";
import type { LayoutLoad } from "./$types";

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

const roomIdsCollection = new RoomsCollection();
// await ;

export const load: LayoutLoad = async () => {
  // We only have access to the window object from here
  let collection = await roomIdsCollection.startStoreAndSendConfirmationEvent();
  return { collection };
};
