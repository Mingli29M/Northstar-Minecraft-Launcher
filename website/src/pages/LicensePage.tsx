import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { LICENSE_RAW, LICENSE_TEXT } from "../lib/site";

export function LicensePage() {
  return (
    <div className="ns-site">
      <TopNav />
      <main className="ns-page-pad">
        <VStack gap={4}>
          <VStack gap={2}>
            <Text type="display-3">License</Text>
            <Text color="secondary">
              All rights reserved — ownership, branding, third-party deps, and Minecraft trademark
              notices.
            </Text>
          </VStack>

          <Banner
            status="error"
            title="All rights reserved"
            description="No open-source license is granted. Viewing the repository does not give rights to copy, modify, redistribute, or rebrand Northstar."
          />

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">LICENSE</Text>
              <pre className="ns-license-block">{LICENSE_TEXT}</pre>
              <Button
                label="View on GitHub"
                variant="secondary"
                onClick={() => {
                  window.location.href = LICENSE_RAW;
                }}
              />
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">Branding</Text>
              <Text color="secondary">
                The Northstar name and installer branding are reserved for official builds from this
                repository (same idea as MultiMC’s branding reservation).
              </Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={2}>
              <Text weight="semibold">Third-party & Minecraft</Text>
              <Text color="secondary">
                Dependencies keep their own licenses. Minecraft is a trademark of Mojang Synergies
                AB; you need a legitimate game copy to play.
              </Text>
            </VStack>
          </Card>
        </VStack>
      </main>
      <SiteFooter />
    </div>
  );
}
