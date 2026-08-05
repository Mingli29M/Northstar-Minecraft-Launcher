export type Locale = "en" | "zh" | "de";
export type MessageKey = keyof typeof en;

const en = {
  brand: "Northstar",
  skipToContent: "Skip to content",
  navCapabilities: "Capabilities",
  navCompare: "Compare",
  navDownload: "Download",
  navChangelog: "Changelog",
  navAbout: "About",
  navLicense: "License",
  langLabel: "Language",
  langEn: "English",
  langZh: "中文",
  langDe: "Deutsch",

  heroPill: "v1.1.1 · Host + ReqGuard",
  heroTagline:
    "A desktop Minecraft launcher with Host, ReqGuard, and Modrinth — built to feel like a tool, not a dashboard.",
  heroDownload: "Download latest",
  heroViewSource: "View source",
  heroPlatformsAria: "Supported platforms",
  heroImageAlt: "Northstar overlay shard rising",

  capabilitiesTitle: "Built for launch and host",
  capabilitiesLead:
    "Prism-class instance control, plus Host and ReqGuard — and the rest of what ships in the desktop app.",

  why1Title: "Launch + Host in one app",
  why1Body:
    "Play and run dedicated servers without bouncing between a launcher and a separate host tool. Console, EULA, properties, and port maps stay in the Host tab.",
  why2Title: "Catch broken mods before you boot",
  why2Body:
    "ReqGuard reads mod dependency metadata and surfaces missing libraries (e.g. Fabric API) before Minecraft starts — fewer crash-loop cycles.",
  why3Title: "Desktop-native shell, modern UI kit",
  why3Body:
    "Tauri 2 keeps the shell native; the UI uses Meta Astryx (same design system as the app). No Electron-sized runtime for the window chrome.",

  shotWhy1: "Screenshot placeholder — Host tab",
  shotWhy2: "Screenshot placeholder — ReqGuard",
  shotWhy3: "Screenshot placeholder — desktop shell",
  shotFeat1: "Screenshot placeholder — versions",
  shotFeat2: "Screenshot placeholder — Modrinth",
  shotFeat3: "Screenshot placeholder — instances",
  shotFeat4: "Screenshot placeholder — Host console",
  shotFeat5: "Screenshot placeholder — accounts",
  shotFeat6: "Screenshot placeholder — appearance",

  feat1Title: "Versions & loaders",
  feat1Body:
    "Vanilla, Fabric, Quilt, Forge, NeoForge, Paper/Purpur. Per-instance JVM args and Java detection.",
  feat2Title: "Modpack & mod management",
  feat2Body:
    "Browse and install from Modrinth in-app. Import .mrpack packs and Prism / MultiMC instance folders.",
  feat3Title: "Instance control",
  feat3Body:
    "Isolated versions with their own settings, mods, and configs — switch setups without contaminating worlds.",
  feat4Title: "Dedicated Host",
  feat4Body:
    "Start/stop servers, live console, player lists, file transfer, and UPnP → NAT-PMP → PCP port mapping.",
  feat5Title: "Accounts",
  feat5Body:
    "Microsoft, offline (stable UUIDs), and LittleSkin (authlib-injector) without leaving Launch.",
  feat6Title: "Appearance & locales",
  feat6Body: "Accent, background, font, and UI scale. English, 简体中文, and Deutsch.",

  compareTitle: "How it stacks up",
  compareLeadBefore: "Architecture and measured idle memory versus common launchers. Full protocol:",
  compareLeadAfter: ".",
  measureUnitNote: "Memory columns are in MiB (mebibytes) — not MB, KB, or GB.",
  compareTableCaption: "Feature comparison plus idle memory. Memory rows are MiB.",
  compareAspect: "Aspect",
  compareToolkit: "UI toolkit",
  compareWs: "Idle Working Set",
  comparePrivate: "Idle Private bytes",
  compareUnitMib: "MiB",
  compareHost: "Built-in Host",
  compareReqguard: "ReqGuard precheck",
  compareLicense: "License",
  compareYes: "Yes",
  compareNo: "No",
  compareLimited: "Limited",
  compareDifferent: "Different",
  compareArr: "All Rights Reserved",
  compareBranding: "Branding reserved",
  compareCustomApache: "Custom + Apache",
  compareNa: "N/A",
  compareNaWin: "N/A (Windows-only)",
  compareFootnote:
    "Memory rows: idle UI on Ubuntu 24.04 (Northstar 1.1.0 release, Prism 11.0.3, MultiMC lin64, HMCL 3.16.3). See BENCHMARKS.md for the full protocol.",

  closeTitle: "Get Northstar",
  closeLeadBefore: "Official builds are on",
  closeLeadAfter: ". Open the latest release and pick the asset for your OS.",
  downloadLeadLink: "GitHub Releases",
  downloadOpenLatestBtn: "Open latest release",
  downloadViewGithub: "View on GitHub",
  downloadFootnote:
    "Settings live under %APPDATA%\\euml\\ on Windows (product name Northstar; folder kept for upgrade stability).",

  navConnect: "Connect",
  connectTitle: "Socials & support",
  connectLead: "Follow development and help keep Northstar moving — donation slots are ready when pages go live.",
  connectSocials: "Socials",
  connectDonate: "Donate",
  connectDonateNote: "Optional. Donations support development time; they are not required to use the launcher.",
  linkAfdian: "Afdian (爱发电)",
  linkSoon: "coming soon",
  linkSoonHint: "URL not published yet — slot reserved",

  footerColSite: "Site",
  footerColSiteLink: "Home on Pages",
  footerColLegal: "Legal",
  footerRights: "© 2026 Northstar contributors. All rights reserved.",
  footerDownloads: "Downloads: GitHub Releases (not hosted on Pages).",
  footerDisclaimer:
    "Not affiliated with Mojang Studios or Microsoft. “Minecraft” is a trademark of Mojang Synergies AB. Mentions of Prism, MultiMC, PCL, and HMCL are for comparison only.",
  footerLicense: "License",
  footerChangelog: "Changelog",
  footerChangelogMd: "website/CHANGELOG.md",
  footerGithub: "GitHub",
  footerLicenseFile: "LICENSE file",

  aboutTitle: "About",
  aboutLead: "Product background and how Northstar relates to other launchers.",
  aboutWhatTitle: "What is Northstar?",
  aboutWhatBody:
    "Northstar is a proprietary desktop Minecraft launcher (early development also used the name EUML). It targets a PCL / HMCL–style launch flow with Tauri 2 packaging and a Meta Astryx UI — plus built-in Host and ReqGuard.",
  aboutIndepTitle: "Independent project",
  aboutIndepBody:
    "Inspired by workflows from Prism, MultiMC, and PCL-class launchers, but not a fork of those projects and not affiliated with them.",
  aboutResTitle: "Resource usage (same-OS sample)",
  aboutResBody:
    "Idle UI after 30s on Ubuntu 24.04 x86_64 (2026-08-04). Working Set ≈ Linux RSS; Private Bytes ≈ Private_Dirty + Private_Clean from smaps_rollup. Full methodology:",
  aboutColLauncher: "Launcher",
  aboutColWs: "Working Set (MiB)",
  aboutColPrivate: "Private (MiB)",
  aboutColCpu: "CPU %",
  aboutColNotes: "Notes",
  aboutNotePcl: "Windows-only; not measured",
  aboutResOutro:
    "After WebKit compositor/DMABUF defaults and frontend lazy-loading, Northstar idle private bytes (~100 MiB) undercut HMCL; Working Set still trails Qt (Prism/MultiMC) because of the WebView process model. With vanilla 1.21.11, the game dominates totals (~1.5–1.7 GiB). See BENCHMARKS.md.",
  aboutUnofficialTitle: "Unofficial software",
  aboutUnofficialBody:
    "Not an official Minecraft product. Not approved by or associated with Mojang Studios or Microsoft.",
  aboutBtnGithub: "GitHub",
  aboutBtnChangelog: "Changelog",

  licenseTitle: "License",
  licenseLead:
    "All rights reserved — ownership, branding, third-party deps, and Minecraft trademark notices.",
  licenseBannerTitle: "All rights reserved",
  licenseBannerBody:
    "No open-source license is granted. Viewing the repository does not give rights to copy, modify, redistribute, or rebrand Northstar.",
  licenseDocTitle: "LICENSE",
  licenseViewGithub: "View on GitHub",
  licenseBrandingTitle: "Branding",
  licenseBrandingBody:
    "The Northstar name and installer branding are reserved for official builds from this repository (same idea as MultiMC’s branding reservation).",
  licenseThirdTitle: "Third-party & Minecraft",
  licenseThirdBody:
    "Dependencies keep their own licenses. Minecraft is a trademark of Mojang Synergies AB; you need a legitimate game copy to play.",
  licenseBindingNote: "The binding legal text is English.",

  changelogTitle: "Changelog",
  changelogLeadBefore: "What’s new in Northstar",
  changelogLeadAfter:
    ". Website and launcher notes are split into two columns below, and live as separate files in the repository.",
  changelogCurrent: "Current: v{version}",
  changelogBannerBody:
    "Website and launcher changelogs are shown side by side. Installers ship from GitHub Releases.",
  changelogColWebsite: "Website",
  changelogColLauncher: "Launcher",
  changelogViewWebsiteMd: "View website/CHANGELOG.md",
  changelogViewLauncherMd: "View CHANGELOG.md",

  cl111Summary: "New Northstar brand marks on the site; website and launcher changelogs split.",
  cl111Sec1: "Branding",
  cl111Sec1I1:
    "Replaced nether-star UI marks and hero/background art with the new Northstar star and overlay shard",
  cl111Sec1I2: "Browser tab favicon (nether-star-16.png) left unchanged",
  cl111Sec2: "Docs",
  cl111Sec2I1:
    "Website and launcher changelogs are separate files, shown together on this page",

  cl110Summary: "Marketing site launched on GitHub Pages.",
  cl110Sec1: "Website",
  cl110Sec1I1: "Landing page under website/ deployed via GitHub Pages",
  cl110Sec1I2: "Centered hero, frosted nav, and merged capabilities sections",

  cl100Summary: "Initial public marketing site alongside the 1.0.0 launcher release.",
  cl100Sec1: "Website",
  cl100Sec1I1: "Initial public marketing site scaffolding",

  lcl111Summary: "New Northstar app icons; launcher changelog split from the website file.",
  lcl111Sec1: "Branding",
  lcl111Sec1I1: "Replaced nether-star window/installer icons with the new Northstar mark",
  lcl111Sec1I2: "Overlay shard mark for taller brand visuals",
  lcl111Sec2: "Docs",
  lcl111Sec2I1: "Launcher and website changelogs are separate files (CHANGELOG.md vs website/CHANGELOG.md)",

  lcl110Summary: "Northstar display rebrand, appearance settings, and Host/network polish.",
  lcl110Sec1: "Branding",
  lcl110Sec1I1:
    "User-facing product name is Northstar (data folder remains %APPDATA%\\euml for install stability)",
  lcl110Sec1I2: "Window title, User-Agent, console titles, and Host MOTD/strings updated",
  lcl110Sec2: "Appearance",
  lcl110Sec2I1: "Settings → Appearance: accent color, background color/image, font family, UI scale",
  lcl110Sec2I2: "Live CSS preview; persisted in settings.json",
  lcl110Sec3: "Host & network",
  lcl110Sec3I1: "UPnP → NAT-PMP → PCP port-map cascade with clearer join addresses",
  lcl110Sec3I2: "Orphan Java reattach, port-in-use detection, and Host KeepAlive route fix",

  lcl100Summary:
    "First public release of Northstar — desktop Minecraft launcher with Host, Modrinth, and multi-account support.",
  lcl100Sec1: "Highlights",
  lcl100Sec1I1:
    "Launch, versions/loaders, Modrinth, ReqGuard, Host, accounts (Microsoft / offline / LittleSkin)",
  lcl100Sec1I2: "Native installers for Windows, macOS, and Linux via GitHub Actions",

  lcl010Summary: "Internal preview that established the core launcher shell and CI publish path.",
  lcl010Sec1: "Added",
  lcl010Sec1I1: "Launch screen, news, Settings → About changelog",
  lcl010Sec1I2: "ReqGuard, Modrinth, LittleSkin, multi-language scaffolding",
};

