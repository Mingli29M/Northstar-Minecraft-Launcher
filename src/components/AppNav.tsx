import { memo } from "react";
import { SideNav, SideNavHeading, SideNavItem, SideNavSection } from "@astryxdesign/core/SideNav";
import { Text } from "@astryxdesign/core/Text";
import {
  Rocket,
  Download,
  Newspaper,
  Boxes,
  Server,
  HardDrive,
  Network,
  Users,
  Settings,
} from "lucide-react";

type Props = {
  appName: string;
  labels: {
    launch: string;
    download: string;
    news: string;
    versions: string;
    servers: string;
    host: string;
    terracotta: string;
    accounts: string;
    settings: string;
  };
  pathname: string;
};

function navKey(pathname: string): string {
  if (pathname.startsWith("/download")) return "download";
  if (pathname.startsWith("/news")) return "news";
  if (pathname.startsWith("/versions")) return "versions";
  if (pathname.startsWith("/servers")) return "servers";
  if (pathname.startsWith("/host")) return "host";
  if (pathname.startsWith("/terracotta")) return "terracotta";
  if (pathname.startsWith("/accounts")) return "accounts";
  if (pathname.startsWith("/settings")) return "settings";
  return "launch";
}

function AppNavInner({ appName, labels, pathname }: Props) {
  const key = navKey(pathname);
  return (
    <SideNav header={<SideNavHeading heading={appName} />} style={{ minHeight: "100%" }}>
      <SideNavSection title={appName}>
        <SideNavItem label={labels.launch} href="/" icon={Rocket} isSelected={key === "launch"} />
        <SideNavItem
          label={labels.download}
          href="/download"
          icon={Download}
          isSelected={key === "download"}
        />
        <SideNavItem label={labels.news} href="/news" icon={Newspaper} isSelected={key === "news"} />
        <SideNavItem
          label={labels.versions}
          href="/versions"
          icon={Boxes}
          isSelected={key === "versions"}
        />
        <SideNavItem
          label={labels.servers}
          href="/servers"
          icon={Server}
          isSelected={key === "servers"}
        />
        <SideNavItem
          label={labels.host}
          href="/host"
          icon={HardDrive}
          isSelected={key === "host"}
        />
        <SideNavItem
          label={labels.terracotta}
          href="/terracotta"
          icon={Network}
          isSelected={key === "terracotta"}
        />
        <SideNavItem
          label={labels.accounts}
          href="/accounts"
          icon={Users}
          isSelected={key === "accounts"}
        />
        <SideNavItem
          label={labels.settings}
          href="/settings"
          icon={Settings}
          isSelected={key === "settings"}
        />
      </SideNavSection>
      <Text color="secondary" type="supporting" style={{ padding: 12 }}>
        Meta Astryx
      </Text>
    </SideNav>
  );
}

export const AppNav = memo(AppNavInner);
