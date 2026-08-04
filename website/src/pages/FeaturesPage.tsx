import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { PageShell } from "../components/PageShell";

const FEATURES: { title: string; body: string }[] = [
  {
    title: "Launch & versions",
    body: "PCL/HMCL-style start flow, multi-loader installs (Vanilla, Fabric, Quilt, Forge, NeoForge, Paper/Purpur), and per-instance JVM settings.",
  },
  {
    title: "ReqGuard",
    body: "Scan mod jar dependency metadata before you boot so missing Fabric API / libraries show up with actionable fixes.",
  },
  {
    title: "Mods & content",
    body: "Browse and install Modrinth mods and modpacks in-app. Import .mrpack and Prism / MultiMC instance folders.",
  },
  {
    title: "Host",
    body: "Dedicated server manager with console, EULA, properties, player lists, file transfer, and UPnP → NAT-PMP → PCP port mapping.",
  },
  {
    title: "Accounts",
    body: "Microsoft, offline (stable UUIDs), and LittleSkin (authlib-injector) accounts — switch without leaving Launch.",
  },
  {
    title: "Appearance & locale",
    body: "Accent color, background, font, and UI scale in Settings. UI languages: English, 简体中文, Deutsch.",
  },
  {
    title: "Downloads",
    body: "Parallel library and asset downloads with optional BMCLAPI mirrors for regions where Mojang CDNs are slow.",
  },
  {
    title: "Native packaging",
    body: "Windows (NSIS + MSI), macOS (Apple Silicon + Intel), and Linux (AppImage, deb, rpm) via GitHub Actions.",
  },
];

export function FeaturesPage() {
  return (
    <PageShell
      title="Features"
      hint="What you get in the desktop app — similar scope to Prism / MultiMC / PCL-class launchers, with Host and ReqGuard on top."
    >
      <VStack gap={3}>
        {FEATURES.map((f) => (
          <Card key={f.title} padding={4}>
            <VStack gap={1}>
              <Text weight="semibold">{f.title}</Text>
              <Text color="secondary">{f.body}</Text>
            </VStack>
          </Card>
        ))}
      </VStack>
    </PageShell>
  );
}
