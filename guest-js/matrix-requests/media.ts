import { Base64, UInt, MxcUri } from "./common";

export interface JsonWebKey {
  kty: "oct";
  key_ops: string[]; // Should at least contain "encrypt" and "decrypt"
  alg: "A256CTR";
  k: Base64; // URL-safe unpadded base64 string
  ext: true;
}

export interface EncryptedFile {
  url: MxcUri;
  key: JsonWebKey;
  iv: Base64;
  hashes: Record<string, Base64>;
  v: "v2";
}

export type MediaSource = PlainMediaSource | EncryptedMediaSource;

export type PlainMediaSource = { url: MxcUri }; // Corresponds to Plain variant, renamed to "url". We don't support Plain for now

export type EncryptedMediaSource = { file: EncryptedFile }; // Corresponds to Encrypted variant, renamed to "file"

export type MediaFormat = "File" | { Thumbnail: MediaThumbnailSettings };

export interface MediaThumbnailSettings {
  method: Method;
  width: UInt;
  height: UInt;
  animated: boolean;
}

export type Method = "crop" | "scale";

export interface MediaRequestParameters {
  source: MediaSource;
  format: MediaFormat;
}
