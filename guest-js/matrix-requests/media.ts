import { Base64, UInt, OwnedMxcUri } from "./common";

export interface JsonWebKey {
  kty: "oct";
  key_ops: string[]; // Should at least contain "encrypt" and "decrypt"
  alg: "A256CTR";
  k: Base64; // URL-safe unpadded base64 string
  ext: true;
}

export interface EncryptedFile {
  url: OwnedMxcUri;
  key: JsonWebKey;
  iv: Base64;
  hashes: Record<string, Base64>;
  v: "v2";
}

export type MediaSource =
  // | { url: OwnedMxcUri } // Corresponds to Plain variant, renamed to "url". We don't support Plain for now
  { file: EncryptedFile }; // Corresponds to Encrypted variant, renamed to "file"

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
