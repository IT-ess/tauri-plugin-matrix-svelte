# Apple release legs (macOS + iOS): postponed

The release pipeline (`.github/workflows/release.yml`, tracked in `CI_CD_TASKS.md`) ships Linux, Windows and Android. The two Apple legs are **postponed** until the Apple Developer Program membership is renewed (decision of 2026-10-22). This file has everything needed to add them later:

- **macOS leg** (`CI_CD_TASKS.md` subtask 4b / Gate 4): aarch64 only, signed with the Developer ID certificate and notarized.
- **iOS leg** (`CI_CD_TASKS.md` Phase 6 / Gate 6): an Ad Hoc IPA for registered devices, hosted on CrabNebula.

Nothing else in the pipeline waits on these legs. The release body already labels `.dmg` and `.ipa` assets. The updater config already has the generic endpoint that macOS needs (see "Updater on macOS" below). `publish` only waits on the legs listed in its `needs`.

> The YAML below is **untested**. It follows the patterns that the Linux, Windows and Android legs proved in CI. Each part that hasn't run yet is marked with a gate check.

## 0. Renew and collect (manual, on the Mac)

Renew the Apple Developer Program membership first. A lapsed membership breaks both legs:

- **Notarization** needs an active membership.
- **Provisioning profiles** (the macOS `embedded.provisionprofile` and the iOS Ad Hoc profiles) stop being valid once the membership lapses.

Check on developer.apple.com whether existing certificates and profiles survived the lapse, and regenerate any that didn't. Any regenerated profile must be re-uploaded as a secret below.

Then add these repo secrets (`gh secret set NAME -R IT-ess/tauri-plugin-matrix-svelte < file`; base64 values with `base64 -i file | gh secret set NAME …`):

| Secret                          | Leg   | Content                                                                                                                                                                                                                |
| ------------------------------- | ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `APPLE_CERTIFICATE`             | macOS | base64 of the **Developer ID Application** cert + key exported as `.p12` (Keychain Access → export)                                                                                                                    |
| `APPLE_CERTIFICATE_PASSWORD`    | macOS | the `.p12` export password                                                                                                                                                                                             |
| `MACOS_PROVISION_PROFILE`       | macOS | base64 of `MatrixSvelteClient.provisionprofile` (currently `/Users/itess/Documents/keys/`)                                                                                                                             |
| `APPLE_API_ISSUER`              | both  | App Store Connect API issuer ID (Users and Access → Integrations)                                                                                                                                                      |
| `APPLE_API_KEY_ID`              | both  | the API key ID                                                                                                                                                                                                         |
| `APPLE_API_KEY`                 | both  | the `.p8` file **content** (not base64)                                                                                                                                                                                |
| `IOS_DIST_CERTIFICATE`          | iOS   | base64 of the **Apple Distribution** cert + key as `.p12`. It must be the cert whose SHA-1 is in `gen/apple/ExportOptionsManual.plist` (`signingCertificate`, `4A76…1329`). If you make a new cert, update that plist. |
| `IOS_DIST_CERTIFICATE_PASSWORD` | iOS   | its `.p12` export password                                                                                                                                                                                             |
| `IOS_PROFILE_APP`               | iOS   | base64 of the Ad Hoc profile `MatrixSvelteClientAdHocV3` (`com.matrix.svelte.client`, with App Group and Communication Notifications)                                                                                  |
| `IOS_PROFILE_NSE`               | iOS   | base64 of the Ad Hoc profile `MatrixSvelteClientNSEAdHocAppGroup` (`com.matrix.svelte.client.nse`, with App Group)                                                                                                     |

**Profile names:** the names in `ExportOptionsManual.plist` must match the profiles' names exactly. If you regenerate a profile under a new name, update the plist too.

## 1. macOS leg

### Facts it relies on

- **Signing identity:** `tauri.conf.json` sets `signingIdentity: "Developer ID Application: Alexis Loiseau (9A6YPCKFPC)"`. The `APPLE_SIGNING_IDENTITY` env var must be the same string.
- **Provisioning profile path:** `bundle.macOS.files."embedded.provisionprofile"` points at an **absolute local path** (`/Users/itess/…`), which doesn't exist on the runner. CI overrides it with `--config`, and the local setup stays unchanged.
  - The profile is what grants the keychain access the session store relies on (`apple-native-keyring-store`). Without it, the app builds but can't persist the Matrix session. Gate 4 checks this.
- **Signing and notarization:** Tauri imports `APPLE_CERTIFICATE` into a temporary keychain itself. It notarizes when `APPLE_API_ISSUER`, `APPLE_API_KEY` (the key **ID**) and `APPLE_API_KEY_PATH` are set.
- **Runner:** `macos-latest` is arm64, and it's free for public repos (as are the Linux and Windows runners).
- **Bundles:** they land in `target/aarch64-apple-darwin/release/bundle`. The existing "Stage bundles for CrabNebula" step already moves `target/*/release/bundle`, so the upload needs no change.

