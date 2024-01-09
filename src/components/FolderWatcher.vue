<script setup lang="ts">
import { open } from "@tauri-apps/api/dialog";
import { DebouncedEvent, watch } from "tauri-plugin-fs-watch-api";
import { useLogStore } from "../stores/logs";
import { add_to_queue, isSingleGamePgn } from "../utils";

const logs = useLogStore();

const props = defineProps<{
  roundId: string;
}>();

async function selectPgnFolder() {
  open({ directory: true }).then((selected) => {
    if (selected === null) {
      return;
    }

    if (typeof selected !== "string") {
      throw new Error("Expected a single folder to be selected");
    }

    startWatchingFolder(selected);
  });
}

async function startWatchingFolder(path: string) {
  const stopWatching = await watch(path, handleFolderChange, {
    recursive: true,
    delayMs: 1000,
  });

  logs.watchProcesses.set(props.roundId, {
    folder: path,
    unlisten: stopWatching,
  });
}

async function handleFolderChange(events: DebouncedEvent) {
  const files = events
    .filter((event) => event.kind === "Any")
    .filter((event) => isSingleGamePgn(event.path))
    .map((event) => event.path);

  if (files.length === 0) {
    return;
  }

  await add_to_queue(props.roundId, files);

  const paths = files.map((file) => file.split("/").pop());
  logs.info(`Modified: ${paths.join(", ")}`);
}
</script>

<template>
  <form class="mt-2">
    <button
      @click="selectPgnFolder"
      class="rounded-md bg-teal-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-teal-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-teal-500"
      type="button"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="inline-block w-4 h-4"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12 10.5v6m3-3H9m4.06-7.19l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z"
        />
      </svg>
      Select Folder
    </button>
  </form>
</template>
