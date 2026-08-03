import type { ComponentProps, ReactNode } from "react";
import { Link as RouterLink } from "react-router-dom";
import { Theme } from "@astryxdesign/core/theme";
import { LinkProvider } from "@astryxdesign/core/Link";
import { LayerProvider } from "@astryxdesign/core/Layer";
import { neutralTheme } from "@astryxdesign/theme-neutral";
import { I18nProvider } from "./i18n";
import { DownloadStatusProvider } from "./lib/downloadStatus";
import { FavoritesProvider } from "./lib/favorites";

/** React Router adapter for Astryx LinkProvider (expects `href`). */
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
  return (
    <RouterLink to={href} {...(rest as object)}>
      {children}
    </RouterLink>
  );
}

export function Providers({ children }: { children: ReactNode }) {
  return (
    <Theme theme={neutralTheme}>
      <LinkProvider component={RRLink}>
        <LayerProvider>
          <I18nProvider>
            <DownloadStatusProvider>
              <FavoritesProvider>{children}</FavoritesProvider>
            </DownloadStatusProvider>
          </I18nProvider>
        </LayerProvider>
      </LinkProvider>
    </Theme>
  );
}
