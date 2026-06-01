import { describe, expect, it } from "vitest";
import {
  buildTypeFactBackendEnv,
  readClientTypeFactBackendSetting,
} from "../../../client/src/type-fact-backend-config";

describe("client type-fact backend config", () => {
  it("defaults invalid setting values to bundled tsgo", () => {
    expect(readClientTypeFactBackendSetting("future")).toBe("tsgo");
    expect(readClientTypeFactBackendSetting(undefined)).toBe("tsgo");
  });

  it("maps bundled tsgo to the server env without workspace resolution", () => {
    expect(
      buildTypeFactBackendEnv("tsgo", {
        OMENA_TSGO_RESOLUTION: "workspace",
      }),
    ).toMatchObject({
      OMENA_TYPE_FACT_BACKEND: "tsgo",
    });
    expect(
      buildTypeFactBackendEnv("tsgo", { OMENA_TSGO_RESOLUTION: "workspace" }),
    ).not.toHaveProperty("OMENA_TSGO_RESOLUTION");
  });

  it("keeps workspace tsgo as an explicit power-user mode", () => {
    expect(buildTypeFactBackendEnv("tsgo-workspace", {})).toMatchObject({
      OMENA_TYPE_FACT_BACKEND: "tsgo",
      OMENA_TSGO_RESOLUTION: "workspace",
    });
  });
});
