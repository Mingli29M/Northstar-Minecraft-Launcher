import type { ReactNode } from "react";
import { Text } from "@astryxdesign/core/Text";
import { VStack } from "@astryxdesign/core/VStack";
import { CHANGELOG, LICENSE_RAW, REPO } from "../lib/site";

export function PageShell({
  title,
  hint,
  children,
}: {
  title?: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <VStack gap={4} className="ns-page">
      {(title || hint) && (
        <VStack gap={2}>
          {title ? <Text type="display-3">{title}</Text> : null}
          {hint ? <Text color="secondary">{hint}</Text> : null}
        </VStack>
      )}
      {children}
      <footer className="ns-footer">
        <VStack gap={1}>
          <Text color="secondary" type="supporting">
            © 2026 Northstar contributors. All rights reserved.{" "}
            <a href={LICENSE_RAW}>License</a> · <a href={CHANGELOG}>Changelog</a> ·{" "}
            <a href={REPO}>GitHub</a>
          </Text>
          <Text color="secondary" type="supporting">
            Not affiliated with Mojang Studios or Microsoft. “Minecraft” is a trademark of Mojang
            Synergies AB.
          </Text>
        </VStack>
      </footer>
    </VStack>
  );
}
