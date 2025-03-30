#!/usr/bin/env -S deno run -A

import { join } from "jsr:@std/path";
import { exists } from "jsr:@std/fs";
import $ from "jsr:@david/dax";

/**
 * Represents a mock API host.
 */
enum Host {
  Curseforge = "api.curseforge.com",
  Modrinth = "api.modrinth.com", // Not implemented yet
}

/**
 * Setup mock server for testing.
 */
async function setupMockServer(): Promise<
  { server: Deno.HttpServer; port: number }
> {
  const port = 25569;
  const controller = new AbortController();
  const { signal } = controller;

  console.log(`🔧 Setting up mock server on port ${port}...`);

  // Get the current directory
  const currentDir = Deno.cwd();
  const mocksDir = join(currentDir, "tests", "mocks");

  // Check if mocks directory exists
  if (!await exists(mocksDir)) {
    throw new Error(`Mock directory does not exist: ${mocksDir}`);
  }

  // Build a map of endpoints to mock files
  const endpoints = new Map<string, { path: string; contentType: string }>();

  // Process Curseforge mocks
  await processMockFiles(Host.Curseforge, mocksDir, endpoints);

  // Set up server
  const server = Deno.serve({ port, signal }, async (request) => {
    const url = new URL(request.url);
    // パスのみ抽出し、クエリパラメータを除去
    const path = url.pathname;

    console.log(`📨 Received request: ${request.method} ${path}`);

    // まずパスが完全に一致するエンドポイントを探す
    let mockData = endpoints.get(path);

    // 完全一致しない場合、クエリパラメータを含むパスがあるか確認
    if (!mockData) {
      // クエリパラメータを含むパスを探す（検索パスをパスの先頭部分として含むキー）
      for (const [endpointPath, data] of endpoints.entries()) {
        // endpointPath がクエリパラメータを含む場合、パス部分だけで比較
        const endpointPathWithoutQuery = endpointPath.split("?")[0];
        if (endpointPathWithoutQuery === path) {
          console.log(
            `🔍 Found mock with query parameters: ${endpointPath} for path: ${path}`,
          );
          mockData = data;
          break;
        }
      }
    }

    if (!mockData) {
      console.error(`❌ No mock found for endpoint: ${path}`);
      return new Response("Not Found", { status: 404 });
    }

    try {
      const fileContent = await Deno.readFile(mockData.path);
      return new Response(fileContent, {
        status: 200,
        headers: {
          "Content-Type": mockData.contentType,
        },
      });
    } catch (error) {
      console.error(`❌ Error serving mock for ${path}:`, error);
      return new Response("Internal Server Error", { status: 500 });
    }
  });

  console.log(`✅ Mock server running at http://localhost:${port}`);

  return { server, port };
}

/**
 * Process mock files for a specific host and add them to endpoints map.
 */
async function processMockFiles(
  host: Host,
  mocksDir: string,
  endpoints: Map<string, { path: string; contentType: string }>,
): Promise<void> {
  const hostDir = join(mocksDir, host);

  if (!await exists(hostDir)) {
    console.warn(
      `⚠️ Mock directory for ${host} does not exist, skipping...`,
    );
    return;
  }

  console.log(`📁 Processing mock files for ${host}...`);

  for await (const entry of Deno.readDir(hostDir)) {
    if (!entry.isFile && !entry.isDirectory) continue;

    if (entry.isDirectory) {
      await processDirectory(
        join(hostDir, entry.name),
        mocksDir,
        endpoints,
      );
    } else {
      processFile(join(hostDir, entry.name), mocksDir, endpoints);
    }
  }
}

/**
 * Process a directory recursively to find mock files.
 */
async function processDirectory(
  dirPath: string,
  mocksDir: string,
  endpoints: Map<string, { path: string; contentType: string }>,
): Promise<void> {
  for await (const entry of Deno.readDir(dirPath)) {
    const entryPath = join(dirPath, entry.name);

    if (entry.isDirectory) {
      await processDirectory(entryPath, mocksDir, endpoints);
    } else if (entry.isFile) {
      processFile(entryPath, mocksDir, endpoints);
    }
  }
}

/**
 * Process a single mock file and add it to endpoints map.
 */
function processFile(
  filePath: string,
  mocksDir: string,
  endpoints: Map<string, { path: string; contentType: string }>,
): void {
  const relativePath = filePath.substring(mocksDir.length);
  const parts = relativePath.split("/");

  // Check if this is an index file
  let urlPath: string;
  if (parts[parts.length - 1].startsWith("index.")) {
    // For index files, use the directory path
    urlPath = parts.slice(0, parts.length - 1).join("/");
  } else {
    // For non-index files, use the full path WITH extension
    urlPath = relativePath;
  }

  // Ensure the URL path starts with a slash
  if (!urlPath.startsWith("/")) {
    urlPath = "/" + urlPath;
  }

  // Determine content type
  const extension = filePath.split(".").pop()?.toLowerCase() || "";
  let contentType: string;
  switch (extension) {
    case "json":
      contentType = "application/json; charset=utf-8";
      break;
    case "txt":
      contentType = "text/plain; charset=utf-8";
      break;
    case "jar":
      contentType = "application/java-archive";
      break;
    case "xml":
      contentType = "application/xml; charset=utf-8";
      break;
    case "html":
      contentType = "text/html; charset=utf-8";
      break;
    default:
      contentType = "application/octet-stream";
  }

  console.log(`📄 Adding mock for endpoint: ${urlPath} -> ${relativePath}`);
  endpoints.set(urlPath, { path: filePath, contentType });
}

/**
 * Run cargo tests with the mock server.
 */
async function runTests(): Promise<number> {
  try {
    // Setup the mock server
    const { server, port } = await setupMockServer();

    try {
      // Set environment variable to tell tests to use the mock server
      console.log("🧪 Running cargo tests with mock server...");
      const process = $`MOCK_SERVER_URL=http://localhost:${port} cargo test`;
      await process;
      return 0; // 正常終了の場合は0を返す
    } catch (error) {
      console.error("❌ Test execution failed:", error);
      return 1; // エラーの場合は1を返す
    } finally {
      // Ensure the server is shut down
      console.log("🛑 Shutting down mock server...");
      server.shutdown();
    }
  } catch (error) {
    console.error("❌ Error:", error);
    return 1;
  }
}

// Run the tests and exit with the appropriate status code
const exitCode = await runTests();
Deno.exit(exitCode);
