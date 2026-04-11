<div align="center">
  <img src="src-tauri/icons/icon.png" width="128" height="128" alt="Windows AppLock Logo">
  <h1>Windows AppLock</h1>
  <p><strong>A professional, high-security application locker for Windows.</strong></p>

  [![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Version](https://img.shields.io/badge/version-0.1.0-orange.svg)](https://github.com/RameshXT/applock/releases)
  [![Platform](https://img.shields.io/badge/platform-Windows-0078d7.svg)](https://www.microsoft.com/windows)
  [![Framework](https://img.shields.io/badge/built%20with-Tauri-24c8db.svg)](https://tauri.app/)
</div>

---

## Project Overview

**Windows AppLock** is a modern security utility designed to protect your privacy by locking specific Windows applications behind a secure PIN. Whether it's your browser, messaging apps, or system tools, Windows AppLock ensures that only you have access.

### Why use it?
- **Privacy First**: Keep personal conversations and sensitive data away from prying eyes.
- **Child Safety**: Prevent children from opening restricted apps or changing system settings.
- **Enterprise Ready**: Secure workstations by locking administrative tools.

### Key Features
- **PIN Protection**: Secure any `.exe` application with a personalized PIN.
- **Real-time Monitoring**: Automatically detects when a locked app is launched.
- **Modern Interface**: Sleek, dark-themed UI built with React and Framer Motion.
- **Military-Grade Security**: Uses `Argon2id` for password hashing and `AES-256-GCM` for configuration encryption.
- **Anti-Bypass**: The PIN window is non-dismissible and persists until the correct code is entered.

---

## Table of Contents
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [License](#license)
- [Support](#support)

---

## Installation

### Prerequisites
Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) (v18 or higher)
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually comes with Windows)

### Setup Steps
1. **Clone the repository:**
   ```bash
   git clone https://github.com/RameshXT/applock.git
   cd applock
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode:**
   ```bash
   npm run tauri dev
   ```

4. **Build the production installer:**
   ```bash
   npm run tauri build
   ```

---

## Usage

### Starting the App
After running `npm run tauri dev`, the application will launch and sit in your system tray.

### Locking an Application
1. Open the **Dashboard**.
2. Click on **Scan Apps** to find installed applications.
3. Select the application you want to lock.
4. Set your secure PIN (this happens during the first-time setup).
5. The application is now protected! Whenever you try to open the locked app, a security popup will appear.

### Expected Output
When a locked app is triggered, you will see a fullscreen-like overlay:
```text
[ Security Alert ]
Application Locked: Chrome.exe
Please enter your PIN to continue.
[ _ _ _ _ ]
```

---

## Configuration

Windows AppLock handles most configurations automatically. However, you can find the underlying settings here:

- **Frontend Config**: Managed via `tauri.conf.json`.
- **Security Storage**: All settings and PIN hashes are encrypted and stored in the user's local app data directory.
- **Encryption**:
  - **PINs**: Hashed using `Argon2id`.
  - **App List**: Encrypted via `AES-256-GCM`.

---

## Project Structure

```text
AppLock/
├── src/                # Frontend (React + TypeScript)
│   ├── components/     # Reusable UI elements
│   ├── pages/          # Full-page views (Dashboard, Setup, etc.)
│   ├── styles/         # Global CSS and themes
│   └── App.tsx         # Main entry point & routing
├── src-tauri/          # Backend (Rust)
│   ├── src/            # Rust business logic & WinAPI interaction
│   ├── capabilities/   # Window and permission definitions
│   └── tauri.conf.json  # Tauri application configuration
├── public/             # Static assets
├── package.json        # Node dependencies and scripts
└── README.md           # You are here!
```

---

## Contributing

We love contributions! Whether you're fixing a bug, suggesting a feature, or improving the documentation, your help is welcome.

1. **Fork** the repository.
2. **Create a branch** (`git checkout -b feature/AmazingFeature`).
3. **Commit** your changes (`git commit -m 'Add some AmazingFeature'`).
4. **Push** to the branch (`git push origin feature/AmazingFeature`).
5. **Open a Pull Request**.

### Coding Standards
- **TypeScript**: Use strict types (no `any`).
- **Rust**: Robust error handling (no `.unwrap()`).
- **Aesthetics**: Maintain the minimal dark theme.

---

## Roadmap
- [ ] **Windows Hello**: Biometric authentication (Fingerprint/Face ID).
- [ ] **Schedule Lock**: Automatically lock apps during specific hours.
- [ ] **Multiple Users**: Different PINs for different family members.
- [ ] **Mobile Sync**: Receive alerts on your phone when a lock is triggered.
- [ ] **Stealth Mode**: Hide the AppLock process from Task Manager.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for more details.

---

## Issues and Support

Found a bug? Have a feature request?
- **Bug Reports**: Open an issue [here](https://github.com/RameshXT/applock/issues).
- **Security Vulnerabilities**: Please DM the maintainers directly.
- **General Support**: Join our community discussion board.

---

## Authors and Credits

- **RameshXT** - *Lead Architect & Developer* - [@RameshXT](https://github.com/RameshXT)
- **You?** - *This project is open for its first contributors!*

<div align="center">
  <sub>Built with power using Tauri and React</sub>
</div>
