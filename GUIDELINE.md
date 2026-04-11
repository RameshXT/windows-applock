# STRICT RULE — DO NOT CHANGE ANYTHING WITHOUT PERMISSION:

DO NOT remove any existing code, logic, feature, function, variable, comment, or styling under any circumstance
DO NOT modify anything outside the exact scope of what was asked
DO NOT refactor, optimize, restructure, or clean up any existing code
DO NOT rename any variable, function, component, or file
DO NOT change any design, layout, spacing, colors, or animations that already exist
DO NOT add any extra features, buttons, menus, or logic that was not explicitly requested
If adding a new feature — only add, never touch existing code unless absolutely required
If fixing a bug — only fix that exact bug, nothing else
If something seems wrong or could be improved — DO NOT change it, ask for permission first
Always return the complete updated file with only the requested change applied
When in doubt — STOP and ask, never assume

Violation of any of these rules is strictly not allowed.


# Windows AppLock — Project Guidelines

> AI Agent: Read this entire file before doing anything. Follow every rule strictly.

---

## 1. Brand

```ts
const APP_NAME = "Windows AppLock";
```

- Always define the app name as a constant variable
- Use `APP_NAME` everywhere — never hardcode the name
- Applies to: title bar, logo, login screen, setup screen, settings, about, window title

---

## 2. Tech Stack

| Layer | Technology |
|---|---|
| Frontend | React + TypeScript + CSS Modules |
| Build Tool | Vite |
| Desktop Framework | Tauri v2 |
| Backend | Rust |
| Password Hashing | Argon2id |
| Encryption | AES-256-GCM |
| Process Control | Windows API (winapi crate) |
| App Scanning | Windows Registry (Rust) |
| Async Runtime | Tokio |

---

## 3. Project Structure

```
AppLock/
├── src/                        → Frontend
│   ├── pages/                  → Full screens
│   │   ├── Home.tsx
│   │   ├── Login.tsx
│   │   ├── Setup.tsx
│   │   ├── Dashboard.tsx
│   │   └── Settings.tsx
│   ├── components/             → Reusable UI pieces
│   │   ├── Navbar.tsx
│   │   ├── AppCard.tsx
│   │   └── PinPopup.tsx
│   ├── hooks/                  → Custom React hooks
│   ├── store/                  → State management
│   ├── utils/                  → Helper functions
│   ├── styles/                 → Global CSS
│   ├── assets/                 → Logos, icons
│   │   ├── logo.png
│   │   ├── logo_square.png
│   │   └── logo_v2.png
│   └── types/                  → TypeScript interfaces
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/           → Tauri command handlers
│   │   ├── services/           → scanner, monitor, config, security
│   │   ├── models/             → Data structs and types
│   │   └── utils/              → Rust helper functions
│   ├── capabilities/
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── ANTI-GRAVITY.md
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## 4. Design System

### Theme
- **Mode:** Dark only
- **Background:** `#0a0a0a`
- **Surface:** `#111111`
- **Card:** `#1a1a1a`
- **Border:** `#2a2a2a`
- **Accent:** `#3b82f6` (blue)
- **Text Primary:** `#ffffff`
- **Text Secondary:** `#888888`
- **Error:** `#ef4444`
- **Success:** `#22c55e`

### Typography
- **Font:** Inter (primary)
- **Sizes:** 12px / 14px / 16px / 20px / 24px / 32px
- **Weight:** 400 (normal) / 500 (medium) / 600 (semibold) / 700 (bold)

### Spacing
- Base unit: `4px`
- Use multiples: `4, 8, 12, 16, 24, 32, 48, 64`

### Border Radius
- Small: `6px`
- Medium: `10px`
- Large: `16px`
- Full: `9999px`

### Design Rules
- Minimal + modern — no clutter
- No unnecessary buttons or menus
- Smooth animations only (no heavy transitions)
- Every screen must be responsive across all desktop sizes
- Cards with subtle borders — no heavy shadows
- Single accent color only — never mix multiple colors
- Icons must be consistent size and style

---

## 5. UI Screens

### Home Screen (default landing)
- Always land here on app start/restart
- AppLock logo centered at top
- 2 stat cards: **Total Apps Installed** + **Total Apps Locked** (real-time)
- 1 button: **"Get Started"** → navigates to Dashboard