### Workflow changes (`build_desktop`)

1. **Matrix:** replace the `# macOS … pending` comment with:

<!-- prettier-ignore -->
```yaml
          # Apple silicon only; signed with Developer ID and notarized.
          - os: macos-latest
```

2. **Build step:** restrict the existing `Build` step to `if: runner.os != 'macOS'`, and add:

<!-- prettier-ignore -->
```yaml
      - name: Build (macOS)
        if: runner.os == 'macOS'
        working-directory: example/matrix-svelte-client
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: 'Developer ID Application: Alexis Loiseau (9A6YPCKFPC)'
          APPLE_API_ISSUER: ${{ secrets.APPLE_API_ISSUER }}
          APPLE_API_KEY: ${{ secrets.APPLE_API_KEY_ID }}
          APPLE_API_KEY_P8: ${{ secrets.APPLE_API_KEY }}
          MACOS_PROVISION_PROFILE: ${{ secrets.MACOS_PROVISION_PROFILE }}
        run: |
          base64 -d <<< "$MACOS_PROVISION_PROFILE" > "$RUNNER_TEMP/embedded.provisionprofile"
          export APPLE_API_KEY_PATH="$RUNNER_TEMP/AuthKey_$APPLE_API_KEY.p8"
          printf '%s' "$APPLE_API_KEY_P8" > "$APPLE_API_KEY_PATH"
          pnpm tauri build --target aarch64-apple-darwin --config \
            "{\"bundle\":{\"macOS\":{\"files\":{\"embedded.provisionprofile\":\"$RUNNER_TEMP/embedded.provisionprofile\"}}}}"
```

3. **`publish`:** no change. It already needs `build_desktop`, which then includes macOS.

### Updater on macOS

Tauri reports the bundle type `app` on macOS, so the first updater endpoint asks CrabNebula for `darwin-aarch64-app`. CN answers **400** for that (checked 2026-10-18), and the updater falls back to the generic `{{target}}-{{arch}}` endpoint, i.e. `darwin-aarch64`. Nothing to change. Just confirm it in Gate 4 with a real update.

### Gate 4 (once 4b is in)

- [ ] The CN draft holds the `.dmg` (expected public platform `dmg-aarch64`) and the updater bundle `.app.tar.gz`, with its `.sig`, under update platform `darwin-aarch64`. Check with `cn_linux release show refs/matrix-svelte-client <version> --channel dryrun`.
- [ ] The downloaded `.dmg` passes `spctl -a -vv -t install <dmg>` ("Notarized Developer ID"), and the app passes `spctl -a -vv`.
- [ ] The app launches, logs in, and **still has its session after a restart**, which proves the provisioning profile is embedded.
- [ ] The update endpoint for `darwin-aarch64` returns the new version for an older one, and an older installed build updates.

## 2. iOS leg

### Facts it relies on

- **Signing team:** `gen/apple/project.yml` sets `DEVELOPMENT_TEAM: 9A6YPCKFPC`, bundle ids `com.matrix.svelte.client` and `com.matrix.svelte.client.nse`, and arm64 only.
- **Why the manual export:** `tauri ios build` archives with signing off and then signs **only the main app**, which leaves the NSE appex unsigned. Its export also uses automatic/cloud signing, which fails for this account ("Cloud signing permission error"). `gen/apple/export-appstore.sh` fixes both:
  - it signs the appex with its entitlements
  - it exports with `ExportOptionsManual.plist` (`release-testing`, manual signing)
  - it **verifies** that the App Group and Communication Notifications entitlements are on the IPA
  - CI runs the script unchanged
- **NSE package prerequisite:** the NSE Swift package (`tauri-plugin-notifications/ios`) needs the submodule's `.tauri/tauri-api` bridge, which a cargo build for `aarch64-apple-ios` generates (see CLAUDE.md). So a `cargo check` of that crate runs before the Xcode build.
- **CN upload:** cn's mobile auto-discovery is undocumented (same as for Android), so the IPA gets an explicit `--file` with a versioned name.

### New job (add after `build_android`, then add `build_ios` to `publish.needs`)

