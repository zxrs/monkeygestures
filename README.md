# MonkeyGestures

An experimental firefox extension for mouse gestures. It allows you to gesture on all pages including chrome page, such as "Option", "Newtab" and also "addons.mozilla.org".

## Installation

- **Edit manifest**
Specify a monkeygestures.exe path into /exe/monkeygestures.json
- **Add registry key**
Specify a monkeygestures.json path into \HKEY_CURRENT_USER\Software\Mozilla\NativeMessagingHosts\MonkeyGestures

More details on [Native manifests](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests).

## Issues

- Go back and go forward does not work on chrome page and "AMO".
- Sometimes context menu shows for a moment.
- x86_64-pc-msvc toolchain does not work. You have to use i686-pc-msvc on 64bit platform.
