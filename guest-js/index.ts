import { invoke } from "@tauri-apps/api/core";
import {
  JoinedRoomInfo,
  MatrixClientConfig,
  Room,
  RoomsCollectionType,
  TimelineState,
} from "./types";
import { RoomStore } from "./stores/room-store.svelte";
import { RoomsCollection } from "./stores/rooms-collection.svelte";
import { LoginStore } from "./stores/login-store.svelte";
import * as events from "./tauri-events";
import { MatrixRequest, createMatrixRequest } from "./matrix-requests/requests";
import {
  timelineDataGuards,
  TimelineItem,
  TimelineItemData,
  VirtualTimelineItem,
} from "./timeline-items/timeline-item";
import { MsgLikeContent } from "./timeline-items/message-like";

export async function ping(value: string): Promise<string | null> {
  return await invoke<{ value?: string }>("plugin:matrix-svelte|ping", {
    payload: {
      value,
    },
  }).then((r) => (r.value ? r.value : null));
}

export async function loginAndCreateNewSession(
  config: MatrixClientConfig,
): Promise<null> {
  return await invoke("plugin:matrix-svelte|login_and_create_new_session", {
    config,
  });
}

export async function submitAsyncRequest(
  request: MatrixRequest,
): Promise<null> {
  return await invoke("plugin:matrix-svelte|submit_async_request", {
    request,
  });
}

export {
  Room,
  RoomsCollectionType,
  MatrixClientConfig,
  RoomStore,
  RoomsCollection,
  JoinedRoomInfo,
  LoginStore,
  events,
  TimelineItem,
  TimelineItemData,
  timelineDataGuards,
  MsgLikeContent,
  VirtualTimelineItem,
  TimelineState,
  createMatrixRequest,
  MatrixRequest,
};
