import { OwnedUserId } from "../matrix-requests/common";
import { TimelineItem } from "./timeline-item";

// Discriminated union for message-like content
export type MsgLikeContent = {
  threadRoot?: string; // OwnedEventId maps to string
  edited: boolean;
  reactions: ReactionsByKeyBySender;
  sender?: string;
  senderId: string;
} & MsgLikeKind;

// Discriminated union for message-like kinds (only Text for now)
export type MsgLikeKind = { kind: "text"; body: TextMessageEventContent };

// Text message content
export interface TextMessageEventContent {
  msgtype: "m.text";
  body: string;
  formatted?: FormattedBody;
}

// Formatted body structure (referenced but not defined in the Rust code)
export interface FormattedBody {
  format?: string;
  formattedBody?: string;
}

// Type guards are now optional - TypeScript can infer types automatically
export const isMsgLikeContent = (
  item: TimelineItem,
): item is TimelineItem & { kind: "msgLike" } => {
  return item.kind === "msgLike";
};

export const isTextMessage = (
  content: MsgLikeContent,
): content is MsgLikeContent & { kind: "text" } => {
  return content.kind === "text";
};

export type ReactionStatus =
  | "LocalToLocal"
  | "LocalToRemote"
  | "RemoteToRemote";

export interface ReactionInfo {
  timestamp: number; // assuming MilliSecondsSinceUnixEpoch is a UNIX timestamp in ms
  status: ReactionStatus;
}

// The inner map: sender → reaction info
export type ReactionsBySender = Record<OwnedUserId, ReactionInfo>;

// The full map: reaction key → sender map
export type ReactionsByKeyBySender = Record<string, ReactionsBySender>;
