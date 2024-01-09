import { invoke } from "@tauri-apps/api";
import { FileEntry, readDir } from "@tauri-apps/api/fs";
import { useLogStore } from "./stores/logs";
import { useSettingsStore } from "./stores/settings";
import { useUserStore } from "./stores/user";

export async function lichessFetch(
  path: string,
  options?: object,
  timeoutMs = 5_000,
): Promise<Response> {
  const settings = useSettingsStore();
  const user = useUserStore();

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  const url = `${settings.lichessUrl}${path}`;
  return fetch(url, {
    headers: new Headers({
      Authorization: `Bearer ${user.accessToken?.access_token}`,
      // "User-Agent": `${system.uaPrefix()} as:${user.username?.toLowerCase() ?? "anon"}`,
    }),
    signal: controller.signal,
    ...options,
  })
    .then((response) => {
      clearTimeout(timeout);
      if (!response.ok) handleFetchError(url, response);
      return response;
    })
    .catch((error: TypeError) => {
      const logs = useLogStore();
      logs.error(`Error: ${error.name} ${error.message} - ${url}`);
      throw error;
    });
}

function handleFetchError(url: string, response: Response) {
  const logs = useLogStore();

  logs.error(
    response.status === 401
      ? "Error: Invalid/expired session. Please log out of this app on the Settings page and then log back in."
      : `Error: ${response.status} ${response.statusText} - ${url}`,
  );

  throw new Error(`${response.status} ${response.statusText}`);
}

export async function openPath(path: string) {
  await invoke("open_path", { path });
}

export function timestampToLocalDatetime(timestamp?: number) {
  if (!timestamp) {
    return "";
  }

  const date = new Date(timestamp);

  return date.toLocaleDateString(undefined, {
    weekday: "long",
    year: "2-digit",
    month: "numeric",
    day: "numeric",
    hour: "numeric",
    minute: "numeric",
  });
}

export function relativeTimeDisplay(timestamp?: number) {
  if (!timestamp) {
    return "";
  }

  const then = new Date(timestamp).getTime();
  const deltaSeconds = Math.round((then - Date.now()) / 1000);

  const cutoffs = [
    60,
    3600,
    86400,
    86400 * 7,
    86400 * 30,
    86400 * 365,
    Infinity,
  ];
  const units: Intl.RelativeTimeFormatUnit[] = [
    "second",
    "minute",
    "hour",
    "day",
    "week",
    "month",
    "year",
  ];
  const unitIndex = cutoffs.findIndex(
    (cutoff) => cutoff > Math.abs(deltaSeconds),
  );
  const divisor = unitIndex ? cutoffs[unitIndex - 1] : 1;
  const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
  return rtf.format(Math.floor(deltaSeconds / divisor), units[unitIndex]);
}

export function delayDisplay(delay?: number): string {
  if (!delay) {
    return "";
  }

  return [
    {
      value: Math.floor(delay / 60),
      label: "minute",
    },
    {
      value: delay % 60,
      label: "second",
    },
  ]
    .filter(({ value }) => value > 0)
    .map(({ value, label }) => {
      return `${value} ${label}${value > 1 ? "s" : ""}`;
    })
    .join(" ");
}

/**
 * Ignore the "games.pgn" file which is a multi-game pgn file.
 * We only want to upload single game pgn files (game-1.pgn, game-2.pgn, etc.)
 */
export function isSingleGamePgn(path: string): boolean {
  return path.endsWith(".pgn") && !path.endsWith("games.pgn");
}

export async function recursiveFileList(folder: string): Promise<string[]> {
  const files: string[] = [];
  const entries = await readDir(folder, { recursive: true });

  const traverse = (entries: FileEntry[]) => {
    for (const entry of entries) {
      if (entry.children) {
        traverse(entry.children);
        continue;
      }
      files.push(entry.path);
    }
  };

  traverse(entries);

  return files.reverse();
}

/**
 * Add a PGN push to the queue, which will be handled in Rust by the backend.
 */
export async function add_to_queue(roundId: string, files: string[]) {
  const settings = useSettingsStore();
  const user = useUserStore();

  const url = `${settings.lichessUrl}/api/broadcast/round/${roundId}/push`;

  await invoke("add_to_queue", {
    apiToken: user.accessToken?.access_token,
    url,
    files,
  });
}
