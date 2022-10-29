import { useState, useEffect } from "react";

export const useCover = (coverUrl?: string) => {
  const [cover, setCover] = useState<string | undefined>(undefined);

  const verifyCover = async (url: string) => {
    const res = await fetch(url);
    if (
      res.status === 200 &&
      res.headers.get("Content-Type") === "image/jpeg"
    ) {
      return url;
    }
    return undefined;
  };

  useEffect(() => {
    if (coverUrl) {
      fetch(coverUrl)
        .then((res) => {
          if (
            res.status === 200 &&
            res.headers.get("Content-Type") === "image/jpeg"
          ) {
            setCover(coverUrl);
          }
        })
        .catch(() => console.log("Failed to fetch Album Cover"));
    }
  }, [coverUrl]);
  return { cover, verifyCover };
};
