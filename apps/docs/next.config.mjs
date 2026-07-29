/* oxlint-disable import/no-default-export -- Next.js loads this configuration through its default export. */
import { createMDX } from "fumadocs-mdx/next";

const withMDX = createMDX();
const basePath = process.env.OMENA_DOCS_BASE_PATH ?? "";

/** @type {import("next").NextConfig} */
const config = {
  output: "export",
  reactStrictMode: true,
  trailingSlash: true,
  basePath,
  assetPrefix: basePath || undefined,
  env: {
    NEXT_PUBLIC_OMENA_DOCS_BASE_PATH: basePath,
  },
  images: {
    unoptimized: true,
  },
};

export default withMDX(config);
