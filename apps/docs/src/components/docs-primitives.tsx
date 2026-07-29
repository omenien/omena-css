interface SupportMatrixRow {
  availability: string;
  channel: string;
  surface: string;
  version: string;
}

export function SupportMatrix({ rows }: { rows: SupportMatrixRow[] }) {
  return (
    <div className="not-prose my-6 overflow-x-auto rounded-lg border border-fd-border">
      <table className="w-full min-w-[36rem] border-collapse text-sm">
        <caption className="sr-only">Currently published Omena product surfaces</caption>
        <thead className="bg-fd-secondary text-left text-fd-muted-foreground">
          <tr>
            <th className="px-4 py-2.5 font-medium">Surface</th>
            <th className="px-4 py-2.5 font-medium">Version</th>
            <th className="px-4 py-2.5 font-medium">Channel</th>
            <th className="px-4 py-2.5 font-medium">Availability</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-fd-border">
          {rows.map((row) => (
            <tr key={row.surface}>
              <th className="px-4 py-3 text-left font-medium text-fd-foreground">{row.surface}</th>
              <td className="px-4 py-3 font-mono text-xs">{row.version}</td>
              <td className="px-4 py-3 text-fd-muted-foreground">{row.channel}</td>
              <td className="px-4 py-3 text-fd-muted-foreground">{row.availability}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