const zh: Record<MessageKey, string> = {
  brand: "Northstar",
  skipToContent: "跳到正文",
  navCapabilities: "能力",
  navCompare: "对比",
  navDownload: "下载",
  navChangelog: "更新日志",
  navAbout: "关于",
  navLicense: "许可",
  langLabel: "语言",
  langEn: "English",
  langZh: "中文",
  langDe: "Deutsch",

  heroPill: "v1.1.1 · Host + ReqGuard",
  heroTagline:
    "桌面端 Minecraft 启动器，内置 Host、ReqGuard 与 Modrinth —— 像工具，而不像仪表盘。",
  heroDownload: "下载最新版",
  heroViewSource: "查看源码",
  heroPlatformsAria: "支持的平台",
  heroImageAlt: "Northstar 叠加碎片升空",

  capabilitiesTitle: "为启动与开服而建",
  capabilitiesLead: "Prism 级实例管理，加上 Host 与 ReqGuard —— 以及桌面应用已交付的其余能力。",

  why1Title: "启动与开服一体",
  why1Body:
    "不必在启动器与独立开服工具之间来回切换。控制台、EULA、配置与端口映射都在 Host 页完成。",
  why2Title: "启动前发现缺依赖",
  why2Body:
    "ReqGuard 读取模组依赖元数据，在启动前标出缺失库（如 Fabric API），减少反复崩溃。",
  why3Title: "原生外壳 + 现代 UI",
  why3Body:
    "Tauri 2 提供原生外壳；界面使用 Meta Astryx（与应用相同的设计系统）。窗口层不必背负 Electron 级体积。",

  shotWhy1: "截图占位 — Host 页",
  shotWhy2: "截图占位 — ReqGuard",
  shotWhy3: "截图占位 — 桌面外壳",
  shotFeat1: "截图占位 — 版本",
  shotFeat2: "截图占位 — Modrinth",
  shotFeat3: "截图占位 — 实例",
  shotFeat4: "截图占位 — Host 控制台",
  shotFeat5: "截图占位 — 账户",
  shotFeat6: "截图占位 — 外观",

  feat1Title: "版本与加载器",
  feat1Body: "Vanilla、Fabric、Quilt、Forge、NeoForge、Paper/Purpur。按实例配置 JVM 与 Java 检测。",
  feat2Title: "整合包与模组",
  feat2Body: "应用内浏览安装 Modrinth。支持导入 .mrpack 以及 Prism / MultiMC 实例文件夹。",
  feat3Title: "实例管理",
  feat3Body: "各版本独立设置、模组与配置 —— 切换方案而不污染存档。",
  feat4Title: "专用 Host",
  feat4Body: "启停服务器、实时控制台、玩家列表、文件传输，以及 UPnP → NAT-PMP → PCP 端口映射。",
  feat5Title: "账户",
  feat5Body: "微软、离线（稳定 UUID）与 LittleSkin（authlib-injector），无需离开启动页。",
  feat6Title: "外观与语言",
  feat6Body: "强调色、背景、字体与界面缩放。支持 English、简体中文、Deutsch。",

  compareTitle: "对比一览",
  compareLeadBefore: "与常见启动器的架构及空闲内存对比。完整协议见：",
  compareLeadAfter: "。",
  measureUnitNote: "内存列单位为 MiB（兆字节 / mebibyte），不是 MB、KB 或 GB。",
  compareTableCaption: "功能对比与空闲内存。内存行为 MiB。",
  compareAspect: "项目",
  compareToolkit: "UI 工具包",
  compareWs: "空闲 Working Set",
  comparePrivate: "空闲 Private 字节",
  compareUnitMib: "MiB",
  compareHost: "内置 Host",
  compareReqguard: "ReqGuard 预检",
  compareLicense: "许可",
  compareYes: "是",
  compareNo: "否",
  compareLimited: "有限",
  compareDifferent: "不同实现",
  compareArr: "保留所有权利",
  compareBranding: "品牌保留",
  compareCustomApache: "自定义 + Apache",
  compareNa: "不适用",
  compareNaWin: "不适用（仅 Windows）",
  compareFootnote:
    "内存行：Ubuntu 24.04 上空闲界面（Northstar 1.1.0 发布版、Prism 11.0.3、MultiMC lin64、HMCL 3.16.3）。完整协议见 BENCHMARKS.md。",

  closeTitle: "获取 Northstar",
  closeLeadBefore: "官方安装包在",
  closeLeadAfter: "。打开最新 Release，选择对应系统资源即可。",
  downloadLeadLink: "GitHub Releases",
  downloadOpenLatestBtn: "打开最新 Release",
  downloadViewGithub: "在 GitHub 查看",
  downloadFootnote:
    "Windows 设置位于 %APPDATA%\\euml\\（产品名 Northstar；目录名为升级兼容而保留）。",

  navConnect: "关注与支持",
  connectTitle: "社交与支持",
  connectLead: "关注开发进展，并在页面就绪后通过捐赠支持 Northstar。",
  connectSocials: "社交",
  connectDonate: "捐赠",
  connectDonateNote: "自愿支持。捐赠用于开发时间；使用启动器无需付费。",
  linkAfdian: "爱发电（Afdian）",
  linkSoon: "即将开放",
  linkSoonHint: "链接尚未公布 — 已预留入口",

  footerColSite: "网站",
  footerColSiteLink: "Pages 首页",
  footerColLegal: "法律",
  footerRights: "© 2026 Northstar 贡献者。保留所有权利。",
  footerDownloads: "下载：GitHub Releases（不在 Pages 托管）。",
  footerDisclaimer:
    "与 Mojang Studios 或 Microsoft 无关。“Minecraft”是 Mojang Synergies AB 的商标。文中提及 Prism、MultiMC、PCL、HMCL 仅供对比。",
  footerLicense: "许可",
  footerChangelog: "更新日志",
  footerChangelogMd: "website/CHANGELOG.md",
  footerGithub: "GitHub",
  footerLicenseFile: "LICENSE 文件",

  aboutTitle: "关于",
  aboutLead: "产品背景，以及 Northstar 与其他启动器的关系。",
  aboutWhatTitle: "Northstar 是什么？",
  aboutWhatBody:
    "Northstar 是专有桌面 Minecraft 启动器（早期开发亦称 EUML）。面向 PCL / HMCL 风格启动流，采用 Tauri 2 与 Meta Astryx UI，并内置 Host 与 ReqGuard。",
  aboutIndepTitle: "独立项目",
  aboutIndepBody:
    "灵感来自 Prism、MultiMC、PCL 等工作流，但并非这些项目的分支，也无隶属关系。",
  aboutResTitle: "资源占用（同系统采样）",
  aboutResBody:
    "Ubuntu 24.04 x86_64（2026-08-04）空闲界面 30 秒后。Working Set ≈ Linux RSS；Private Bytes ≈ smaps_rollup 的 Private_Dirty + Private_Clean。完整方法见：",
  aboutColLauncher: "启动器",
  aboutColWs: "Working Set（MiB）",
  aboutColPrivate: "Private（MiB）",
  aboutColCpu: "CPU %",
  aboutColNotes: "备注",
  aboutNotePcl: "仅 Windows；未测量",
  aboutResOutro:
    "经 WebKit 合成器/DMABUF 与前端懒加载优化后，Northstar 空闲私有字节约 100 MiB，低于 HMCL；Working Set 仍高于 Qt（Prism/MultiMC），因其 WebView 多进程模型。原版 1.21.11 运行时游戏占主导（约 1.5–1.7 GiB）。详见 BENCHMARKS.md。",
  aboutUnofficialTitle: "非官方软件",
  aboutUnofficialBody: "非 Minecraft 官方产品。未经 Mojang Studios 或 Microsoft 批准或关联。",
  aboutBtnGithub: "GitHub",
  aboutBtnChangelog: "更新日志",

  licenseTitle: "许可",
  licenseLead: "保留所有权利 —— 所有权、品牌、第三方依赖与 Minecraft 商标说明。",
  licenseBannerTitle: "保留所有权利",
  licenseBannerBody:
    "未授予开源许可。浏览仓库并不意味着可以复制、修改、再分发或重新品牌化 Northstar。",
  licenseDocTitle: "LICENSE",
  licenseViewGithub: "在 GitHub 查看",
  licenseBrandingTitle: "品牌",
  licenseBrandingBody:
    "Northstar 名称与安装程序品牌仅保留给本仓库官方构建（类似 MultiMC 的品牌保留）。",
  licenseThirdTitle: "第三方与 Minecraft",
  licenseThirdBody:
    "依赖库遵循各自许可。Minecraft 是 Mojang Synergies AB 的商标；游玩需拥有正版游戏。",
  licenseBindingNote: "具有约束力的法律文本以英文为准。",

  changelogTitle: "更新日志",
  changelogLeadBefore: "Northstar",
  changelogLeadAfter: " 的新内容。网站与启动器说明分左右两栏，并在仓库中分文件维护。",
  changelogCurrent: "当前：v{version}",
  changelogBannerBody: "网站与启动器更新日志并排显示。安装包发布于 GitHub Releases。",
  changelogColWebsite: "网站",
  changelogColLauncher: "启动器",
  changelogViewWebsiteMd: "查看 website/CHANGELOG.md",
  changelogViewLauncherMd: "查看 CHANGELOG.md",

  cl111Summary: "网站换上新的 Northstar 品牌标识；网站与启动器更新日志拆分。",
  cl111Sec1: "品牌",
  cl111Sec1I1: "下界之星 UI 标识与英雄/背景图替换为新的 Northstar 星标与叠加碎片",
  cl111Sec1I2: "浏览器标签页图标（nether-star-16.png）保持不变",
  cl111Sec2: "文档",
  cl111Sec2I1: "网站与启动器更新日志分文件维护，并在本页并排展示",

  cl110Summary: "营销站经 GitHub Pages 上线。",
  cl110Sec1: "网站",
  cl110Sec1I1: "website/ 下的落地页，经 GitHub Pages 部署",
  cl110Sec1I2: "居中英雄区、磨砂导航与合并的能力区",

  cl100Summary: "与 1.0.0 启动器发布同期的首个公开营销站。",
  cl100Sec1: "网站",
  cl100Sec1I1: "初始公开营销站脚手架",

  lcl111Summary: "新的 Northstar 应用图标；启动器更新日志与网站文件拆分。",
  lcl111Sec1: "品牌",
  lcl111Sec1I1: "窗口/安装程序图标由下界之星替换为新的 Northstar 标识",
  lcl111Sec1I2: "叠加碎片标识用于更高的品牌视觉",
  lcl111Sec2: "文档",
  lcl111Sec2I1: "启动器与网站更新日志分文件（CHANGELOG.md 与 website/CHANGELOG.md）",

  lcl110Summary: "Northstar 品牌展示、外观设置与 Host/网络打磨。",
  lcl110Sec1: "品牌",
  lcl110Sec1I1: "面向用户的产品名为 Northstar（数据目录仍为 %APPDATA%\\euml，保证安装稳定）",
  lcl110Sec1I2: "更新窗口标题、User-Agent、控制台标题与 Host MOTD/文案",
  lcl110Sec2: "外观",
  lcl110Sec2I1: "设置 → 外观：强调色、背景色/图、字体、界面缩放",
  lcl110Sec2I2: "实时 CSS 预览；持久化到 settings.json",
  lcl110Sec3: "Host 与网络",
  lcl110Sec3I1: "UPnP → NAT-PMP → PCP 端口映射级联，更清晰的加入地址",
  lcl110Sec3I2: "孤儿 Java 重连、端口占用检测、Host KeepAlive 路由修复",

  lcl100Summary: "Northstar 首次公开发布 —— 桌面 Minecraft 启动器，含 Host、Modrinth 与多账户。",
  lcl100Sec1: "亮点",
  lcl100Sec1I1: "启动、版本/加载器、Modrinth、ReqGuard、Host、账户（微软 / 离线 / LittleSkin）",
  lcl100Sec1I2: "经 GitHub Actions 提供 Windows、macOS、Linux 原生安装包",

  lcl010Summary: "内部预览，奠定核心启动器外壳与 CI 发布路径。",
  lcl010Sec1: "新增",
  lcl010Sec1I1: "启动页、新闻、设置 → 关于中的更新日志",
  lcl010Sec1I2: "ReqGuard、Modrinth、LittleSkin、多语言脚手架",
};

