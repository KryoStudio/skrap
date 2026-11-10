# Changelog

Every version of Skrap, newest first. The app shows the same notes once after an update ("What's new"), from [`app/src/lib/changelog.ts`](app/src/lib/changelog.ts): keep both in step (`node scripts/version.mjs check` does).

## 0.1.0

The first version.

- One Scan button, then every item with its size, its risk and the reason it can go. Nothing is cleaned before you say so.
- 35 rules in four modules: System (temporary files, error reports, crash dumps, thumbnails, Windows Update caches and old logs, blue screen dumps, the Recycle Bin), Games and launchers (DirectX, NVIDIA, AMD and Intel shader caches, driver installer leftovers, Steam, Epic, Battle.net and Riot), Browsers (Chrome, Edge, Brave, Vivaldi, Opera, Firefox, Zen) and Apps (Discord, VS Code, Slack, Teams).
- Only Safe rules are ticked. A program that is open keeps its files: its rule waits, and says why.
- The Reserve keeps what does not rebuild itself for 7 days (3 to 30), back in one click.
- Documents, Desktop, Pictures, Videos, Music, Downloads, game saves, cloud folders and the Krak library are never touched, wherever Windows put them.
- Signed in-app updates, offered in the middle of the screen at launch.
- English and French.
