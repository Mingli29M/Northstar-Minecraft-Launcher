import { useEffect, useRef, type CSSProperties, type ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import {
  Boxes,
  HardDrive,
  MonitorSmartphone,
  Package,
  Palette,
  Rocket,
  ScanSearch,
  Server,
  ShieldAlert,
  Users,
} from "lucide-react";
import { Button } from "@astryxdesign/core/Button";
import { SiteFooter } from "../components/SiteFooter";
import { TopNav } from "../components/TopNav";
import { ConnectSection } from "../components/ConnectSection";
import { useI18n } from "../i18n";
import type { MessageKey } from "../i18n/messages";
import { openDownload, REPO, RELEASES_LATEST } from "../lib/site";

const CAPABILITIES: {
  titleKey: MessageKey;
  bodyKey: MessageKey;
  icon: LucideIcon;
  shotKey: MessageKey;
  experimental?: boolean;
}[] = [
  { icon: Server, titleKey: "why1Title", bodyKey: "why1Body", shotKey: "shotWhy1" },
  {
    icon: ScanSearch,
    titleKey: "why2Title",
    bodyKey: "why2Body",
    shotKey: "shotWhy2",
    experimental: true,
  },
  {
    icon: MonitorSmartphone,
    titleKey: "why3Title",
    bodyKey: "why3Body",
    shotKey: "shotWhy3",
  },
  { icon: Boxes, titleKey: "feat1Title", bodyKey: "feat1Body", shotKey: "shotFeat1" },
  { icon: Package, titleKey: "feat2Title", bodyKey: "feat2Body", shotKey: "shotFeat2" },
  { icon: Rocket, titleKey: "feat3Title", bodyKey: "feat3Body", shotKey: "shotFeat3" },
  { icon: HardDrive, titleKey: "feat4Title", bodyKey: "feat4Body", shotKey: "shotFeat4" },
  { icon: Users, titleKey: "feat5Title", bodyKey: "feat5Body", shotKey: "shotFeat5" },
  { icon: Palette, titleKey: "feat6Title", bodyKey: "feat6Body", shotKey: "shotFeat6" },
];

function Reveal({ children, className = "" }: { children: ReactNode; className?: string }) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      el.classList.add("is-visible");
      return;
    }

    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry?.isIntersecting) {
          el.classList.add("is-visible");
          io.disconnect();
        }
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.12 },
    );
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <div ref={ref} className={`ns-reveal ${className}`.trim()}>
      {children}
    </div>
  );
}

function ShotBox({ label }: { label: string }) {
  return (
    <div className="ns-shot" role="img" aria-label={label}>
      <div className="ns-shot-frame">
        <span className="ns-shot-label">{label}</span>
      </div>
    </div>
  );
}

function CapabilityRows({
  items,
}: {
  items: {
    titleKey: MessageKey;
    bodyKey: MessageKey;
    icon: LucideIcon;
    shotKey: MessageKey;
    experimental?: boolean;
  }[];
}) {
  const { t } = useI18n();
  return (
    <div className="ns-split-list">
      {items.map((item, i) => {
        const Icon = item.icon;
        const reverse = i % 2 === 1;
        return (
          <Reveal key={item.titleKey}>
            <article className={`ns-split${reverse ? " ns-split-reverse" : ""}`}>
              <div className="ns-split-copy">
                <div className="ns-split-icon" aria-hidden>
                  <Icon size={22} strokeWidth={1.75} />
                </div>
                <h3>
                  {t(item.titleKey)}
                  {item.experimental ? (
                    <span className="ns-experimental">{t("experimentalBadge")}</span>
                  ) : null}
                </h3>
                <p>{t(item.bodyKey)}</p>
              </div>
              <ShotBox label={t(item.shotKey)} />
            </article>
          </Reveal>
        );
      })}
    </div>
  );
}

function useLegacyHashRedirect() {
  useEffect(() => {
    const map: Record<string, string> = {
      "#why": "capabilities",
      "#features": "capabilities",
    };
    const target = map[window.location.hash];
    if (!target) return;
    const el = document.getElementById(target);
    if (el) {
      el.scrollIntoView();
      history.replaceState(null, "", `#${target}`);
    }
  }, []);
}

