import { emit } from "@tauri-apps/api/event";
import {
  RuneStore,
  type StoreHooks,
  type TauriPluginSvelteRuneStoreOptions,
} from "@tauri-store/svelte";
import { MatrixSvelteEmitEvent } from "../tauri-events";
import { RoomsCollectionType } from "../types";

export const ROOMS_COLLECTION_STORE_ID = "rooms-collection";

export class RoomsCollection extends RuneStore<RoomsCollectionType> {
  /**
   * Creates a new RoomsCollections instance
   */

  constructor() {
    const hooks: StoreHooks = {
      error: (err) => console.error(err),
    };
    const options: TauriPluginSvelteRuneStoreOptions<RoomsCollectionType> = {
      hooks,
      saveOnChange: true,
      // TODO: tweak default params if necessary
      // see : https://tb.dev.br/tauri-store/plugin-svelte/guide/synchronization
      // saveStrategy: "debounce",
      // saveInterval: 500,
    };

    super(
      ROOMS_COLLECTION_STORE_ID,
      {
        allJoinedRooms: {
          // should be replaced during first sync
          default: {
            roomId: "default",
            roomName: "default",
            numUnreadMentions: 0,
            numUnreadMessages: 0,
            latest: null,
            altAlias: null,
            canonicalAlias: null,
            hasBeenPaginated: false,
            isSelected: false,
          },
        },
        displayedJoinedRooms: [],
        status: { status: "notLoaded", message: "Initiating" },
        currentActiveRoom: null,
        maxKnownRooms: undefined,
      },
      options,
    );
  }

  /**
   * Start the Rune store
   */
  async startStoreAndSendConfirmationEvent(): Promise<void> {
    await this.start();
    await this.save();
    await emit(MatrixSvelteEmitEvent.RoomsCollectionStarted);
  }

  /**
   * Gets the current joined rooms ids
   * @returns An array of joined rooms
   */
  getDisplayedJoinedRoomsIds(): string[] {
    const ids = this.state.displayedJoinedRooms; // pass by value
    return ids;
  }

  /**
   * Adds a room id to the joined rooms list
   * @returns The added id
   */
  addDisplayedJoinedRoomId(id: string): string {
    if (!this.isDisplayedRoomRunning(id)) {
      this.state.displayedJoinedRooms.push(id);
    }
    return id;
  }

  /**
   * Checks from a room id if its store is running already
   */
  isDisplayedRoomRunning(id: string): boolean {
    return this.state.displayedJoinedRooms.includes(id);
  }
}
