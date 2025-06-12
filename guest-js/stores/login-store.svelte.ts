import {
  RuneStore,
  type StoreHooks,
  type TauriPluginSvelteRuneStoreOptions,
} from "@tauri-store/svelte";

export const LOGIN_STATE_STORE_ID = "login-state";

export type LoginStateType = {
  state: "initiating" | "restored" | "awaitingForLogin" | "loggedIn";
  userId: string | null;
};

export class LoginStore extends RuneStore<LoginStateType> {
  /**
   * Creates a new LoginState instance
   */
  constructor() {
    const hooks: StoreHooks = {
      error: (err) => console.error(err),
    };
    const options: TauriPluginSvelteRuneStoreOptions<LoginStateType> = {
      hooks,
    };

    super(LOGIN_STATE_STORE_ID, { state: "initiating", userId: null }, options);
  }

  /**
   * Start the Rune store
   */
  async startStoreAndSendConfirmationEvent(): Promise<void> {
    await this.start();
    await this.save();
    // await emit(MatrixSvelteEmitEvent.RoomsCollectionStarted);
    // emit event to warn backend that the store has started ?
  }
}
