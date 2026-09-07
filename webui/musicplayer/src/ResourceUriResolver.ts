import _ from "lodash";

let pathSep = "/";
let coversDir = "";

export const resourceUriResolver = {
  async initializeForNativeWrapper(): Promise<void> {
    if (import.meta.env.VITE_NATIVE_WRAPPER !== "tauri") return;
    const { path } = window.__TAURI__;
    const appDataDir = await path.appDataDir();
    coversDir = await path.join(appDataDir, "covers");
    // In Tauri v2, `path.sep` is a function returning the separator
    pathSep = path.sep();
  },
  resolve(path: string | undefined): string | undefined {
    if (!path) return path;
    if (import.meta.env.VITE_NATIVE_WRAPPER !== "tauri") {
      if (_.startsWith(_.replace(path, /^\/covers\//, ""), "http")) {
        return _.replace(path, /^\/covers\//, "");
      }
      return path;
    }
    if (_.startsWith(_.replace(path, /^\/covers\//, ""), "http")) {
      return _.replace(path, /^\/covers\//, "");
    }
    // Image file in covers/ data directory
    if (/^\/covers\/[^<>:;,?"*|/]+\.(?:jpg|png)$/.test(path)) {
      const { core } = window.__TAURI__;
      const devicePath = [coversDir, path.substring(8)].join(pathSep);
      return core.convertFileSrc(devicePath);
    }
    return path;
  },
};
