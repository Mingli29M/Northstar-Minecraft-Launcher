import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { PageShell } from "../components/PageShell";
import { RELEASES } from "../lib/site";

const PLATFORMS: { name: string; detail: string }[] = [
  {
    name: "Windows",
    detail: "NSIS installer and MSI from GitHub Releases. Requires a 64-bit Windows 10/11 system.",
  },
  {
    name: "macOS",
    detail:
      "Apple Silicon and Intel DMGs. Ad-hoc signed by default; Developer ID + notarization when CI secrets are configured. Gatekeeper may require allowing the app once.",
  },
  {
    name: "Linux",
    detail: "AppImage, deb, and rpm artifacts. Make AppImages executable before running.",
  },
];

export function DownloadPage() {
  return (
    <PageShell
      title="Download"
      hint="Grab the latest installer from GitHub Releases — same pattern as Prism and MultiMC download pages."
    >
      <Banner
        status="info"
        title="Official builds only"
        description="Only download Northstar from this project’s GitHub Releases. Third-party mirrors are not endorsed."
      />

      <Card padding={4}>
        <VStack gap={3}>
          <Text>
            Open the releases page, pick the asset for your OS, then install or extract and run.
            Draft releases from the publish workflow may need to be published by maintainers before
            they appear as latest.
          </Text>
          <Button
            label="Open GitHub Releases"
            variant="primary"
            onClick={() => {
              window.location.href = RELEASES;
            }}
          />
        </VStack>
      </Card>

      <VStack gap={3}>
        {PLATFORMS.map((p) => (
          <Card key={p.name} padding={4}>
            <VStack gap={1}>
              <Text weight="semibold">{p.name}</Text>
              <Text color="secondary">{p.detail}</Text>
            </VStack>
          </Card>
        ))}
      </VStack>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">After install</Text>
          <Text color="secondary">
            Launcher settings and caches live under <code>%APPDATA%\euml\</code> on Windows (and the
            equivalent app-data path on other platforms). The product name is Northstar; the data
            folder name is kept for install stability.
          </Text>
        </VStack>
      </Card>
    </PageShell>
  );
}
