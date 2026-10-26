export const inspectError = error => {
  const errors = Array.isArray(error.errors) ? error.errors.map(e => ({
    message: String(e.message ?? "GraphQL error"), path: e.path ?? null, extensions: e.extensions ?? null,
  })) : [];
  const execution = errors.find(e => e.extensions?._type === "EXEC_ERROR");
  const ext = execution?.extensions;
  return {
    message: String(error.message ?? error), errors,
    exec: ext ? {
      message: execution.message,
      cmd: Array.isArray(ext.cmd) ? ext.cmd.filter(x => typeof x === "string") : [],
      exitCode: Number.isInteger(ext.exitCode) ? ext.exitCode : -1,
      stdout: typeof ext.stdout === "string" ? ext.stdout : "",
      stderr: typeof ext.stderr === "string" ? ext.stderr : "",
    } : null,
  };
};
