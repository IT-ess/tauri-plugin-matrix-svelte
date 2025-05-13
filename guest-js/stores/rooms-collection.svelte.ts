import {
  RuneStore,
  type TauriPluginSvelteRuneStoreOptions,
} from "@tauri-store/svelte";
import { emit } from "@tauri-apps/api/event";

export const ROOMS_COLLECTION_STORE_ID = "rooms-collection";
const ROOMS_COLLECTION_START_EVENT = "rooms-collection-started";

type RoomsCollectionType = {
  runningJoinedRoomsStores: Set<string>;
};

export class RoomsCollection extends RuneStore<RoomsCollectionType> {
  /**
   * Creates a new RoomsCollections instance
   * @param options Optional configuration options
   */
  constructor() {
    const options: TauriPluginSvelteRuneStoreOptions<RoomsCollectionType> = {
      saveOnChange: true,
      // TODO: tweak default params if necessary
      // see : https://tb.dev.br/tauri-store/plugin-svelte/guide/synchronization
      saveStrategy: "debounce",
      saveInterval: 500,
    };

    super(
      ROOMS_COLLECTION_STORE_ID,
      { runningJoinedRoomsStores: new Set<string>() },
      options,
    );
  }

  /**
   * Start the Rune store
   */
  async startStoreAndSendConfirmationEvent(): Promise<this> {
    // this.enabled
    await this.start();
    await emit(ROOMS_COLLECTION_START_EVENT);
    return this;
  }

  /**
   * Gets the current joined rooms ids
   * @returns An array of joined rooms
   */
  getRunningJoinedRoomsStoreIds(): Set<string> {
    return { ...this.state.runningJoinedRoomsStores };
  }

  /**
   * Adds a room id to the joined rooms list
   * @returns The added id
   */
  addRunningJoinedRoomId(id: string): string {
    this.state.runningJoinedRoomsStores.add(id);
    return id;
  }

  /**
   * Checks from a room id if its store is running already
   */
  isJoinedRoomRunning(id: string): boolean {
    return this.state.runningJoinedRoomsStores.has(id);
  }
}
