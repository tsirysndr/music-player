/** @jsx h */
import { serve } from "https://deno.land/std@0.190.0/http/server.ts";
import html, { h } from "https://deno.land/x/htm@0.2.1/mod.ts";

const handler = (req: Request) =>
  html({
    title: "Hello World!",
    styles: [
      "html, body { margin: 0; height: 100%; }",
      "body { background: #86efac; display: flex; flex-direction: column; align-items: center; justify-content: center; }",
    ],
    body: (
      <body>
        <img width="64" src="https://dash.deno.com/assets/logo.svg" />
        <h1>Hello Patricia!</h1>
      </body>
    ),
  });

serve(handler);