const de: Record<MessageKey, string> = {
  brand: "Northstar",
  skipToContent: "Zum Inhalt springen",
  navCapabilities: "Fähigkeiten",
  navCompare: "Vergleich",
  navDownload: "Download",
  navChangelog: "Änderungen",
  navAbout: "Über",
  navLicense: "Lizenz",
  langLabel: "Sprache",
  langEn: "English",
  langZh: "中文",
  langDe: "Deutsch",

  heroPill: "v1.1.1 · Host + ReqGuard",
  heroTagline:
    "Ein Desktop-Minecraft-Launcher mit Host, ReqGuard und Modrinth — wie ein Werkzeug, kein Dashboard.",
  heroDownload: "Neueste Version laden",
  heroViewSource: "Quellcode ansehen",
  heroPlatformsAria: "Unterstützte Plattformen",
  heroImageAlt: "Northstar-Overlay-Kristall steigt auf",

  capabilitiesTitle: "Für Start und Host gebaut",
  capabilitiesLead:
    "Instanzverwaltung auf Prism-Niveau, plus Host und ReqGuard — und der Rest der Desktop-App.",

  why1Title: "Starten + Hosten in einer App",
  why1Body:
    "Kein Wechsel zwischen Launcher und separatem Host-Tool. Konsole, EULA, Properties und Port-Maps bleiben im Host-Tab.",
  why2Title: "Kaputte Mods vor dem Start erkennen",
  why2Body:
    "ReqGuard liest Mod-Abhängigkeiten und zeigt fehlende Bibliotheken (z. B. Fabric API) vor dem Start — weniger Crash-Schleifen.",
  why3Title: "Nativer Shell, modernes UI-Kit",
  why3Body:
    "Tauri 2 hält die Shell nativ; die UI nutzt Meta Astryx (dasselbe Designsystem wie die App). Kein Electron-großer Fenster-Runtime.",

  shotWhy1: "Screenshot-Platzhalter — Host-Tab",
  shotWhy2: "Screenshot-Platzhalter — ReqGuard",
  shotWhy3: "Screenshot-Platzhalter — Desktop-Shell",
  shotFeat1: "Screenshot-Platzhalter — Versionen",
  shotFeat2: "Screenshot-Platzhalter — Modrinth",
  shotFeat3: "Screenshot-Platzhalter — Instanzen",
  shotFeat4: "Screenshot-Platzhalter — Host-Konsole",
  shotFeat5: "Screenshot-Platzhalter — Konten",
  shotFeat6: "Screenshot-Platzhalter — Erscheinungsbild",

  feat1Title: "Versionen & Loader",
  feat1Body:
    "Vanilla, Fabric, Quilt, Forge, NeoForge, Paper/Purpur. JVM-Args und Java-Erkennung pro Instanz.",
  feat2Title: "Modpacks & Mods",
  feat2Body:
    "Modrinth in der App durchsuchen und installieren. Import von .mrpack sowie Prism-/MultiMC-Ordnern.",
  feat3Title: "Instanzverwaltung",
  feat3Body:
    "Getrennte Versionen mit eigenen Einstellungen, Mods und Configs — Setups wechseln ohne Welten zu vermischen.",
  feat4Title: "Dedizierter Host",
  feat4Body:
    "Server starten/stoppen, Live-Konsole, Spielerlisten, Dateitransfer und UPnP → NAT-PMP → PCP.",
  feat5Title: "Konten",
  feat5Body:
    "Microsoft, Offline (stabile UUIDs) und LittleSkin (authlib-injector) ohne die Startseite zu verlassen.",
  feat6Title: "Erscheinungsbild & Sprachen",
  feat6Body: "Akzent, Hintergrund, Schrift und UI-Skalierung. English, 简体中文 und Deutsch.",

  compareTitle: "Im Vergleich",
  compareLeadBefore:
    "Architektur und gemessener Idle-Speicher gegenüber gängigen Launchern. Protokoll:",
  compareLeadAfter: ".",
  measureUnitNote: "Speicher-Spalten in MiB (Mebibyte) — nicht MB, KB oder GB.",
  compareTableCaption: "Feature-Vergleich plus Idle-Speicher. Speicherzeilen in MiB.",
  compareAspect: "Aspekt",
  compareToolkit: "UI-Toolkit",
  compareWs: "Idle Working Set",
  comparePrivate: "Idle Private Bytes",
  compareUnitMib: "MiB",
  compareHost: "Eingebauter Host",
  compareReqguard: "ReqGuard-Vorprüfung",
  compareLicense: "Lizenz",
  compareYes: "Ja",
  compareNo: "Nein",
  compareLimited: "Begrenzt",
  compareDifferent: "Anders",
  compareArr: "Alle Rechte vorbehalten",
  compareBranding: "Branding vorbehalten",
  compareCustomApache: "Custom + Apache",
  compareNa: "k. A.",
  compareNaWin: "k. A. (nur Windows)",
  compareFootnote:
    "Speicherzeilen: Idle-UI unter Ubuntu 24.04 (Northstar 1.1.0 Release, Prism 11.0.3, MultiMC lin64, HMCL 3.16.3). Siehe BENCHMARKS.md.",

  closeTitle: "Hol dir Northstar",
  closeLeadBefore: "Offizielle Builds liegen auf",
  closeLeadAfter: ". Öffne das neueste Release und wähle das Asset für dein OS.",
  downloadLeadLink: "GitHub Releases",
  downloadOpenLatestBtn: "Neues Release öffnen",
  downloadViewGithub: "Auf GitHub ansehen",
  downloadFootnote:
    "Einstellungen unter %APPDATA%\\euml\\ auf Windows (Produktname Northstar; Ordnername für Upgrade-Stabilität).",

  navConnect: "Connect",
  connectTitle: "Soziales & Unterstützung",
  connectLead:
    "Entwicklung folgen und Northstar unterstützen — Spenden-Slots sind vorbereitet, sobald Seiten live sind.",
  connectSocials: "Soziales",
  connectDonate: "Spenden",
  connectDonateNote:
    "Freiwillig. Spenden unterstützen Entwicklungszeit; der Launcher bleibt ohne Zahlung nutzbar.",
  linkAfdian: "Afdian (爱发电)",
  linkSoon: "demnächst",
  linkSoonHint: "URL noch nicht veröffentlicht — Slot reserviert",

  footerColSite: "Website",
  footerColSiteLink: "Start auf Pages",
  footerColLegal: "Rechtliches",
  footerRights: "© 2026 Northstar-Mitwirkende. Alle Rechte vorbehalten.",
  footerDownloads: "Downloads: GitHub Releases (nicht auf Pages gehostet).",
  footerDisclaimer:
    "Nicht verbunden mit Mojang Studios oder Microsoft. „Minecraft“ ist eine Marke von Mojang Synergies AB. Erwähnungen von Prism, MultiMC, PCL und HMCL dienen nur dem Vergleich.",
  footerLicense: "Lizenz",
  footerChangelog: "Änderungen",
  footerChangelogMd: "website/CHANGELOG.md",
  footerGithub: "GitHub",
  footerLicenseFile: "LICENSE-Datei",

  aboutTitle: "Über",
  aboutLead: "Produkthintergrund und Beziehung zu anderen Launchern.",
  aboutWhatTitle: "Was ist Northstar?",
  aboutWhatBody:
    "Northstar ist ein proprietärer Desktop-Minecraft-Launcher (Frühphase auch EUML genannt). PCL-/HMCL-artiger Startfluss mit Tauri 2 und Meta-Astryx-UI — plus Host und ReqGuard.",
  aboutIndepTitle: "Unabhängiges Projekt",
  aboutIndepBody:
    "Inspiriert von Prism-, MultiMC- und PCL-Workflows, aber kein Fork und keine Zugehörigkeit zu diesen Projekten.",
  aboutResTitle: "Ressourcenverbrauch (gleiche OS-Stichprobe)",
  aboutResBody:
    "Idle-UI nach 30s unter Ubuntu 24.04 x86_64 (2026-08-04). Working Set ≈ Linux-RSS; Private Bytes ≈ Private_Dirty + Private_Clean aus smaps_rollup. Methodik:",
  aboutColLauncher: "Launcher",
  aboutColWs: "Working Set (MiB)",
  aboutColPrivate: "Private (MiB)",
  aboutColCpu: "CPU %",
  aboutColNotes: "Notizen",
  aboutNotePcl: "Nur Windows; nicht gemessen",
  aboutResOutro:
    "Nach WebKit-Compositor/DMABUF und Frontend-Lazy-Loading unterbieten Northstars Idle-Private-Bytes (~100 MiB) HMCL; Working Set liegt weiterhin hinter Qt (Prism/MultiMC) wegen des WebView-Prozessmodells. Mit Vanilla 1.21.11 dominiert das Spiel (~1,5–1,7 GiB). Siehe BENCHMARKS.md.",
  aboutUnofficialTitle: "Inoffizielle Software",
  aboutUnofficialBody:
    "Kein offizielles Minecraft-Produkt. Nicht von Mojang Studios oder Microsoft genehmigt oder damit verbunden.",
  aboutBtnGithub: "GitHub",
  aboutBtnChangelog: "Änderungen",

  licenseTitle: "Lizenz",
  licenseLead:
    "Alle Rechte vorbehalten — Eigentum, Branding, Drittanbieter-Deps und Minecraft-Markenhinweise.",
  licenseBannerTitle: "Alle Rechte vorbehalten",
  licenseBannerBody:
    "Keine Open-Source-Lizenz. Das Ansehen des Repos gewährt keine Rechte zum Kopieren, Ändern, Weitergeben oder Rebranding von Northstar.",
  licenseDocTitle: "LICENSE",
  licenseViewGithub: "Auf GitHub ansehen",
  licenseBrandingTitle: "Branding",
  licenseBrandingBody:
    "Name und Installer-Branding von Northstar sind offiziellen Builds aus diesem Repo vorbehalten (ähnlich MultiMCs Branding-Vorbehalt).",
  licenseThirdTitle: "Drittanbieter & Minecraft",
  licenseThirdBody:
    "Abhängigkeiten behalten ihre Lizenzen. Minecraft ist eine Marke von Mojang Synergies AB; zum Spielen brauchst du eine legitime Kopie.",
  licenseBindingNote: "Der verbindliche Rechtstext ist Englisch.",

  changelogTitle: "Änderungsprotokoll",
  changelogLeadBefore: "Was neu ist in Northstar",
  changelogLeadAfter:
    ". Website- und Launcher-Notizen stehen unten in zwei Spalten und als getrennte Dateien im Repository.",
  changelogCurrent: "Aktuell: v{version}",
  changelogBannerBody:
    "Website- und Launcher-Changelogs werden nebeneinander angezeigt. Installer kommen von GitHub Releases.",
  changelogColWebsite: "Website",
  changelogColLauncher: "Launcher",
  changelogViewWebsiteMd: "website/CHANGELOG.md ansehen",
  changelogViewLauncherMd: "CHANGELOG.md ansehen",

  cl111Summary: "Neue Northstar-Markenicons auf der Site; Website- und Launcher-Changelogs getrennt.",
  cl111Sec1: "Branding",
  cl111Sec1I1:
    "Netherstern-UI-Marken und Hero-/Hintergrundgrafiken durch den neuen Northstar-Stern und Overlay-Kristall ersetzt",
  cl111Sec1I2: "Browser-Tab-Favicon (nether-star-16.png) unverändert gelassen",
  cl111Sec2: "Docs",
  cl111Sec2I1:
    "Website- und Launcher-Changelogs sind getrennte Dateien und werden hier gemeinsam angezeigt",

  cl110Summary: "Marketingseite über GitHub Pages gestartet.",
  cl110Sec1: "Website",
  cl110Sec1I1: "Landing unter website/, bereitgestellt über GitHub Pages",
  cl110Sec1I2: "Zentrierter Hero, frosted Nav und zusammengeführte Capabilities",

  cl100Summary: "Erste öffentliche Marketingseite zusammen mit dem Launcher-Release 1.0.0.",
  cl100Sec1: "Website",
  cl100Sec1I1: "Initiales öffentliches Marketing-Site-Gerüst",

  lcl111Summary: "Neue Northstar-App-Icons; Launcher-Changelog von der Website-Datei getrennt.",
  lcl111Sec1: "Branding",
  lcl111Sec1I1: "Netherstern-Fenster-/Installer-Icons durch das neue Northstar-Mark ersetzt",
  lcl111Sec1I2: "Overlay-Kristall-Mark für höhere Markenvisuals",
  lcl111Sec2: "Docs",
  lcl111Sec2I1: "Launcher- und Website-Changelogs sind getrennte Dateien (CHANGELOG.md vs website/CHANGELOG.md)",

  lcl110Summary: "Northstar-Anzeige-Rebrand, Erscheinungsbild-Einstellungen und Host-/Netzwerk-Politur.",
  lcl110Sec1: "Branding",
  lcl110Sec1I1:
    "Nutzerseitiger Produktname Northstar (Datenordner bleibt %APPDATA%\\euml für Upgrade-Stabilität)",
  lcl110Sec1I2: "Fenstertitel, User-Agent, Konsoltitel und Host-MOTD/Texte aktualisiert",
  lcl110Sec2: "Erscheinungsbild",
  lcl110Sec2I1: "Einstellungen → Erscheinungsbild: Akzent, Hintergrundfarbe/-bild, Schrift, UI-Skalierung",
  lcl110Sec2I2: "Live-CSS-Vorschau; in settings.json gespeichert",
  lcl110Sec3: "Host & Netzwerk",
  lcl110Sec3I1: "UPnP → NAT-PMP → PCP Port-Map-Kaskade mit klareren Join-Adressen",
  lcl110Sec3I2: "Verwaiste Java-Reconnects, Port-belegt-Erkennung, Host-KeepAlive-Routenfix",

  lcl100Summary:
    "Erste öffentliche Northstar-Veröffentlichung — Desktop-Minecraft-Launcher mit Host, Modrinth und Multi-Account.",
  lcl100Sec1: "Highlights",
  lcl100Sec1I1:
    "Start, Versionen/Loader, Modrinth, ReqGuard, Host, Konten (Microsoft / Offline / LittleSkin)",
  lcl100Sec1I2: "Native Installer für Windows, macOS und Linux über GitHub Actions",

  lcl010Summary: "Interne Vorschau, die die Kern-Launcher-Shell und den CI-Publish-Pfad etablierte.",
  lcl010Sec1: "Hinzugefügt",
  lcl010Sec1I1: "Startbildschirm, News, Einstellungen → Über mit Changelog",
  lcl010Sec1I2: "ReqGuard, Modrinth, LittleSkin, Mehrsprachen-Gerüst",
};

export const dictionaries = { en, zh, de } as const;

export const LOCALE_STORAGE_KEY = "northstar-site-locale";
