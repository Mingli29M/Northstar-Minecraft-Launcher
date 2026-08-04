import { Banner } from "@astryxdesign/core/Banner";
import { Button } from "@astryxdesign/core/Button";
import { Card } from "@astryxdesign/core/Card";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { PageShell } from "../components/PageShell";
import { LICENSE_RAW, LICENSE_TEXT } from "../lib/site";

export function LicensePage() {
  return (
    <PageShell
      title="License"
      hint="Legal terms for Northstar — structured like MultiMC / PCL project pages: clear ownership, branding, and third-party notices."
    >
      <Banner
        status="error"
        title="All rights reserved"
        description="No open-source license is granted. Viewing the repository does not give you rights to copy, modify, redistribute, or rebrand Northstar."
      />

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">LICENSE (repository root)</Text>
          <pre className="ns-license">{LICENSE_TEXT}</pre>
          <Button
            label="View LICENSE on GitHub"
            variant="secondary"
            onClick={() => {
              window.location.href = LICENSE_RAW;
            }}
          />
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">What you may do</Text>
          <Text color="secondary">
            Download and run official builds from this project’s GitHub Releases for personal use,
            subject to the copyright notice above and any additional terms published by the
            copyright holders.
          </Text>
          <Text color="secondary">
            Read the public source for understanding. That is not a grant of copyright, trademark,
            or patent rights.
          </Text>
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">What you may not do</Text>
          <Text color="secondary">
            Copy, modify, fork-and-redistribute, sublicense, or commercially exploit Northstar
            without written permission. Do not reuse the Northstar name, logos, or installer
            branding for unofficial builds (same idea as MultiMC’s branding reservation).
          </Text>
          <Text color="secondary">
            Do not strip or hide this license notice in redistributed materials if permission is
            ever granted in writing — follow that grant instead.
          </Text>
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">Branding & identifying marks</Text>
          <Text color="secondary">
            The Northstar name and related product marks are reserved for official builds from this
            repository. Unofficial builds must not present themselves as Northstar.
          </Text>
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">Third-party software</Text>
          <Text color="secondary">
            npm and Cargo dependencies keep their own licenses (MIT, Apache-2.0, etc.). See{" "}
            <code>package-lock.json</code>, <code>Cargo.lock</code>, and notices in dependency
            packages. Meta Astryx is separate open-source software used by this project under its
            own terms.
          </Text>
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">Minecraft & Mojang</Text>
          <Text color="secondary">
            Minecraft is a trademark of Mojang Synergies AB. To play Minecraft you must own a
            legitimate copy and comply with Mojang’s terms. Northstar does not distribute the
            Minecraft game client as a substitute for purchasing the game.
          </Text>
        </VStack>
      </Card>

      <Card padding={4}>
        <VStack gap={2}>
          <Text weight="semibold">This website</Text>
          <Text color="secondary">
            Site source under <code>website/</code> is part of the same proprietary project unless a
            file states otherwise. Content here is provided for information about official
            downloads and licensing.
          </Text>
        </VStack>
      </Card>
    </PageShell>
  );
}
