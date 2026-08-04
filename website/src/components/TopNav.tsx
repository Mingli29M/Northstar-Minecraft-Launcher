import { Link } from "react-router-dom";

const BASE = import.meta.env.BASE_URL.replace(/\/?$/, "/");

const LINKS = [
  { href: `${BASE}#why`, label: "Why", hash: true },
  { href: `${BASE}#features`, label: "Features", hash: true },
  { href: `${BASE}#compare`, label: "Compare", hash: true },
  { href: `${BASE}#download`, label: "Download", hash: true },
  { href: "/about", label: "About", hash: false },
  { href: "/license", label: "License", hash: false },
];

export function TopNav() {
  return (
    <header className="ns-topnav">
      <Link to="/" className="ns-topnav-brand">
        Northstar
      </Link>
      <nav className="ns-topnav-links" aria-label="Primary">
        {LINKS.map((l) =>
          l.hash ? (
            <a key={l.href} href={l.href}>
              {l.label}
            </a>
          ) : (
            <Link key={l.href} to={l.href}>
              {l.label}
            </Link>
          ),
        )}
      </nav>
    </header>
  );
}
