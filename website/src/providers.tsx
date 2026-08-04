import type { ComponentProps, ReactNode } from "react";
import { BrowserRouter, Link as RouterLink } from "react-router-dom";
import { Theme } from "@astryxdesign/core/theme";
import { LinkProvider } from "@astryxdesign/core/Link";
import { LayerProvider } from "@astryxdesign/core/Layer";
import { neutralTheme } from "@astryxdesign/theme-neutral";
import { I18nProvider } from "./i18n";

function RRLink({
  href,
  children,
  ...rest
}: ComponentProps<"a"> & { href?: string }) {
  if (!href) {
    return <a {...rest}>{children}</a>;
  }
  if (/^https?:\/\//i.test(href) || href.startsWith("mailto:") || href.startsWith("#")) {
    return (
      <a href={href} {...rest} rel={rest.rel ?? (href.startsWith("http") ? "noreferrer" : undefined)}>
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
          <LayerProvider>
            <I18nProvider>{children}</I18nProvider>
          </LayerProvider>
        </LinkProvider>
      </Theme>
    </BrowserRouter>
  );
}