<!-- prettier-ignore -->
```yaml
  build_ios:
    needs: [prepare, draft]
    runs-on: macos-latest
    env:
      VERSION: ${{ needs.prepare.outputs.version }}
    steps:
      - uses: actions/checkout@v7
        with:
          ref: ${{ needs.prepare.outputs.sha }}
          submodules: recursive
      - uses: maxim-lobanov/setup-xcode@v1
        with:
          xcode-version: latest-stable
      - uses: pnpm/action-setup@v6
      - uses: actions/setup-node@v7
        with:
          node-version-file: .node-version
          cache: pnpm
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios
      - uses: Swatinem/rust-cache@v2
      - run: pnpm install --frozen-lockfile
      - run: pnpm build
      - name: Import signing certificate and profiles
        env:
          IOS_DIST_CERTIFICATE: ${{ secrets.IOS_DIST_CERTIFICATE }}
          IOS_DIST_CERTIFICATE_PASSWORD: ${{ secrets.IOS_DIST_CERTIFICATE_PASSWORD }}
          IOS_PROFILE_APP: ${{ secrets.IOS_PROFILE_APP }}
          IOS_PROFILE_NSE: ${{ secrets.IOS_PROFILE_NSE }}
        run: |
          KEYCHAIN="$RUNNER_TEMP/ios-signing.keychain-db"
          kc_pw=$(openssl rand -hex 16)
          security create-keychain -p "$kc_pw" "$KEYCHAIN"
          security set-keychain-settings -lut 21600 "$KEYCHAIN"
          security unlock-keychain -p "$kc_pw" "$KEYCHAIN"
          base64 -d <<< "$IOS_DIST_CERTIFICATE" > "$RUNNER_TEMP/dist.p12"
          security import "$RUNNER_TEMP/dist.p12" -P "$IOS_DIST_CERTIFICATE_PASSWORD" -A -t cert -f pkcs12 -k "$KEYCHAIN"
          security set-key-partition-list -S apple-tool:,apple: -k "$kc_pw" "$KEYCHAIN"
          security list-keychains -d user -s "$KEYCHAIN" $(security list-keychains -d user | tr -d '"')
          # Older and newer Xcode look in different folders; install into both.
          for dir in "$HOME/Library/MobileDevice/Provisioning Profiles" \
                     "$HOME/Library/Developer/Xcode/UserData/Provisioning Profiles"; do
            mkdir -p "$dir"
            for p in "$IOS_PROFILE_APP" "$IOS_PROFILE_NSE"; do
              base64 -d <<< "$p" > "$RUNNER_TEMP/p.mobileprovision"
              uuid=$(security cms -D -i "$RUNNER_TEMP/p.mobileprovision" | plutil -extract UUID raw -)
              cp "$RUNNER_TEMP/p.mobileprovision" "$dir/$uuid.mobileprovision"
            done
          done
      # Generates tauri-plugin-notifications/.tauri/tauri-api, which the NSE Swift package links.
      - run: cargo check --release --target aarch64-apple-ios -p tauri-plugin-notifications
      - name: Build archive
        working-directory: example/matrix-svelte-client
        env:
          APPLE_DEVELOPMENT_TEAM: 9A6YPCKFPC
        # The export at the end of `tauri ios build` fails with cloud signing (see
        # export-appstore.sh). Only the archive is needed; the next step exports it.
        run: |
          pnpm tauri ios build --export-method release-testing \
            || test -d src-tauri/gen/apple/build/matrix-svelte-client_iOS.xcarchive
      - name: Sign NSE, export and verify entitlements
        run: example/matrix-svelte-client/src-tauri/gen/apple/export-appstore.sh
      - name: Stage IPA
        run: |
          mkdir -p "$RUNNER_TEMP/ipa"
          cp example/matrix-svelte-client/src-tauri/gen/apple/build/manual-export/matrix-svelte-client.ipa \
            "$RUNNER_TEMP/ipa/matrix-svelte-client_${VERSION}_aarch64.ipa"
      - uses: crabnebula-dev/cloud-release@v0
        with:
          working-directory: ${{ runner.temp }}/ipa
          command: release upload ${{ env.CN_APPLICATION }} ${{ env.VERSION }} --file matrix-svelte-client_${{ env.VERSION }}_aarch64.ipa ${{ env.CN_CHANNEL }}
          api-key: ${{ secrets.CN_API_KEY }}
      - name: Remove signing keychain
        if: always()
        run: security delete-keychain "$RUNNER_TEMP/ios-signing.keychain-db" || true
```

### Gate 6

- [ ] The job's "Sign NSE, export and verify entitlements" step prints `Entitlements verified`.
- [ ] `matrix-svelte-client_<version>_aarch64.ipa` is on the CN draft.
- [ ] The IPA installs on a **registered** device (Ad Hoc). The app logs in, and a silent push shows the decrypted message, not the `SINGLE_UNREAD` fallback, which proves the App Group reached the NSE.

### Known pitfalls

- **Cert and profiles must match the plist.** If `export-appstore.sh` fails on `signingCertificate`, the imported `.p12` isn't the cert whose SHA-1 is in `ExportOptionsManual.plist`. If it fails on profiles, a profile's name doesn't match the plist.
- **If the tolerated failure in "Build archive" also stops the archive** (the `test -d` fails), read the real error. The `|| test -d` only hides the expected export failure, not a compile error.
- **Xcode version:** `latest-stable` can move under you. Pin it in `setup-xcode` if a new Xcode breaks the build.

## 3. When both legs are done

- Tick 4b / Gate 4 and Phase 6 / Gate 6 in `CI_CD_TASKS.md`.
- Gate 0 can then be ticked, once `gh secret list` shows every secret.
- Mention the macOS and iOS downloads in the README if you want them advertised.
