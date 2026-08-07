import { lazy, Suspense, useMemo, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { AppShell } from "@astryxdesign/core/AppShell";
import { Button } from "@astryxdesign/core/Button";
import { Spinner } from "@astryxdesign/core/Spinner";
import { useI18n } from "./i18n";
import { AppNav } from "./components/AppNav";
import { DownloadStatusBar } from "./components/DownloadStatusBar";
import { ExitGuard } from "./components/ExitGuard";
import { KeepAliveRoutes } from "./components/KeepAliveRoutes";
import { LaunchPage } from "./pages/LaunchPage";
import { useDownloadStatus } from "./lib/downloadStatus";

// Heavy pages stay out of the cold-start JS/JIT heap until first visit.
const DownloadPage = lazy(() =>
  import("./pages/DownloadPage").then((m) => ({ default: m.DownloadPage })),
);
const ModrinthDetailPage = lazy(() =>
  import("./pages/ModrinthDetailPage").then((m) => ({ default: m.ModrinthDetailPage })),
);
const NewsPage = lazy(() => import("./pages/NewsPage").then((m) => ({ default: m.NewsPage })));
const VersionsPage = lazy(() =>
  import("./pages/VersionsPage").then((m) => ({ default: m.VersionsPage })),
);
const ServersPage = lazy(() =>
  import("./pages/ServersPage").then((m) => ({ default: m.ServersPage })),
);
const HostPage = lazy(() => import("./pages/HostPage").then((m) => ({ default: m.HostPage })));
const TerracottaPage = lazy(() =>
  import("./pages/TerracottaPage").then((m) => ({ default: m.TerracottaPage })),
);
const AccountsPage = lazy(() =>
  import("./pages/AccountsPage").then((m) => ({ default: m.AccountsPage })),
);
const SettingsPage = lazy(() =>
  import("./pages/SettingsPage").then((m) => ({ default: m.SettingsPage })),
);

function PageFallback() {
  return (
    <div style={{ display: "grid", placeItems: "center", minHeight: 180 }}>
      <Spinner size="md" />
    </div>
  );
}

function LazyPane({ children }: { children: ReactNode }) {
  return <Suspense fallback={<PageFallback />}>{children}</Suspense>;
}

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
      terracotta: t("navTerracotta"),
      accounts: t("navAccounts"),
      settings: t("navSettings"),
    }),
    [t],
  );

  const stickyPaneIds = useMemo(() => ["launch"], []);

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
        element: (
          <LazyPane>
            <ModrinthDetailPage />
          </LazyPane>
        ),
      },
      {
        id: "download",
        match: (p: string) => p.startsWith("/download") && !p.startsWith("/download/mod/"),
        element: (
          <LazyPane>
            <DownloadPage />
          </LazyPane>
        ),
      },
      {
        id: "news",
        match: (p: string) => p.startsWith("/news"),
        element: (
          <LazyPane>
            <NewsPage />
          </LazyPane>
        ),
      },
      {
        id: "versions",
        match: (p: string) => p.startsWith("/versions"),
        element: (
          <LazyPane>
            <VersionsPage />
          </LazyPane>
        ),
      },
      {
        id: "servers",
        match: (p: string) => p.startsWith("/servers"),
        element: (
          <LazyPane>
            <ServersPage />
          </LazyPane>
        ),
      },
      {
        id: "host",
        match: (p: string) => p.startsWith("/host"),
        element: (
          <LazyPane>
            <HostPage />
          </LazyPane>
        ),
      },
      {
        id: "terracotta",
        match: (p: string) => p.startsWith("/terracotta"),
        element: (
          <LazyPane>
            <TerracottaPage />
          </LazyPane>
        ),
      },
      {
        id: "accounts",
        match: (p: string) => p.startsWith("/accounts"),
        element: (
          <LazyPane>
            <AccountsPage />
          </LazyPane>
        ),
      },
      {
        id: "settings",
        match: (p: string) => p.startsWith("/settings"),
        element: (
          <LazyPane>
            <SettingsPage />
          </LazyPane>
        ),
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
        <KeepAliveRoutes panes={panes} maxAlive={3} stickyIds={stickyPaneIds} />
      </AppShell>
      <DownloadStatusBar />
      <ExitGuard />
    </>
  );
}
