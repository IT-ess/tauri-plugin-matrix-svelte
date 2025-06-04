import { TimelineItem } from "./timeline-items/timeline-item";

export type MatrixClientConfig = {
  username: string;
  password: string;
  homeserver_url: string;
  client_name: string;
};

// Equivalent to RoomScreen Rust Struct
export type Room = {
  roomId: string;
  roomName: string;
  tlState?: TimelineState;
};

export type TimelineState = {
  roomId: string;
  userPower: number;
  fullyPaginated: boolean;
  items: TimelineItem[];
  lastScrolledIndex: number;
  prevFirstIndex?: number;
  scrolledPastReadMarker: boolean;
  latestOwnUserReceipt?: any; // Not handled for the moment
};

// Equivalent to RoomsList Rust Struct
export type RoomsCollectionType = {
  // invitedRooms: ... ???
  allJoinedRooms: Record<string, JoinedRoomInfo>;
  // displayFilter: ???
  // displayedInvitedRooms: string[] ???
  displayedJoinedRooms: string[];
  status: RoomsCollectionStatus;
  currentActiveRoom: string | null;
  maxKnownRooms?: number;
};

export type RoomsCollectionStatus =
  | { status: "notLoaded"; message: string }
  | { status: "loading"; message: string }
  | { status: "loaded"; message: string }
  | { status: "error"; message: string };

export type JoinedRoomInfo = {
  roomId: string;
  roomName: string;
  numUnreadMessages: number;
  numUnreadMentions: number;
  canonicalAlias: string | null;
  altAlias: string[] | null;
  // tags: string[] ???
  latest: [number, string] | null;
  // avatar: string | null;
  hasBeenPaginated: boolean;
  isSelected: boolean;
};
