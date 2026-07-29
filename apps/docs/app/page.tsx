/* oxlint-disable import/no-default-export -- Next.js discovers route pages through a default export. */
import Link from "next/link";
import {
  ArrowRight,
  Blocks,
  Braces,
  CircleCheckBig,
  Code2,
  GitBranch,
  ScanSearch,
  Sparkles,
} from "lucide-react";
import { site } from "@/lib/site";

const capabilities = [
  {
    icon: ScanSearch,
    title: "Understand before changing",
    body: "Typed evidence connects selectors, source references, cascade winners, and transformation decisions.",
  },
  {
    icon: GitBranch,
    title: "One engine, every surface",
    body: "CLI, LSP, NAPI, WASM, and build adapters consume the same semantic contracts.",
  },
  {
    icon: Blocks,
    title: "Fail closed when evidence ends",
    body: "Precision states stay visible instead of turning an unknown into a confident rewrite.",
  },
] as const;

export default function HomePage() {
  return (
    <main className="omena-home">
      <div className="omena-home-orb omena-home-orb-one" />
      <div className="omena-home-orb omena-home-orb-two" />
      <nav className="omena-home-nav" aria-label="Primary navigation">
        <Link href="/" className="omena-wordmark">
          <span aria-hidden="true" className="omena-wordmark-mark">
            O
          </span>
          Omena
        </Link>
        <div>
          <Link href="/docs/getting-started">Docs</Link>
          <Link href="/docs/reference/README">Reference</Link>
          <a href={site.repository} aria-label="Omena on GitHub">
            <Code2 aria-hidden="true" />
          </a>
        </div>
      </nav>

      <section className="omena-hero">
        <div className="omena-hero-copy">
          <p className="omena-eyebrow">
            <Sparkles aria-hidden="true" />
            Semantic CSS infrastructure
          </p>
          <h1>
            CSS tools should show
            <span> why they are right.</span>
          </h1>
          <p className="omena-hero-lede">
            Omena gives CSS Modules, Sass, editors, and build pipelines one evidence-aware engine
            for diagnostics, safe transformations, and workspace intelligence.
          </p>
          <div className="omena-hero-actions">
            <Link href="/docs/getting-started" className="omena-cta omena-cta-primary">
              Start in five minutes
              <ArrowRight aria-hidden="true" />
            </Link>
            <Link href="/docs/playground" className="omena-cta omena-cta-secondary">
              Try it in the browser
              <Braces aria-hidden="true" />
            </Link>
          </div>
          <ul className="omena-proof-list">
            <li>
              <CircleCheckBig aria-hidden="true" />
              Rust engine
            </li>
            <li>
              <CircleCheckBig aria-hidden="true" />
              Browser WASM
            </li>
            <li>
              <CircleCheckBig aria-hidden="true" />
              Typed evidence
            </li>
          </ul>
        </div>

        <div className="omena-hero-console" aria-label="Example Omena finding">
          <div className="omena-console-chrome">
            <span />
            <span />
            <span />
            <strong>omena lint</strong>
          </div>
          <div className="omena-console-source">
            <span className="omena-line-number">1</span>
            <code>
              <b>.button</b> {"{"}
            </code>
            <span className="omena-line-number">2</span>
            <code>
              &nbsp;&nbsp;animation: <mark>enter</mark> 180ms ease-out;
            </code>
            <span className="omena-line-number">3</span>
            <code>{"}"}</code>
          </div>
          <div className="omena-console-finding">
            <span>missing-keyframes</span>
            <p>
              <strong>enter</strong> is referenced, but no matching @keyframes is visible in this
              workspace.
            </p>
            <small>Evidence: declaration → animation-name → unresolved definition</small>
          </div>
        </div>
      </section>

      <section className="omena-capability-grid" aria-labelledby="capability-heading">
        <div className="omena-section-heading">
          <p>Built for accountable automation</p>
          <h2 id="capability-heading">A semantic spine, not another collection of wrappers.</h2>
        </div>
        <div className="omena-capability-cards">
          {capabilities.map(({ icon: Icon, title, body }, index) => (
            <article key={title} style={{ "--card-index": index } as React.CSSProperties}>
              <Icon aria-hidden="true" />
              <h3>{title}</h3>
              <p>{body}</p>
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}
