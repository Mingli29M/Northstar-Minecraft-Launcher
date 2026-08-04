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
