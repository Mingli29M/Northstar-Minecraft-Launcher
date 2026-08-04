const RELEASES =
  "https://github.com/Mingli29M/Northstar-Minecraft-Launcher/releases";
const REPO = "https://github.com/Mingli29M/Northstar-Minecraft-Launcher";

export function App() {
  return (
    <div className="sky">
      <header className="hero">
        <h1 className="brand">Northstar</h1>
        <p className="tagline">
          A desktop Minecraft launcher with Host, ReqGuard, and Modrinth — built to feel like a
          tool, not a dashboard.
        </p>
        <div className="cta-row">
          <a className="btn btn-primary" href={RELEASES}>
            Download
          </a>
          <a className="btn btn-ghost" href={REPO}>
            View on GitHub
          </a>
        </div>
      </header>

      <section className="section" aria-labelledby="features-heading">
        <h2 id="features-heading">Built for play and hosting</h2>
        <p>One app for launching, mods, and dedicated servers — without the clutter.</p>
        <div className="features">
          <article className="feature">
            <h3>Launch & versions</h3>
            <p>
              PCL/HMCL-style start flow, multi-loader installs, and per-instance JVM settings.
            </p>
          </article>
          <article className="feature">
            <h3>ReqGuard</h3>
            <p>Scan mod dependencies before you boot so missing libraries show up early.</p>
          </article>
          <article className="feature">
            <h3>Host</h3>
            <p>
              Dedicated servers with console, EULA, properties, and UPnP / NAT-PMP / PCP port maps.
            </p>
          </article>
          <article className="feature">
            <h3>Modrinth & accounts</h3>
            <p>Browse and install content in-app. Microsoft, offline, and LittleSkin accounts.</p>
          </article>
        </div>
      </section>

      <footer className="footer">
        <p>
          Northstar is proprietary software. All rights reserved. See the{" "}
          <a href={`${REPO}/blob/main/LICENSE`}>LICENSE</a> in the repository. Minecraft is a
          trademark of Mojang Synergies AB.
        </p>
      </footer>
    </div>
  );
}
