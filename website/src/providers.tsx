import type { ComponentProps, ReactNode } from "react";
import { BrowserRouter, Link as RouterLink } from "react-router-dom";
import { Theme } from "@astryxdesign/core/theme";
import { LinkProvider } from "@astryxdesign/core/Link";
import { LayerProvider } from "@astryxdesign/core/Layer";
import { neutralTheme } from "@astryxdesign/theme-neutral";

function RRLink({
  href,
  children,
  ...rest
}: ComponentProps<"a"> & { href?: string }) {
  if (!href) {
    return (
      <a {...rest}>
        {children}
      </a>
    );
  }
  if (/^https?:\/\//i.test(href) || href.startsWith("mailto:")) {
    return (
      <a href={href} {...rest} rel={rest.rel ?? "noreferrer"}>
        {children}
      </a>
    );
  }
  return (
    <RouterLink to={href} {...(rest as object)}>
      {children}
    </RouterLink>
  );
}

export function Providers({ children }: { children: ReactNode }) {
  return (
    <BrowserRouter basename="/Northstar-Minecraft-Launcher">
      <Theme theme={neutralTheme}>
        <LinkProvider component={RRLink}>
          <LayerProvider>{children}</LayerProvider>
        </LinkProvider>
      </Theme>
    </BrowserRouter>
  );
}
