import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { CHANGELOG, REPO } from "../lib/site";

const IDLE_ROWS: [string, string, string, string, string][] = [
  ["Prism 11.0.3", "64.1", "58.0", "0.0", "Qt6 portable"],
  ["MultiMC stable", "110.9", "50.2", "0.0", "Qt5 lin64"],
  ["HMCL 3.16.3", "296.5", "217.1", "0.3", "JavaFX jar"],
  ["Northstar 1.1.0", "337.0", "99.8", "0.3", "Tauri/WebKitGTK after opt"],
  ["PCL CE 2.15.0", "—", "—", "—", "Windows-only; not measured"],
];

export function AboutPage() {
  return (
    <div className="ns-site">
      <TopNav />
      <main className="ns-page-pad">
        <VStack gap={4}>
          <VStack gap={2}>
            <Text type="display-3">About</Text>
            <Text color="secondary">
              Product background and how Northstar relates to other launchers.
            </Text>
          </VStack>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">What is Northstar?</Text>
              <Text>
                Northstar is a proprietary desktop Minecraft launcher (early development also used
                the name EUML). It targets a PCL / HMCL–style launch flow with Tauri 2 packaging and
                a Meta Astryx UI — plus built-in Host and ReqGuard.
              </Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">Independent project</Text>
              <Text color="secondary">
                Inspired by workflows from Prism, MultiMC, and PCL-class launchers, but not a fork
                of those projects and not affiliated with them.
              </Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">Resource usage (same-OS sample)</Text>
              <Text color="secondary">
                Idle UI after 30s on Ubuntu 24.04 x86_64 (2026-08-04). Working Set ≈ Linux RSS;
                Private Bytes ≈ <code>Private_Dirty + Private_Clean</code> from{" "}
                <code>smaps_rollup</code>. Full methodology:{" "}
                <a href={`${REPO}/blob/main/BENCHMARKS.md`}>BENCHMARKS.md</a>.
              </Text>
              <div style={{ overflowX: "auto" }}>
                <table
                  style={{
                    width: "100%",
                    borderCollapse: "collapse",
                    fontSize: 13,
                    lineHeight: 1.45,
                  }}
                >
                  <thead>
                    <tr>
                      {["Launcher", "Working Set (MiB)", "Private (MiB)", "CPU %", "Notes"].map(
                        (h) => (
                          <th
                            key={h}
                            style={{
                              textAlign: h === "Launcher" || h === "Notes" ? "left" : "right",
                              padding: "6px 8px",
                              borderBottom: "1px solid var(--color-border, #ddd)",
                            }}
                          >
                            {h}
                          </th>
                        ),
                      )}
                    </tr>
                  </thead>
                  <tbody>
                    {IDLE_ROWS.map((row) => (
                      <tr key={row[0]}>
                        {row.map((cell, i) => (
                          <td
                            key={i}
                            style={{
                              textAlign: i === 0 || i === 4 ? "left" : "right",
                              padding: "6px 8px",
                              borderBottom: "1px solid var(--color-border, #eee)",
                              fontWeight: row[0].startsWith("Northstar") && i === 0 ? 600 : undefined,
                            }}
                          >
                            {cell}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <Text color="secondary">
                After WebKit compositor/DMABUF defaults and frontend lazy-loading, Northstar idle
                private bytes (~100 MiB) undercut HMCL; Working Set still trails Qt (Prism/MultiMC)
                because of the WebView process model. With vanilla <strong>1.21.11</strong>, the
                game dominates totals (~1.5–1.7 GiB). See BENCHMARKS.md.
              </Text>
            </VStack>
          </Card>

          <Banner
            status="warning"
            title="Unofficial software"
            description="Not an official Minecraft product. Not approved by or associated with Mojang Studios or Microsoft."
          />

          <div className="ns-cta-row">
            <Button
              label="GitHub"
              variant="secondary"
              onClick={() => {
                window.location.href = REPO;
              }}
            />
            <Button
              label="Changelog"
              variant="secondary"
              onClick={() => {
                window.location.href = CHANGELOG;
              }}
            />
          </div>
        </VStack>
      </main>
      <SiteFooter />
    </div>
  );
}
