import { docs } from "collections/server";
import { loader } from "fumadocs-core/source";
import { site } from "./site";

export const source = loader({
  baseUrl: site.docsBaseUrl,
  source: docs.toFumadocsSource(),
});
