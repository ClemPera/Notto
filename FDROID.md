# F-Droid packaging notes

Tracking readiness for submitting Nooto's Android build to F-Droid. Based on:

- [Submitting to F-Droid Quick Start Guide](https://f-droid.org/docs/Submitting_to_F-Droid_Quick_Start_Guide/)
- [All About Descriptions, Graphics, and Screenshots](https://f-droid.org/docs/All_About_Descriptions_Graphics_and_Screenshots/)
- [Inclusion Policy](https://f-droid.org/docs/Inclusion_Policy/)

## Done in this PR

- Added `fastlane/metadata/android/en-US/` at the repo root with `short_description.txt`,
  `full_description.txt`, `title.txt` and `images/icon.png` (reused from
  `client/src-tauri/icons/icon.png`, 512x512). This is the F-Droid-supported Fastlane
  structure and gets picked up automatically once the app is in the main repo, no merge
  request needed for text/graphics updates after that.
- Disabled the Play "dependencies info" block (`dependenciesInfo { includeInApk = false;
  includeInBundle = false }`) in `app/build.gradle.kts`. AGP signs a proto blob of
  dependency metadata into release builds by default, which is not reproducible across
  machines and isn't needed outside Play. F-Droid's own build compares byte-for-byte
  against what you publish, so this has to go.
- Verified the app already clears the basics: AGPL-3.0-or-later license (`LICENSE`),
  public source repo, only `INTERNET` permission in the manifest, no Firebase/GMS/analytics,
  and the one non-`androidx` dependency (`com.google.android.material`) comes from Google's
  Maven repo, which F-Droid's policy explicitly allows.

## Still needed before submitting

These need your input, a device, or a GitLab account, none of which I have access to.

1. **Real screenshots.** F-Droid wants at least one screenshot for the app to show up on
   the "Latest" tab, and the only screenshot in the repo (`.github/assets/screenshot.png`)
   is a desktop window, not a phone screenshot. Put 1+ images at
   `fastlane/metadata/android/en-US/images/phoneScreenshots/1.png`, `2.png`, etc, taken
   from an actual Android build. I didn't fabricate placeholders here since that would
   misrepresent the app.
2. **Feature graphic** (optional but recommended): a landscape banner at
   `fastlane/metadata/android/en-US/images/featureGraphic.png`, shown above the description
   in the F-Droid client.
3. ~~Commit a `Cargo.lock`~~ done.
4. **Tag every Android release.** F-Droid checks out a specific commit by its git tag, and
   the tag needs to match the `versionName` in the build (e.g. `client-v0.1.0`, which
   already exists). Keep doing this for every release, it's already the convention here.
5. **Changelog added for versionCode 1000.** Tauri derives the Android versionCode as
   `major*1000000 + minor*1000 + patch` unless overridden in
   `tauri.conf.json > bundle > android > versionCode` (not set here), so `0.1.0` is `1000`.
   `fastlane/metadata/android/en-US/changelogs/1000.txt` is added on that basis. Worth
   double-checking against the actual `client-v0.1.0` release artifact once you can, and
   worth committing `gen/android/app/tauri.properties` after your next Android build so the
   versionCode is pinned to the commit instead of recomputed. It currently isn't gitignored,
   it's just never been generated in this checkout.
6. **Don't test with plain Gradle, test with the same command your CI already uses.**
   I initially suggested testing with `./gradlew assembleRelease` directly. That was wrong,
   and if you'd left it that way in the actual `fdroiddata` recipe it would have failed on
   F-Droid's build server too, for a real architectural reason, not a config mistake: Tauri's
   generated `buildSrc/BuildTask.kt` shells out to `tauri android android-studio-script`,
   which connects to a local WebSocket RPC server that only exists while a live Tauri CLI
   process is running (started by `npm run tauri android build` or Android Studio). Plain
   Gradle, whether `./gradlew` or F-Droid's own `gradlew-fdroid`, has no such parent process,
   so the build fails with a connection-refused error trying to reach that RPC server.
   The fix is to not use F-Droid's automatic `gradle:` build method at all, and instead have
   F-Droid run the same command your own CI already runs successfully:
   `npm run tauri android build -- --apk`, which lets the Tauri CLI orchestrate the RPC
   server and Gradle itself, the way it's designed to. See the updated recipe below.
7. **The actual submission** happens in a separate repository
   (`gitlab.com/fdroid/fdroiddata`), not this one. I don't have a GitLab account or access
   to fork that repo, so this step has to be done by you:
   - Open a "Request For Packaging" issue at <https://gitlab.com/fdroid/rfp/-/issues>.
   - Fork `fdroiddata`, add `metadata/com.nooto.app.yml` (template below), and open a merge
     request there. `fdroid lint com.nooto.app` and `fdroid build com.nooto.app` (via the
     fdroidserver Docker container) can catch problems before you submit.

## Draft build metadata (for the `fdroiddata` merge request, not this repo)

This is a starting point, not something committed here, since F-Droid's build recipe
lives in their own `fdroiddata` repo. Fill in the blanks and validate with
`fdroid lint`/`fdroid build` before submitting.

```yaml
Categories:
  - Writing
License: AGPL-3.0-or-later
AuthorName: Clément Pera
AuthorWebSite: https://nooto.sh
SourceCode: https://github.com/ClemPera/Nooto
IssueTracker: https://github.com/ClemPera/Nooto/issues

RepoType: git
Repo: https://github.com/ClemPera/Nooto

Builds:
  - versionName: 0.1.0
    versionCode: 1000 # major*1000000 + minor*1000 + patch, confirm against the real build
    commit: client-v0.1.0
    subdir: client
    sudo:
      - apt-get update
      - apt-get install -y nodejs npm
    prebuild:
      - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      - . "$HOME/.cargo/env"
      - rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
      - npm install
    build:
      - . "$HOME/.cargo/env"
      - npm run tauri android build -- --apk
    output: src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk

AutoUpdateMode: Version
UpdateCheckMode: Tags ^client-v[0-9.]+$
CurrentVersion: 0.1.0
CurrentVersionCode: 1000
```

Notes on the draft:

- `subdir: client` now, not `client/src-tauri/gen/android`. Since the build runs through
  `npm run tauri android build` instead of raw Gradle, F-Droid just needs to be positioned
  wherever `package.json` lives so `npm run tauri` resolves; the Tauri CLI takes care of
  finding and driving `src-tauri/gen/android` itself. `output:` is therefore given relative
  to `client/`.
- No `gradle:` field. This is intentionally a "manual" build (F-Droid's term for when you
  supply `build:` yourself instead of using the `gradle:`/`maven:` automatic methods),
  because the automatic Gradle method calls Gradle directly, which is exactly what doesn't
  work here, see item 6 above.
- `--apk` builds a single universal APK covering every architecture, rather than per-ABI
  splits. That avoids F-Droid's multi-APK-per-build complications entirely, at the cost of a
  somewhat larger download. Fine to revisit with `--target` and multiple `Builds:` entries
  later if the download size becomes a real complaint.
- `sudo`/`prebuild` toolchain setup (installing Node and Rust) is a best guess, not verified.
  F-Droid's buildserver image may already ship Node and/or Rust given both are explicitly
  named as allowed toolchains in the Inclusion Policy; check what's actually preinstalled
  with `fdroid build -v` before assuming this is needed as written.
- `UpdateCheckMode` is scoped to `client-v*` tags only, since this repo also tags server
  releases (`server-v*`) that aren't Android builds.
- Rust toolchain and target setup will very likely need adjustment once you actually run
  this. F-Droid's policy allows prebuilt Rust/Rustup and crates.io binaries, so this is a
  policy-solved problem, but the exact `sudo`/`prebuild` incantation needs testing.

