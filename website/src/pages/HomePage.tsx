import { Button } from "@astryxdesign/core/Button";
import { Banner } from "@astryxdesign/core/Banner";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { REPO, RELEASES } from "../lib/site";

const WHY = [
  {
    title: "Launch + Host in one app",
    body: "Play and run dedicated servers without bouncing between a launcher and a separate host tool. Console, EULA, properties, and port maps stay in the Host tab.",
  },
  {
    title: "Catch broken mods before you boot",
    body: "ReqGuard reads mod dependency metadata and surfaces missing libraries (e.g. Fabric API) before Minecraft starts — fewer crash-loop cycles.",
  },
  {
    title: "Desktop-native shell, modern UI kit",
    body: "Tauri 2 keeps the shell native; the UI uses Meta Astryx (same design system as the app). No Electron-sized runtime for the window chrome.",
  },
];

const FEATURES = [
  {
    title: "Versions & loaders",
    body: "Vanilla, Fabric, Quilt, Forge, NeoForge, Paper/Purpur. Per-instance JVM args and Java detection.",
  },
  {
    title: "Modrinth & imports",
    body: "Browse/install mods and packs in-app. Import .mrpack and Prism / MultiMC folders.",
  },
  {
    title: "Accounts",
    body: "Microsoft, offline (stable UUIDs), and LittleSkin (authlib-injector).",
  },
  {
    title: "Networking for Host",
    body: "UPnP → NAT-PMP → PCP cascade, join addresses, adapter list, firewall helpers on Windows.",
  },
  {
    title: "Appearance",
    body: "Accent, background, font, and UI scale — persisted in local settings.",
  },
  {
    title: "Locales",
    body: "English, 简体中文, and Deutsch in Settings.",
  },
];

