export interface PnpmCliCommand {
  readonly executable: string;
  readonly args: readonly string[];
}

interface PnpmCliCommandOptions {
  readonly platform?: NodeJS.Platform;
  readonly env?: NodeJS.ProcessEnv;
  readonly nodeExecutable?: string;
}

export function pnpmCliCommand(
  args: readonly string[],
  options: PnpmCliCommandOptions = {},
): PnpmCliCommand {
  const platform = options.platform ?? process.platform;
  const env = options.env ?? process.env;

  if (env.npm_execpath) {
    return {
      executable: options.nodeExecutable ?? process.execPath,
      args: [env.npm_execpath, ...args],
    };
  }

  if (platform === "win32") {
    return {
      executable: env.ComSpec ?? "cmd.exe",
      args: ["/d", "/s", "/c", "pnpm", ...args],
    };
  }

  return { executable: "pnpm", args };
}
