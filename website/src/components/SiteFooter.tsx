import { Link } from "react-router-dom";
import { CHANGELOG, LICENSE_RAW, REPO } from "../lib/site";

export function SiteFooter() {
  return (
    <footer className="ns-footer">
      <div className="ns-footer-inner">
        <p>
          © 2026 Northstar contributors. All rights reserved.{" "}
          <Link to="/license">License</Link> · <a href={CHANGELOG}>Changelog</a> ·{" "}
          <a href={REPO}>GitHub</a> · <a href={LICENSE_RAW}>LICENSE file</a>
        </p>
        <p>
          Not affiliated with Mojang Studios or Microsoft. “Minecraft” is a trademark of Mojang
          Synergies AB. Mentions of Prism, MultiMC, PCL, and HMCL are for comparison only.
        </p>
      </div>
    </footer>
  );
}