### Login Screen
- PIN or Password entry
- Show attempt count
- 3 wrong attempts → 30 sec lockout with countdown
- No close button — cannot be dismissed

### Setup Screen (first launch only)
- Choose: 4-digit PIN or Password
- Confirm entry
- Save and go to Home

### Dashboard — Locked Apps Tab
- Grid of locked apps with icons
- App name + path
- Lock icon indicator
- Click to unlock/remove

### Dashboard — Unlocked Apps Tab
- All installed apps scanned from Windows Registry
- App icon + name + path
- Checkbox or lock button to add lock
- Search bar with animated placeholder typing: "Search WhatsApp", "Search Slack", "Search Teams", "Search Telegram", "Search Instagram", "Search VS Code"
- Shows total app count

### PIN Prompt Popup
- Appears when locked app tries to launch
- App is suspended immediately
- Cannot be closed by any key or button
- Correct PIN → app resumes and opens
- Wrong PIN → app is killed/blocked
- No escape, no bypass

### Settings Screen
- Change PIN / Password
- Toggle PIN or Password mode
- Wrong attempt limit (3 / 5 / 10)
- Lockout duration (30 sec / 1 min / 5 min)
- Auto start with Windows
- Light / Dark theme toggle
- Language selection
- Export locked apps list
- Import locked apps list
- Reset AppLock
- About / Credits
- **NO self-lock toggle** — AppLock is always locked by default internally

---

## 6. Security Rules

- Master PIN/Password hashed with **Argon2id** — never store raw
- Config file encrypted with **AES-256-GCM**
- Config stored in `AppData` folder
- Anti-debug: detect debugger attached → exit app
- Anti-tamper: detect config file modification → reset
- No plain text secrets anywhere in code or config
- Wrong attempt lockout enforced strictly
- AppLock itself is always protected — no bypass

---

## 7. Process Monitoring Logic

```
Every 500ms:
  → Scan all running processes
  → If locked app .exe detected:
      → Suspend process immediately (SuspendThread)
      → Show PIN popup in AppLock
      → Wait for PIN result:
          Correct PIN → ResumeThread → app opens
          Wrong PIN   → TerminateProcess → app blocked
```

**Windows API used:**
- `CreateToolhelp32Snapshot` — scan processes
- `SuspendThread` — freeze app
- `ResumeThread` — allow app to open
- `TerminateProcess` — kill app

---

## 8. Tauri Commands (Rust ↔ React Bridge)

```
get_installed_apps      → returns all installed Windows apps
save_locked_apps        → saves checked apps to encrypted config
verify_pin              → checks entered PIN against Argon2 hash
set_pin                 → hashes and saves new PIN
start_process_monitor   → starts background watcher thread
unlock_app              → resumes suspended process
block_app               → kills suspended process
change_pin              → verify old PIN + save new PIN hash
get_stats               → returns total installed + total locked count
```

---

## 9. Rust Dependencies (Cargo.toml)

```toml
argon2 = "0.5"
aes-gcm = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
winapi = { version = "0.3", features = ["processthreadsapi", "tlhelp32", "handleapi"] }
tokio = { version = "1", features = ["full"] }
```

---

## 10. Coding Rules

- Never hardcode `APP_NAME` — always use the constant
- Never store raw PIN or password anywhere
- Never rewrite logic unless explicitly asked
- Never restructure code unless explicitly asked
- Never optimize or refactor unless explicitly asked
- Always return the complete updated file — never partial
- CSS must use CSS Modules — no inline styles
- TypeScript strict mode — no `any` types
- All Rust functions must handle errors properly — no `.unwrap()` in production
- Keep components small and single responsibility
- Comment only complex logic — no obvious comments

---

## 11. Build & Output

```bash
# Development
npm run tauri dev

# Production build
npm run tauri build
```

**Output:** `.exe` + `.msi` installer in `src-tauri/target/release/bundle/`

---

## 12. What NOT to do

- Do NOT add extra menus or buttons not in this guideline
- Do NOT use multiple accent colors
- Do NOT use light theme as default
- Do NOT allow PIN prompt to be closed or bypassed
- Do NOT store any sensitive data in plain text
- Do NOT use `.unwrap()` in Rust production code
- Do NOT move `src-tauri/` root files — Tauri requirement
- Do NOT add self-lock toggle in settings

## Commit message.

- after doen with edit, give the commit message in 1 line for each changes whenever happen.
