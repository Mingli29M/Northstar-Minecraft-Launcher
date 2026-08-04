import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { PageShell } from "../components/PageShell";
import { CHANGELOG, REPO } from "../lib/site";

export function AboutPage() {
  return (
    <PageShell
      title="About"
      hint="Who we are, what this is, and how it relates to other launchers."
    >
      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
            What is Northstar?
          </Text>
          <Text>
            Northstar is a proprietary desktop Minecraft launcher (formerly referred to as EUML in
            early development). It targets a PCL / HMCL–style launch experience with modern
            packaging: Tauri 2, React 19, and Meta’s Astryx design system.
          </Text>
          <Text color="secondary">
            The goal is a focused tool for launching, content, and hosting — not an embedded
            browser chrome dashboard. Appearance settings, Host networking, and ReqGuard are first-class
            parts of that product.
          </Text>
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">Compared to other launchers</Text>
          <Text color="secondary">
            <strong>Prism Launcher</strong> and <strong>MultiMC</strong> emphasize free
            redistributability, instance isolation, and lightweight Qt UIs.{" "}
            <strong>PCL / PCL CE</strong> emphasize a polished Windows-first start flow and
            community forks with explicit license guidelines.
          </Text>
          <Text color="secondary">
            Northstar is inspired by those workflows (instances, Modrinth, multi-account, clear
            About/License pages) but is an independent project with an All Rights Reserved license.
            It is not a fork of PCL, Prism, or MultiMC, and does not claim affiliation with those
            projects.
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
            <a href="https://github.com/mingli29m/northstar-minecraft-launcher/blob/main/BENCHMARKS.md">
              BENCHMARKS.md
            </a>
            .
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
                {[
                  ["Prism 11.0.3", "64.1", "58.0", "0.0", "Qt6 portable"],
                  ["MultiMC stable", "110.9", "50.2", "0.0", "Qt5 lin64"],
                  ["HMCL 3.16.3", "296.5", "217.1", "0.3", "JavaFX jar"],
                  ["Northstar 1.1.0", "660.5", "376.7", "0.3", "Tauri/WebKitGTK (3 procs)"],
                  ["PCL CE 2.15.0", "—", "—", "—", "Windows-only; not measured"],
                ].map((row) => (
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
            With vanilla <strong>1.21.11</strong> running (title screen, llvmpipe), the game
            dominated memory: Prism total ~1553 MiB RSS / Northstar total ~1670 MiB RSS. See
            BENCHMARKS.md for launcher vs game split.
          </Text>
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">Stack</Text>
          <Text color="secondary">
            Desktop shell: Tauri 2 + Rust. Frontend: React, TypeScript, Vite, Tailwind CSS layers,
            and <code>@astryxdesign/core</code> with <code>theme-neutral</code>. This marketing site
            uses the same Astryx UI system as the launcher.
          </Text>
        </VStack>
      </Card>

      <Banner
        status="warning"
        title="Unofficial software"
        description="Northstar is not an official Minecraft product. It is not approved by or associated with Mojang Studios or Microsoft."
      />

      <Card padding={4}>
        <VStack gap={3}>
          <Text weight="semibold">Links</Text>
          <Button
            label="Source on GitHub"
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
        </VStack>
      </Card>
    </PageShell>
  );
}
