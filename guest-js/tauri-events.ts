// Emit events

export enum MatrixSvelteEmitEvent {
  RoomsCollectionStarted = "matrix-svelte://rooms-collection-started",
  RoomCreated = "matrix-svelte://room-created",
  VerificationResult = "matrix-svelte://verification-result",
  UpdateCurrentActiveRoom = "matrix-svelte://update-current-active-room",
}

export type RoomCreatedEventType = {
  id: string;
  message: string;
};

export type VerificationResultEventType = {
  confirmed: boolean;
};

export type UpdateCurrentActiveRoom = {
  roomId: string | null;
  roomName: string | null;
};

// Listen events

export enum MatrixSvelteListenEvent {
  RoomCreate = "matrix-svelte://room-create",
  VerificationStart = "matrix-svelte://verification-start",
  MessageText = "matrix-svelte://message-text",
}

export type RoomCreateEventType = {
  id: string;
};

export type VerificationEmojisEventType = {
  emojis: string;
};

export type MessageTextEventType = {
  sender: string;
  body: string;
};
