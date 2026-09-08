import _ from "lodash";

/**
 * Turns a cover path into something an `<img>` can load.
 *
 * A remote provider hands back absolute, already-authenticated urls; the
 * daemon's own covers arrive as `/covers/<file>` and are served from the same
 * origin. Both cases reduce to "strip the prefix if what is underneath is
 * already a url".
 */
export const resourceUriResolver = {
  resolve(path: string | undefined): string | undefined {
    if (!path) return path;
    const withoutPrefix = _.replace(path, /^\/covers\//, "");
    return _.startsWith(withoutPrefix, "http") ? withoutPrefix : path;
  },
};
