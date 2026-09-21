export interface PnpmRunCommand {
  readonly executable: string;
  readonly args: readonly string[];
  readonly display: readonly string[];
}

interface PnpmRunCommandOptions {
  readonly platform?: NodeJS.Platform;
  readonly env?: NodeJS.ProcessEnv;
  readonly nodeExecutable?: string;
}

export function pnpmRunCommand(
  scriptName: string,
  extraArgs: readonly string[] = [],
  options: PnpmRunCommandOptions = {},
): PnpmRunCommand {
  const platform = options.platform ?? process.platform;
  const env = options.env ?? process.env;
  const nodeExecutable = options.nodeExecutable ?? process.execPath;
  const pnpmArgs = ["run", scriptName, ...(extraArgs.length > 0 ? ["--", ...extraArgs] : [])];
  const display = ["pnpm", ...pnpmArgs];

  const cliPath = env.npm_execpath;
  const windowsShim = platform === "win32" && /\.(?:cmd|bat)$/iu.test(cliPath ?? "");
  if (cliPath && !windowsShim) {
    // Standalone pnpm distributions are native executables, not Node scripts.
    const nodeScript = /\.(?:cjs|mjs|js)$/iu.test(cliPath);
    return {
      executable: nodeScript ? nodeExecutable : cliPath,
      args: nodeScript ? [cliPath, ...pnpmArgs] : pnpmArgs,
      display,
    };
  }

  if (platform === "win32") {
    return {
      executable: env.ComSpec ?? "cmd.exe",
      args: ["/d", "/s", "/c", "pnpm", ...pnpmArgs],
      display,
    };
  }

  return {
    executable: "pnpm",
    args: pnpmArgs,
    display,
  };
}
