import { useMemo } from "react";
import { useLocation } from "react-router-dom";
import { AppShell } from "@astryxdesign/core/AppShell";
import { useI18n } from "./i18n";
import { AppNav } from "./components/AppNav";
import { DownloadStatusBar } from "./components/DownloadStatusBar";
import { KeepAliveRoutes } from "./components/KeepAliveRoutes";
import { LaunchPage } from "./pages/LaunchPage";
import { DownloadPage } from "./pages/DownloadPage";
import { ModrinthDetailPage } from "./pages/ModrinthDetailPage";
import { NewsPage } from "./pages/NewsPage";
import { VersionsPage } from "./pages/VersionsPage";
import { ServersPage } from "./pages/ServersPage";
import { HostPage } from "./pages/HostPage";
import { AccountsPage } from "./pages/AccountsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { useDownloadStatus } from "./lib/downloadStatus";
import { Button } from "@astryxdesign/core/Button";

function ConsoleNavButton() {
  const { t } = useI18n();
  const { consoleOpen, setConsoleOpen, consoleDetached, openConsoleWindow } = useDownloadStatus();
  return (
    <div style={{ padding: "8px 12px" }}>
      <Button
        size="sm"
        width="100%"
        label={consoleDetached ? t("consoleFocus") : consoleOpen ? t("hideConsole") : t("showConsole")}
        onClick={() => {
          if (consoleDetached) void openConsoleWindow();
          else setConsoleOpen(!consoleOpen);
        }}
      />
    </div>
  );
}

export default function App() {
  const { t } = useI18n();
  const { pathname } = useLocation();

  const labels = useMemo(
    () => ({
      launch: t("navLaunch"),
      download: t("navDownload"),
      news: t("navNews"),
      versions: t("navVersions"),
      servers: t("navServers"),
      host: t("navHost"),
      accounts: t("navAccounts"),
      settings: t("navSettings"),
    }),
    [t],
  );

  const sideNav = useMemo(
    () => (
      <>
        <AppNav appName={t("appName")} labels={labels} pathname={pathname} />
        <ConsoleNavButton />
      </>
    ),
    [t, labels, pathname],
  );

  const panes = useMemo(
    () => [
      { id: "launch", match: (p: string) => p === "/", element: <LaunchPage /> },
      {
        id: "mod-detail",
        match: (p: string) => p.startsWith("/download/mod/"),
        element: <ModrinthDetailPage />,
      },
      {
        id: "download",
        match: (p: string) => p.startsWith("/download") && !p.startsWith("/download/mod/"),
        element: <DownloadPage />,
      },
      {
        id: "news",
        match: (p: string) => p.startsWith("/news"),
        element: <NewsPage />,
      },
      {
        id: "versions",
        match: (p: string) => p.startsWith("/versions"),
        element: <VersionsPage />,
      },
      {
        id: "servers",
        match: (p: string) => p.startsWith("/servers"),
        element: <ServersPage />,
      },
      {
        id: "host",
        match: (p: string) => p.startsWith("/host"),
        element: <HostPage />,
      },
      {
        id: "accounts",
        match: (p: string) => p.startsWith("/accounts"),
        element: <AccountsPage />,
      },
      {
        id: "settings",
        match: (p: string) => p.startsWith("/settings"),
        element: <SettingsPage />,
      },
    ],
    [],
  );

  return (
    <>
      <AppShell
        variant="elevated"
        height="auto"
        contentPadding={5}
        sideNav={sideNav}
        mobileNav={{ breakpoint: "none" }}
      >
        <KeepAliveRoutes panes={panes} />
      </AppShell>
      <DownloadStatusBar />
    </>
  );
}
