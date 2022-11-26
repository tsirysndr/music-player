let pathSep = '/';
let coversDir = '';

export const resourceUriResolver = {
  async initializeForNativeWrapper(): Promise<void> {
    if (process.env.REACT_APP_NATIVE_WRAPPER !== "tauri") return;
    const { path } = window.__TAURI__;
    const appDataDir = await path.appDataDir();
    coversDir = await path.join(appDataDir, "covers");
    pathSep = path.sep;
  },
  resolve(path: string | undefined): string | undefined {
    if (!path) return path;
    if (process.env.REACT_APP_NATIVE_WRAPPER !== "tauri") return path;
    // Image file in covers/ data directory
    if (/^\/covers\/[^<>:;,?"*|/]+.(?:jpg|png)$/.test(path)) {
      const { tauri } = window.__TAURI__;
      const devicePath = [coversDir, path.substring(8)].join(pathSep);
      return tauri.convertFileSrc(devicePath);
    }
    return path;
  }
};
