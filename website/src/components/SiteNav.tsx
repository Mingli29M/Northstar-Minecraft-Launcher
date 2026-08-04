import { memo } from "react";
import { SideNav, SideNavHeading, SideNavItem, SideNavSection } from "@astryxdesign/core/SideNav";
import { Text } from "@astryxdesign/core/Text";
import { Home, Sparkles, Download, Info, Scale } from "lucide-react";

type Props = { pathname: string };

function navKey(pathname: string): string {
  if (pathname.startsWith("/features")) return "features";
  if (pathname.startsWith("/download")) return "download";
  if (pathname.startsWith("/about")) return "about";
  if (pathname.startsWith("/license")) return "license";
  return "home";
}

function SiteNavInner({ pathname }: Props) {
  const key = navKey(pathname);
  return (
    <SideNav header={<SideNavHeading heading="Northstar" />} style={{ minHeight: "100%" }}>
      <SideNavSection title="Site">
        <SideNavItem label="Home" href="/" icon={Home} isSelected={key === "home"} />
        <SideNavItem
          label="Features"
          href="/features"
          icon={Sparkles}
          isSelected={key === "features"}
        />
        <SideNavItem
          label="Download"
          href="/download"
          icon={Download}
          isSelected={key === "download"}
        />
        <SideNavItem label="About" href="/about" icon={Info} isSelected={key === "about"} />
        <SideNavItem
          label="License"
          href="/license"
          icon={Scale}
          isSelected={key === "license"}
        />
      </SideNavSection>
      <Text color="secondary" type="supporting" style={{ padding: 12 }}>
        Meta Astryx UI
      </Text>
    </SideNav>
  );
}

export const SiteNav = memo(SiteNavInner);
