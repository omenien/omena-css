import { metaSchema, pageSchema } from "fumadocs-core/source/schema";
import { defineConfig, defineDocs } from "fumadocs-mdx/config";
import { z } from "zod";

const documentationKinds = ["tutorial", "how-to", "reference", "explanation"] as const;
const documentationStatuses = ["stable", "preview", "experimental", "deprecated"] as const;
const sourceKinds = ["authored", "generated", "hybrid"] as const;

export const docs = defineDocs({
  dir: "../../docs",
  docs: {
    schema: pageSchema.extend({
      kind: z.enum(documentationKinds),
      status: z.enum(documentationStatuses),
      products: z.array(z.string()).min(1),
      owner: z.string().min(1),
      sourceOfTruth: z.enum(sourceKinds),
    }),
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
  meta: {
    schema: metaSchema,
  },
});

export default defineConfig({
  mdxOptions: {},
});
