#!/usr/bin/env bash
# Manually export the iOS archive for App Store Connect using MANUAL signing.
#
# Why: `tauri ios build --export-method app-store-connect` generates an
# ExportOptions.plist with `signingStyle: automatic`, which makes Xcode try to
# use an Apple "cloud managed" distribution certificate. With this account that
# fails as: "exportArchive Cloud signing permission error".
#
# This script reuses the already-built .xcarchive and exports it with manual
# signing against the local Apple Distribution cert + App Store provisioning
# profile, bypassing cloud signing.
#
# With manual signing, EVERY nested bundle needs its own provisioning profile
# entry in ExportOptionsManual.plist — the NSE appex included. If the appex's
# entry is missing Xcode "skips" its profile and re-signs it with EMPTY
# entitlements (no App Group), so the extension can't reach the shared Matrix
# store and every push shows the SINGLE_UNREAD fallback. This script verifies
# the appex entitlements after export and fails if the App Group is absent.
#
# Second trap: `tauri ios build` archives with CODE_SIGNING_ALLOWED=NO and then
# signs ONLY the main .app inside the archive — it doesn't know about app
# extensions, so the NSE appex lands in the archive completely unsigned.
# exportArchive derives the entitlements it re-signs each bundle with from
# that bundle's EXISTING archive signature (the provisioningProfiles mapping
# only picks the embedded profile), so an unsigned appex silently exports with
# empty entitlements even when its profile is mapped correctly. Fix: sign the
# appex in the archive with its entitlements file before exporting.
#
# Usage:
#   0) One-time: in the developer portal, create an Ad Hoc distribution profile
#      named "MatrixSvelteClientNSEAdHocAppGroup" for the App ID
#      com.matrix.svelte.client.nse (App Group capability enabled) and install it.
#      The APP's App ID (com.matrix.svelte.client) also needs the
#      "Communication Notifications" capability enabled and its Ad Hoc profile
#      regenerated/re-downloaded — the sender-avatar notifications need the
#      com.apple.developer.usernotifications.communication entitlement in the
#      app's profile. (The capability doesn't exist for the NSE App ID.)
#   1) pnpm tauri ios build --export-method app-store-connect   # builds the archive; export step will fail — that's OK
#   2) ./src-tauri/gen/apple/export-appstore.sh
#   -> IPA at src-tauri/gen/apple/build/manual-export/Zwietess.ipa
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARCHIVE="$DIR/build/matrix-svelte-client_iOS.xcarchive"
OPTS="$DIR/ExportOptionsManual.plist"
OUT="$DIR/build/manual-export"

[ -d "$ARCHIVE" ] || { echo "Archive not found: $ARCHIVE — run the tauri ios build first."; exit 1; }

# Sign the NSE appex in the archive (tauri ios build leaves it unsigned, see
# header). The entitlements file is the target's own CODE_SIGN_ENTITLEMENTS;
# exportArchive then re-signs it against the mapped provisioning profile,
# carrying these entitlements over — exactly what it already does for the app.
ARCHIVE_APPEX="$ARCHIVE/Products/Applications/matrix-svelte-client.app/PlugIns/matrix-svelte-client_NSE.appex"
CERT="$(plutil -extract signingCertificate raw -o - "$OPTS")"
codesign -f -s "$CERT" \
  --entitlements "$DIR/NotificationService/NotificationService.entitlements" \
  "$ARCHIVE_APPEX"

rm -rf "$OUT"
xcodebuild -exportArchive \
  -archivePath "$ARCHIVE" \
  -exportOptionsPlist "$OPTS" \
  -exportPath "$OUT" \
  -allowProvisioningUpdates

IPA="$OUT/matrix-svelte-client.ipa"
APP_GROUP="group.com.matrix.svelte.client"

# Verify the app and NSE appex were signed with the entitlements silent
# pushes need. Both need the shared App Group; only the APP carries the
# communication-notifications entitlement (the capability doesn't exist for
# NSE App IDs — the NSE's INSendMessageIntent rewrite renders under the host
# app's entitlement).
CHECK_DIR="$(mktemp -d)"
trap 'rm -rf "$CHECK_DIR"' EXIT
unzip -q "$IPA" -d "$CHECK_DIR"
APP_BUNDLE="$CHECK_DIR/Payload/matrix-svelte-client.app"
check_entitlement() {
  local bundle="$1" required="$2"
  if ! codesign -d --entitlements - "$bundle" 2>/dev/null | grep -q "$required"; then
    echo "ERROR: $(basename "$bundle") is signed WITHOUT the $required entitlement."
    echo "Check the provisioningProfiles mapping in $OPTS: the bundle's profile must carry it (capability enabled on the App ID, profile regenerated)."
    exit 1
  fi
}
check_entitlement "$APP_BUNDLE" "$APP_GROUP"
check_entitlement "$APP_BUNDLE" "com.apple.developer.usernotifications.communication"
check_entitlement "$APP_BUNDLE/PlugIns/matrix-svelte-client_NSE.appex" "$APP_GROUP"
echo "Entitlements verified (App Group on app+appex, communication on app)."

echo "Exported: $IPA"
