#!/usr/bin/env deno run --allow-read --allow-write

import { ensureDir, walk } from "jsr:@std/fs";
import { basename, dirname, join } from "jsr:@std/path";

// Base directory for mocks
const MOCKS_DIR = new URL("../", import.meta.url).pathname;
const ASSETS_DIR = join(MOCKS_DIR, "api.curseforge.com/assets");

// Regular expression to match URLs
const URL_REGEX =
  /https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)/g;

// Function to create asset file and return local URL
async function handleDownloadUrl(url: string): Promise<string> {
  // Extract filename from URL
  const filename = basename(url);

  // Create asset file with filename as content
  const assetPath = join(ASSETS_DIR, filename);

  await ensureDir(dirname(assetPath));
  await Deno.writeTextFile(assetPath, filename);

  // Return local URL
  return `http://127.0.0.1:25569/api.curseforge.com/assets/${filename}`;
}

// Function to process JSON files
async function processJsonFile(filePath: string) {
  console.log(`Processing: ${filePath}`);

  try {
    // Read file content
    const content = await Deno.readTextFile(filePath);

    // Parse JSON
    const data = JSON.parse(content);

    // Flag to track if changes were made
    let modified = false;

    // Function to recursively process object properties
    // deno-lint-ignore no-inner-declarations
    async function processObject(obj: unknown): Promise<unknown> {
      if (!obj || typeof obj !== "object") return obj;

      if (Array.isArray(obj)) {
        return Promise.all(obj.map((item) => processObject(item)));
      }

      const result: Record<string, unknown> = {};

      for (const [key, value] of Object.entries(obj)) {
        if (typeof value === "string" && value.match(URL_REGEX)) {
          modified = true;

          if (
            key === "downloadUrl" ||
            key === "data" && filePath.includes("download-url")
          ) {
            result[key] = await handleDownloadUrl(value);
          } else {
            result[key] = "https://example.com";
          }
        } else if (value && typeof value === "object") {
          result[key] = await processObject(value);
        } else {
          result[key] = value;
        }
      }

      return result;
    }

    // Process the JSON data
    const processedData = await processObject(data);

    // Write back to file if modified
    if (modified) {
      await Deno.writeTextFile(
        filePath,
        JSON.stringify(processedData, null, 4),
      );
      console.log(`Modified: ${filePath}`);
    } else {
      console.log(`No URLs found in: ${filePath}`);
    }
  } catch (error) {
    // deno-lint-ignore ban-ts-comment
    // @ts-expect-error
    console.error(`Error processing ${filePath}: ${error.message}`);
  }
}

// Main function
async function main() {
  console.log("Starting URL replacement process...");

  // Ensure assets directory exists
  await ensureDir(ASSETS_DIR);

  // Find all JSON files in the mocks directory
  for await (
    const entry of walk(MOCKS_DIR, {
      exts: ["json"],
      skip: [/node_modules/, /target/],
    })
  ) {
    if (entry.isFile) {
      await processJsonFile(entry.path);
    }
  }

  console.log("URL replacement completed!");
}

// Run the script
main().catch((err) => console.error("Error:", err));
