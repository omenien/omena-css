import fs from "node:fs";

const DEFAULT_STABLE_MACOS_DEVELOPER_DIR = "/Applications/Xcode.app/Contents/Developer";

export function nativeRustBuildEnv(
  additionalEnv: Readonly<NodeJS.ProcessEnv> = {},
): NodeJS.ProcessEnv {
  const env = { ...process.env, ...additionalEnv };
  if (process.platform !== "darwin") return env;

  const configuredDeveloperDir = env.OMENA_MACOS_NATIVE_DEVELOPER_DIR?.trim();
  if (configuredDeveloperDir && !fs.existsSync(configuredDeveloperDir)) {
    throw new Error(`OMENA_MACOS_NATIVE_DEVELOPER_DIR does not exist: ${configuredDeveloperDir}`);
  }

  const stableDeveloperDir = configuredDeveloperDir || DEFAULT_STABLE_MACOS_DEVELOPER_DIR;
  if (fs.existsSync(stableDeveloperDir)) {
    env.DEVELOPER_DIR = stableDeveloperDir;
  }
  return env;
}
