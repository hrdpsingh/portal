# Portal

**Portal** is a lightweight, secure file-sharing tool designed for transferring files across your local network. It allows you to serve files from your machine to any other device on the same local network via a web browser. It focuses on being extremely resource-efficient while maintaining security through encryption.

## Features
*   Serves all files over **HTTPS**.
*   Consumes less than **10MB of memory**.
*   Uses file streaming to handle multi-gigabyte files without excessive memory usage.
*   Native binaries available for Windows and Linux.

---

## Installation

You can download the latest pre-compiled binaries from the release page:

**[Download Portal Releases](https://gitlab.com/harshdeep-singh/portal/-/releases)**

*Available for: Windows and Linux.*

---

## Usage

Run the executable via your terminal or command prompt:
```console
path/to/executable
```

 Then specify the password and the paths to the files (NOT folders) you wish to share:
```console
Enter password: <your-password>
Enter the number of files to be shared: <file-count>
Enter file path 1/<file-count>: /file/path/1
Enter file path 2/<file-count>: /file/path/2
```

Once running, the terminal will display a link (e.g., `https://192.168.1.10:8000`). Open this link on any device connected to the same Wi-Fi/network to access your files.
```console
Server running at https://192.168.1.10:8000
If you are using a firewall, you may need to expose the 8000 port.
Press Ctrl+C to stop...
```

---

## Important Notes

### Firewall Configuration
Portal runs on **port 8000** by default. To allow other devices to connect, you may need to add an inbound rule to your firewall:
*   **Linux (ufw):** `sudo ufw allow 8000`
*   **Windows:** Allow the application through the Windows Defender Firewall prompt when it first runs.

### Browser Warning (Self-Signed Certificates)
To make setup instant and minimize dependencies, Portal uses **self-signed certificates** for HTTPS encryption.
*   When you visit the link, your browser will show a warning: *"Your connection is not private"* or *"Potential Security Risk Ahead"*.
*   **This is normal.** Since the certificate isn't signed by a global authority, the browser flags it.
*   **Action:** Click **"Advanced"** and then **"Proceed to [IP address] (unsafe)"** or **"Accept the Risk and Continue"**. Your connection remains encrypted.
