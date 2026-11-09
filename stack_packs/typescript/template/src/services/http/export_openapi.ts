import { readFile, writeFile } from "node:fs/promises";

import { HttpService } from "#src/services/http/http_service.ts";

// The service's contract: every route, with the wire models it takes and answers with, as the OpenAPI document
// zod-openapi builds from the routers. It is kept in the project so that a change to the API shows up as a change to
// this file, and so that whatever calls the API can check itself against it.
const CONTRACT_PATH: string = "openapi.json";

// Writes the contract (`pnpm run export-openapi`), or with `--check` fails when the file is out of date.
async function main(): Promise<number> {
  // Sorted keys, so the file only changes when the contract does.
  const contract: string = JSON.stringify(sortKeys(HttpService.getContract()), null, 2) + "\n";

  if (!process.argv.slice(2).includes("--check")) {
    await writeFile(CONTRACT_PATH, contract, "utf-8");
    return 0;
  }
  const currentContract: string | undefined = await readFile(CONTRACT_PATH, "utf-8").catch((): undefined => undefined);
  if (currentContract !== contract) {
    console.error(`${CONTRACT_PATH} is out of date: run \`pnpm run export-openapi\``);
    return 1;
  }
  return 0;
}

function sortKeys(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map((item: unknown): unknown => sortKeys(item));
  }
  if (typeof value === "object" && value !== null) {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([left]: [string, unknown], [right]: [string, unknown]): number =>
          left < right ? -1 : left > right ? 1 : 0,
        )
        .map(([key, item]: [string, unknown]): [string, unknown] => [key, sortKeys(item)]),
    );
  }
  return value;
}

process.exitCode = await main();
