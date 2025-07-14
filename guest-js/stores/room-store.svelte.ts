import {
  RuneStore,
  type StoreHooks,
  type TauriPluginSvelteRuneStoreOptions,
} from "@tauri-store/svelte";
import type { Room } from "../types";
import { emit } from "@tauri-apps/api/event";
import { RoomsCollection } from "./rooms-collection.svelte";
import { MatrixSvelteEmitEvent, RoomCreatedEventType } from "../tauri-events";

export class RoomStore extends RuneStore<Room> {
  /**
   * Creates a new RoomStore instance
   * @param id The Matrix room id used to name the store
   * @param options Optional configuration options
   */
  constructor(
    id: string,
    options: TauriPluginSvelteRuneStoreOptions<Room> = {},
  ) {
    // Initialize with default empty Room state
    const defaultRoom: Room = {
      roomId: "Not known yet",
      roomName: "Not known yet",
      tlState: undefined,
      members: {},
    };

    const hooks: StoreHooks = {
      error: (err) => console.error(err),
    };

    options = {
      hooks,
      syncStrategy: "debounce",
      syncInterval: 1000,
    };

    super(id, defaultRoom, options);
  }

  /**
   * Start the Rune store
   */
  async startStoreAndSendConfirmationEvent(
    id: string,
    roomsCollection: RoomsCollection,
  ): Promise<void> {
    // start the store
    await this.start();

    // add the id to the running stores set
    roomsCollection.addDisplayedJoinedRoomId(id);

    // send event to rust to confirm that the store is ready
    let payload: RoomCreatedEventType = {
      id,
      message: `Room store with id ${id} has been sucessfully created and started`,
    };
    await emit(MatrixSvelteEmitEvent.RoomCreated, payload);
  }
}
