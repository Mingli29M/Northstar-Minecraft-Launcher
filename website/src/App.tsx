import { useMemo } from "react";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";
import { AppShell } from "@astryxdesign/core/AppShell";
import { SiteNav } from "./components/SiteNav";
import { HomePage } from "./pages/HomePage";
import { FeaturesPage } from "./pages/FeaturesPage";
import { DownloadPage } from "./pages/DownloadPage";
import { AboutPage } from "./pages/AboutPage";
import { LicensePage } from "./pages/LicensePage";

export function App() {
  const { pathname } = useLocation();
  const sideNav = useMemo(() => <SiteNav pathname={pathname} />, [pathname]);

  return (
    <AppShell
      variant="elevated"
      height="auto"
      contentPadding={5}
      sideNav={sideNav}
      mobileNav={{ breakpoint: "md" }}
    >
      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route path="/features" element={<FeaturesPage />} />
        <Route path="/download" element={<DownloadPage />} />
        <Route path="/about" element={<AboutPage />} />
        <Route path="/license" element={<LicensePage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AppShell>
  );
}