export function HomePage() {
  return (
    <div className="ns-site">
      <TopNav />

      <section className="ns-hero" aria-label="Northstar">
        <div className="ns-hero-inner">
          <h1 className="ns-brand">Northstar</h1>
          <p className="ns-tagline">
            A desktop Minecraft launcher with Host, ReqGuard, and Modrinth — built to feel like a
            tool, not a dashboard.
          </p>
          <div className="ns-cta-row">
            <Button
              label="Download"
              variant="primary"
              onClick={() => {
                window.location.href = RELEASES;
              }}
            />
            <Button
              label="GitHub"
              variant="secondary"
              onClick={() => {
                window.location.href = REPO;
              }}
            />
          </div>
        </div>
      </section>

      <section className="ns-section" id="why" aria-labelledby="why-heading">
        <h2 id="why-heading">Why use this</h2>
        <p className="ns-section-lead">
          If you want Prism-class instance management plus a built-in dedicated Host and a
          pre-launch dependency check, Northstar is aimed at that workflow.
        </p>
        <div className="ns-grid ns-grid-2">
          {WHY.map((item) => (
            <Card key={item.title} padding={4}>
              <VStack gap={2}>
                <Text weight="semibold">{item.title}</Text>
                <Text color="secondary">{item.body}</Text>
              </VStack>
            </Card>
          ))}
        </div>
      </section>

      <section className="ns-section" id="features" aria-labelledby="features-heading">
        <h2 id="features-heading">Features</h2>
        <p className="ns-section-lead">
          One app for launching, content, accounts, and hosting — without stuffing the first screen
          with stats strips.
        </p>
        <div className="ns-grid ns-grid-2">
          {FEATURES.map((f) => (
            <Card key={f.title} padding={4}>
              <VStack gap={1}>
                <Text weight="semibold">{f.title}</Text>
                <Text color="secondary">{f.body}</Text>
              </VStack>
            </Card>
          ))}
        </div>
      </section>

      <section className="ns-section" id="compare" aria-labelledby="compare-heading">
        <h2 id="compare-heading">Compare</h2>
        <p className="ns-section-lead">
          Architecture and product scope versus common launchers. Idle RAM / CPU numbers are not
          published here yet — see the note below.
        </p>

        <Banner
          status="warning"
          title="Measured efficiency TBD"
          description="Fair idle-RAM and CPU comparisons need a release build of Northstar plus Prism and MultiMC on the same machine. This PC has PCL CE, HMCL, and a debug euml.exe only — not enough for a trustworthy table. Hand that benchmark pass to a cloud agent (or a dedicated test box) before citing numbers."
        />

        <div style={{ height: 12 }} />

        <div className="ns-compare-wrap">
          <table className="ns-compare">
            <thead>
              <tr>
                <th scope="col">Aspect</th>
                <th scope="col">Northstar</th>
                <th scope="col">Prism</th>
                <th scope="col">MultiMC</th>
                <th scope="col">PCL / CE</th>
                <th scope="col">HMCL</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>UI toolkit</td>
                <td>Tauri 2 + WebView + Astryx</td>
                <td>Qt</td>
                <td>Qt</td>
                <td>.NET (WPF-class)</td>
                <td>Java / JVM UI</td>
              </tr>
              <tr>
                <td>Idle RAM (measured)</td>
                <td className="ns-muted">Pending release + bench</td>
                <td className="ns-muted">Pending install + bench</td>
                <td className="ns-muted">Pending install + bench</td>
                <td className="ns-muted">Local install; not published yet</td>
                <td className="ns-muted">Local install; not published yet</td>
              </tr>
              <tr>
                <td>Disk (installer / portable)</td>
                <td className="ns-muted">From Releases (varies by OS)</td>
                <td>Native Qt package</td>
                <td>Lightweight portable</td>
                <td>~20 MB CE exe (typical)</td>
                <td>~10 MB jar/exe (typical)</td>
              </tr>
              <tr>
                <td>Platforms</td>
                <td>Windows, macOS, Linux</td>
                <td>Windows, macOS, Linux</td>
                <td>Windows, macOS, Linux</td>
                <td>Windows-first</td>
                <td>Cross-platform JVM</td>
              </tr>
              <tr>
                <td>License</td>
                <td>All Rights Reserved</td>
                <td>GPL-3</td>
                <td>Source for collaboration; branding reserved</td>
                <td>Custom core + Apache elsewhere</td>
                <td>GPL</td>
              </tr>
              <tr>
                <td>Built-in dedicated Host</td>
                <td>Yes (console, UPnP cascade)</td>
                <td>No (external tools)</td>
                <td>No</td>
                <td>Limited / different model</td>
                <td>Limited / different model</td>
              </tr>
              <tr>
                <td>ReqGuard-style precheck</td>
                <td>Yes</td>
                <td>No equivalent first-class</td>
                <td>No</td>
                <td>Different checks</td>
                <td>Different checks</td>
              </tr>
              <tr>
                <td>Modrinth in-app</td>
                <td>Yes</td>
                <td>Yes</td>
                <td>Yes</td>
                <td>Yes</td>
                <td>Yes</td>
              </tr>
              <tr>
                <td>Redistributable FOSS</td>
                <td>No</td>
                <td>Yes</td>
                <td>Restricted branding / keys</td>
                <td>Follow their guidelines</td>
                <td>Yes (GPL)</td>
              </tr>
            </tbody>
          </table>
        </div>
        <p className="ns-footnote">
          Comparison is qualitative on purpose. Do not treat “Pending” cells as wins or losses.
          Prism and MultiMC often win on minimal Qt idle footprints; Java and .NET launchers pay a
          runtime baseline; Tauri sits between a pure native toolkit and Electron. Publish measured
          Working Set / private bytes after a controlled idle-30s protocol on identical hardware.
        </p>
      </section>

      <section className="ns-section" id="download" aria-labelledby="download-heading">
        <h2 id="download-heading">Download</h2>
        <p className="ns-section-lead">
          Official builds only — Windows, macOS, and Linux installers from GitHub Releases.
        </p>
        <Card padding={4}>
          <VStack gap={3}>
            <Text color="secondary">
              Pick the asset for your OS. Settings live under <code>%APPDATA%\euml\</code> on Windows
              (product name Northstar; folder kept for upgrade stability).
            </Text>
            <div className="ns-cta-row">
              <Button
                label="Open GitHub Releases"
                variant="primary"
                onClick={() => {
                  window.location.href = RELEASES;
                }}
              />
            </div>
          </VStack>
        </Card>
      </section>

      <SiteFooter />
    </div>
  );
}