export function HomePage() {
  const { t } = useI18n();
  useLegacyHashRedirect();

  return (
    <div className="ns-site">
      <a className="ns-skip" href="#capabilities">
        {t("skipToContent")}
      </a>
      <TopNav />

      <section className="ns-hero" aria-labelledby="hero-title">
        <div
          className="ns-hero-bg"
          aria-hidden="true"
          style={
            {
              "--ns-hero-image": `url(${import.meta.env.BASE_URL}northstar-hero.png)`,
            } as CSSProperties
          }
        />
        <div className="ns-hero-inner">
          <p className="ns-hero-pill">{t("heroPill")}</p>
          <h1 id="hero-title" className="ns-brand">
            {t("brand")}
          </h1>
          <p className="ns-tagline">{t("heroTagline")}</p>
          <div className="ns-cta-row">
            <Button label={t("heroDownload")} variant="primary" onClick={openDownload} />
            <span className="ns-cta-ghost">
              <Button
                label={t("heroViewSource")}
                variant="secondary"
                onClick={() => {
                  window.location.href = REPO;
                }}
              />
            </span>
          </div>
          <ul className="ns-platforms" aria-label={t("heroPlatformsAria")}>
            <li>Windows</li>
            <li>macOS</li>
            <li>Linux</li>
          </ul>
          <div className="ns-hero-star-trail" aria-hidden="true">
            <div className="ns-hero-star-stack">
              <img
                className="ns-hero-star-ghost"
                src={`${import.meta.env.BASE_URL}northstar-overlay.png`}
                alt=""
                width={512}
                height={768}
              />
              <img
                className="ns-hero-star"
                src={`${import.meta.env.BASE_URL}northstar-overlay.png`}
                alt=""
                width={512}
                height={768}
              />
            </div>
          </div>
        </div>
      </section>

      <main>
        <section
          className="ns-section ns-section-wide"
          id="capabilities"
          aria-labelledby="capabilities-heading"
        >
          <div className="ns-caps-head">
            <h2 id="capabilities-heading">{t("capabilitiesTitle")}</h2>
            <p className="ns-section-lead">{t("capabilitiesLead")}</p>
          </div>
          <CapabilityRows items={CAPABILITIES} />
        </section>

        <section
          className="ns-section ns-compare-section"
          id="compare"
          aria-labelledby="compare-heading"
        >
          <div className="ns-section-head">
            <h2 id="compare-heading">{t("compareTitle")}</h2>
            <p className="ns-section-lead">
              {t("compareLeadBefore")}{" "}
              <a href="https://github.com/Mingli29M/Northstar-Minecraft-Launcher/blob/main/BENCHMARKS.md">
                BENCHMARKS.md
              </a>
              {t("compareLeadAfter")}
            </p>
          </div>

          <p className="ns-measure-note">{t("measureUnitNote")}</p>

          <p className="ns-table-caption" id="compare-table-caption">
            {t("compareTableCaption")}
          </p>
          <div className="ns-compare-wrap">
            <table className="ns-compare" aria-describedby="compare-table-caption">
              <thead>
                <tr>
                  <th scope="col">{t("compareAspect")}</th>
                  <th scope="col">Northstar</th>
                  <th scope="col">Prism</th>
                  <th scope="col">MultiMC</th>
                  <th scope="col">PCL / CE</th>
                  <th scope="col">HMCL</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>{t("compareToolkit")}</td>
                  <td>Tauri 2 + WebView + Astryx</td>
                  <td>Qt</td>
                  <td>Qt</td>
                  <td>.NET (WPF-class)</td>
                  <td>Java / JVM UI</td>
                </tr>
                <tr>
                  <td>
                    {t("compareWs")}
                    <span className="ns-unit">{t("compareUnitMib")}</span>
                  </td>
                  <td className="ns-num">~337</td>
                  <td className="ns-num">~64</td>
                  <td className="ns-num">~111</td>
                  <td>{t("compareNaWin")}</td>
                  <td className="ns-num">~297</td>
                </tr>
                <tr>
                  <td>
                    {t("comparePrivate")}
                    <span className="ns-unit">{t("compareUnitMib")}</span>
                  </td>
                  <td className="ns-num">~100</td>
                  <td className="ns-num">~58</td>
                  <td className="ns-num">~50</td>
                  <td>{t("compareNa")}</td>
                  <td className="ns-num">~217</td>
                </tr>
                <tr>
                  <td>
                    <span className="ns-inline-icon" aria-hidden>
                      <ShieldAlert size={14} strokeWidth={2} />
                    </span>{" "}
                    {t("compareHost")}
                  </td>
                  <td>{t("compareYes")}</td>
                  <td>{t("compareNo")}</td>
                  <td>{t("compareNo")}</td>
                  <td>{t("compareLimited")}</td>
                  <td>{t("compareLimited")}</td>
                </tr>
                <tr>
                  <td>{t("compareReqguard")}</td>
                  <td>{t("compareYes")}</td>
                  <td>{t("compareNo")}</td>
                  <td>{t("compareNo")}</td>
                  <td>{t("compareDifferent")}</td>
                  <td>{t("compareDifferent")}</td>
                </tr>
                <tr>
                  <td>{t("compareLicense")}</td>
                  <td>{t("compareArr")}</td>
                  <td>GPL-3</td>
                  <td>{t("compareBranding")}</td>
                  <td>{t("compareCustomApache")}</td>
                  <td>GPL</td>
                </tr>
              </tbody>
            </table>
          </div>
          <p className="ns-footnote">{t("compareFootnote")}</p>
        </section>

        <section className="ns-close" id="download" aria-labelledby="close-heading">
          <div className="ns-close-inner">
            <h2 id="close-heading">{t("closeTitle")}</h2>
            <p className="ns-section-lead">
              {t("closeLeadBefore")}{" "}
              <a href={RELEASES_LATEST}>{t("downloadLeadLink")}</a>
              {t("closeLeadAfter")}
            </p>
            <div className="ns-cta-row">
              <Button label={t("downloadOpenLatestBtn")} variant="primary" onClick={openDownload} />
              <Button
                label={t("downloadViewGithub")}
                variant="secondary"
                onClick={() => {
                  window.location.href = REPO;
                }}
              />
            </div>
            <p className="ns-footnote ns-footnote-spaced">{t("downloadFootnote")}</p>
          </div>
        </section>

        <ConnectSection />
      </main>

      <SiteFooter />
    </div>
  );
}
