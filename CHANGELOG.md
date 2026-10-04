# Changelog

## 0.4.1 - 2026-10-04

### Features

- Add switch to define privacy of a room
- Update to tauri 2.11 and fix android keyring context init
- Display rich text for events that contain some
- Room/user pills and room previews (#27)
- Better management of event focus in room timelines
- Configure push notification gateway
- Configure base notifications for iOS
- Setup silent notifications with dummy handler
- Display basic working decrypted silent notification
- Message style notifications that stacks
- Notifications trigger matrix uri handler
- Working iOS silent notification decryption through NSE
- Better styling of iOS notifications + open discussion on click
- Pass and fetch room avatar too
- **ios:** Manage badges and notification dismissal on read receipt
- **example:** Virtualize room timeline with shadcn chat components
- **example:** Use Bubble collapsible pattern for long text messages
- **example:** Overlay desktop message actions on the bubble
- **example:** Show thread root and spinner while a thread loads
- **example:** Desktop auto-updates via CrabNebula Cloud

### Bug fixes

- Cargo lock
- Send real read receipt instead of account data read receipt
- Regenerate project with latest cli
- Cold path event fetching through appropriate init
- Crash when opening from cold silent notification
- Refresh session if needed
- Display room avatar
- Cleanup old android plugin attempts for push notifications
- Correctly apply google-services
- **android:** Cold path silent notifications due to proguard policy
- Android single process configuration
- Do not show notification placeholder when receiving read receipt
- Dismiss read notifications and stack messages on android
- Adapt to pass the room avatar on android
- Open the room directly when the matrix uri is an event
- First auto claude review
- Adapt to matrix-ui-serializable api change
- Adapt to tauri-plugin-notifications refacto
- Ios compilation
- Use git submodule for plugin-notifications ios linking
- Pnpm-workspace allowbuild
- **android:** Configure CA allow-list after sdk changes
- **android:** Regenerate project with new gradle and dsl
- **android:** Borrow ndk_context in silent-push init for tao 0.37
- **example:** Keep timeline follow state in sync after navigation and jumps
- **example:** Send read receipt when the latest message arrives after the unread count
- **example:** Don't mark a room read before jumping to a focused event
- **example:** Correct jump-to-message offset after rows are measured
- **example:** Drop hidden read marker from the timeline list
- Reactions alignement
- Hide avatar for own sent messages
- Pagination marker
- Remove debounced load for timeline
- Remove content-visibility: auto on videos too
- **example:** Bump rustls-platform-verifier to 0.7.1
- First run of sv migrate

### Other

- Pnpm-lock
- Update sdk to 0.18
- Use local package
- Pass android data directory to silent push handler
- Set node version to 24
- Ios generated files
- **android:** Adjust minSdk and gradle plugin version
- Reference git packages + update deps
- Bump to latest version of SDK
- Update notifications submodule
- Bump all deps
- Merge plugin and example into single cargo + pnpm workspaces
- Add AI notice
- Add shadcn-svelte skill
- Base package upgrade to tauri 2.12
- Use svelte mcp
- Bump tauri-plugin-notifications submodule
- Migrate to SvelteKit 3 config and dependencies
- Delete migration_tasks
- Add release pipeline plan (CI_CD_TASKS.md)
- Add version bump script and git-cliff changelog config
- Release workflow skeleton (prepare, CrabNebula draft, Linux build)
- Temporarily set pnpm minimumReleaseAge to 1 minute
- Pin pnpm 11.3.0 via packageManager
- Build the plugin JS package before the app
- Upload the rpm as a signed update bundle
- Record Gate 3 results in CI_CD_TASKS.md
- Add the Windows build leg (unsigned)
- Move actions to their Node 24 majors
