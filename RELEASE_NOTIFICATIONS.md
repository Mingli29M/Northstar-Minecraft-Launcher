# Release notifications and publishing

## What the CI does

The GitHub Actions `publish` workflow builds installers for Windows, macOS (Apple Silicon + Intel), and Linux and attaches them to a **draft** GitHub Release tagged `northstar-v__VERSION__` (version comes from `tauri.conf.json` / `package.json`). Publish the draft from the Releases page when ready.

## Triggering a release

1. Merge the intended commits into `main`.
2. Fast-forward or merge `main` into `release` (or merge the release PR).
3. Push to `release` — or run **Actions → publish → Run workflow**.

## macOS signing — permanent Gatekeeper fix

Apple will show **“app is damaged”** for downloads that are not Developer ID signed + notarized.

### Without secrets (default)

CI uses **ad-hoc** signing (`APPLE_SIGNING_IDENTITY=-`). Users may need:

```bash
xattr -cr /Applications/Northstar.app
```

then right-click → **Open**.

### With Apple Developer Program (permanent)

Create a **Developer ID Application** certificate, export as `.p12`, then add repository secrets:

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | `openssl base64 -A -in certificate.p12` output |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | Optional. e.g. `Developer ID Application: Your Name (TEAMID)`. Auto-detected if omitted. |
| `APPLE_ID` | Apple ID email |
| `APPLE_PASSWORD` | [App-specific password](https://appleid.apple.com) |
| `APPLE_TEAM_ID` | 10-character Team ID |

**Or** App Store Connect API key instead of Apple ID password:

| Secret | Value |
| --- | --- |
| `APPLE_API_ISSUER` | Issuer ID |
| `APPLE_API_KEY` | Key ID |
| `APPLE_API_KEY_PATH` | Path to downloaded `.p8` on the runner (or wire a step that writes it) |

Also required: **Settings → Actions → General → Workflow permissions → Read and write**.

When `APPLE_CERTIFICATE` is present, the workflow imports it into a temporary keychain, signs with Developer ID, and Tauri notarizes using the Apple ID / API credentials.

## Finding the files

After a green run: **Releases** → draft tag `northstar-v__VERSION__` (or current version) → review assets → **Publish release**.

In-app notes: **Settings → About**.
