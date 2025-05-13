import { RuneStore } from "@tauri-store/svelte";
import { RoomsCollection } from "./rooms-collection.svelte";

const CONFIRMATION_EVENT_NAME = "room-created";

export type ConfirmationEventType = {
  id: string;
  message: string;
};

export type Room = {
  id: string;
  name: string;
  avatar: string | null;
  unreadCount: UnreadCount;
};

type UnreadCount = {
  highlightCount: number;
  notificationCount: number;
};

export class RoomStore extends RuneStore<Room> {
  /**
   * Creates a new RoomStore instance
   * @param id The Matrix room id used to name the store
   * @param options Optional configuration options
   */
  constructor(id: string, options = {}) {
    // Initialize with default empty Room state
    const defaultRoom: Room = {
      id: id,
      name: "",
      avatar: null,
      unreadCount: {
        highlightCount: 0,
        notificationCount: 0,
      },
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
    roomsCollection.addRunningJoinedRoomId(id);

    // // send event to rust to confirm that the store is ready
    // let payload: ConfirmationEventType = {
    //   id,
    //   message: `Room store with id ${id} has been sucessfully created and started`,
    // };
    // await emit(CONFIRMATION_EVENT_NAME, payload);
  }

  /**
   * Updates the room name
   * @param name The new name for the room
   */
  updateName(name: string): void {
    this.state.name = name;
  }

  // /**
  //  * Updates the room avatar
  //  * @param avatarUrl URL of the avatar image, or null to remove
  //  */
  // updateAvatar(avatarUrl: string | null): void {
  //   this.state.avatar = avatarUrl;
  // }

  /**
   * Increments the unread count
   * @param count Number to increment by
   * @param isMention Whether this update includes a mention
   */
  incrementUnread(count: number = 1, isMention: boolean = false): void {
    this.state.unreadCount.notificationCount += count;

    if (isMention) {
      this.state.unreadCount.highlightCount += 1;
    }
  }

  /**
   * Resets unread counts to zero
   */
  markAsRead(): void {
    this.state.unreadCount = {
      notificationCount: 0,
      highlightCount: 0,
    };
  }

  /**
   * Gets the current room data
   * @returns The current Room object
   */
  getRoom(): Room {
    return { ...this.state };
  }

  // /**
  //  * Completely replaces the room data
  //  * @param room New room data
  //  */
  // updateRoom(room: Room): void {
  //   // Create a copy to avoid direct reference mutation
  //   this.state = { ...room };
  // }
}
