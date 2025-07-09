import {
  RuneStore,
  type StoreHooks,
  type TauriPluginSvelteRuneStoreOptions,
} from "@tauri-store/svelte";
import type { UserId } from "../matrix-requests/common";

export const PROFILES_STORE_ID = "profiles";

export type ProfileType =
  | { state: "requested" }
  | {
      state: "loaded";
      data: {
        userId: string;
        username: string;
        avatarUrl?: string;
        rooms: string[];
      };
    };

export type ProfileRecord = Record<UserId, ProfileType>;

export class ProfileStore extends RuneStore<ProfileRecord> {
  /**
   * Creates a new ProfileStore instance
   */
  constructor() {
    const hooks: StoreHooks<ProfileRecord> = {
      error: (err) => console.error(err),
    };
    const options: TauriPluginSvelteRuneStoreOptions<ProfileRecord> = {
      hooks,
      syncStrategy: "debounce",
      syncInterval: 1000,
    };

    super(PROFILES_STORE_ID, {}, options);
  }
}
