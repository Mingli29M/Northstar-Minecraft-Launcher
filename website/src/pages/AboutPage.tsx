import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { CHANGELOG, REPO } from "../lib/site";

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
