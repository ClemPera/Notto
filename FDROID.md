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
3. **Commit a `Cargo.lock`.** The workspace `Cargo.toml` currently has no lockfile
   committed. Without one, F-Droid's build (and anyone else's) can resolve different
   transitive dependency versions over time, which breaks reproducibility and makes
   `versionCode`/`versionName` mismatches harder to debug. Run `cargo build` once and
   commit the generated `Cargo.lock`.
4. **Tag every Android release.** F-Droid checks out a specific commit by its git tag, and
   the tag needs to match the `versionName` in the build (e.g. `client-v0.1.0`, which
   already exists). Keep doing this for every release, it's already the convention here.
5. **Changelogs are optional but nice to have:** `fastlane/metadata/android/en-US/changelogs/<versionCode>.txt`
   (max 500 characters), where `<versionCode>` is the literal integer from the built AAB,
   not the version name. I couldn't determine the exact versionCode used for the
   `client-v0.1.0` release without running the Android build (`tauri.properties` is
   generated at build time and isn't committed), so I left this out rather than guess
   wrong. Worth checking after your next Android build with
   `unzip -p app-release.aab BundleConfig.pb` or by inspecting the built APK's manifest.
6. **Test an actual offline/CI-style build before submitting.** There's a known history of
   Tauri Android + F-Droid friction: F-Droid's build server strips and replaces the
   `gradlew` wrapper it doesn't trust, and older `tauri android build` versions assumed
   their own wrapper path and failed
   ([tauri-apps/tauri#6367](https://github.com/tauri-apps/tauri/issues/6367)). That was
   filed against a 2023 alpha, likely fixed by now given `gen/android` is already committed
   here as a plain Gradle project, but it's worth actually running the fdroidserver build
   container locally before opening a merge request rather than assuming. Instructions are
   in the Quick Start Guide linked above.
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
    versionCode: 1 # confirm against the real built AAB, see item 5 above
    commit: client-v0.1.0
    subdir: client/src-tauri/gen/android
    sudo:
      - apt-get update
      - apt-get install -y rustup
      - rustup default stable
      - rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
    prebuild:
      - cd ../../../.. && npm install --prefix client
    gradle:
      - yes

AutoUpdateMode: Version
UpdateCheckMode: Tags ^client-v[0-9.]+$
CurrentVersion: 0.1.0
CurrentVersionCode: 1
```

Notes on the draft:

- `subdir` points at `client/src-tauri/gen/android` since that's the actual Gradle project
  root (has `gradlew`, `settings.gradle.kts`, `app/`). The Fastlane metadata added in this
  PR is placed at the repo root, which F-Droid supports regardless of `subdir`.
- `UpdateCheckMode` is scoped to `client-v*` tags only, since this repo also tags server
  releases (`server-v*`) that aren't Android builds.
- The `prebuild` step is a guess at installing JS dependencies before Gradle invokes the
  Rust/Cargo build via the `rust` Gradle plugin; this needs to be verified against an
  actual fdroidserver build run, not assumed to work.
- Rust toolchain and target setup will very likely need adjustment once you actually run
  this. F-Droid's policy allows prebuilt Rust/Rustup and crates.io binaries, so this is a
  policy-solved problem, but the exact `sudo`/`prebuild` incantation needs testing.
