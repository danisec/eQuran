#!/usr/bin/env node
// Builds the SvelteKit frontend for a Tauri bundle.
//
// The main `tauri.conf.json` finds the frontend with a POSIX shell loop
// (`sh -c 'while [ ! -f ... ]'`), which assumes a shell that Windows does not
// ship. This script does the same job in Node, which is always present because
// the frontend itself is a Node project.
//
// The repository root is derived from this file's own location rather than the
// working directory: Tauri does not run `beforeBuildCommand` from a stable cwd,
// and on Windows it ran one level too high, so a relative path missed the tree.
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// This file lives at <repo>/scripts/build-frontend.mjs.
const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(scriptDir);
const frontend = join(repoRoot, "frontend");

if (!existsSync(join(frontend, "package.json"))) {
  console.error(
    `build-frontend: no frontend/package.json at ${frontend}. ` +
      `The script is at ${scriptDir}; is the checkout complete?`,
  );
  process.exit(1);
}

// `shell: true` is what makes this work on Windows, where npm is `npm.cmd`.
const result = spawnSync("npm", ["run", "build"], {
  cwd: frontend,
  stdio: "inherit",
  shell: true,
});

if (result.error) {
  console.error("build-frontend: could not run npm:", result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 1);
