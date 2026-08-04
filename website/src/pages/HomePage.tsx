import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { HStack } from "@astryxdesign/core/HStack";
import { VStack } from "@astryxdesign/core/VStack";
import { PageShell } from "../components/PageShell";
import { REPO, RELEASES } from "../lib/site";

export function HomePage() {
  return (
    <PageShell>
      <div className="ns-hero">
        <h1 className="ns-brand">Northstar</h1>
        <Text color="secondary" style={{ maxWidth: "28rem", marginBottom: 20, display: "block" }}>
          A desktop Minecraft launcher with Host, ReqGuard, and Modrinth — built to feel like a
          tool, not a dashboard.
        </Text>
        <HStack gap={3} style={{ flexWrap: "wrap" }}>
          <Button
            label="Download"
            variant="primary"
            onClick={() => {
              window.location.href = RELEASES;
            }}
          />
          <Button
            label="View on GitHub"
            variant="secondary"
            onClick={() => {
              window.location.href = REPO;
            }}
          />
        </HStack>
      </div>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold" type="display-3" style={{ fontSize: 20 }}>
            Overview
          </Text>
          <Text color="secondary">
            Northstar is a Tauri 2 desktop launcher for Minecraft Java Edition. Manage versions,
            accounts, Modrinth content, and dedicated servers in one place — with a UI built on
            Meta’s open-source Astryx design system (same stack as the app itself).
          </Text>
          <Text color="secondary">
            Installers for Windows, macOS, and Linux ship from GitHub Releases. Settings and game
            data stay on your machine under the launcher data folder.
          </Text>
        </VStack>
      </Card>
    </PageShell>
  );
}
