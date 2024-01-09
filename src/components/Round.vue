<script setup lang="ts">
import { computed, ref } from "vue";
import { router } from "../router";
import { RoundResponse } from "../types";
import FolderWatcher from "./FolderWatcher.vue";
import {
  add_to_queue,
  delayDisplay,
  isSingleGamePgn,
  lichessFetch,
  openPath,
  recursiveFileList,
  relativeTimeDisplay,
  timestampToLocalDatetime,
} from "../utils";
import { useLogStore } from "../stores/logs";

const logs = useLogStore();

const round = ref<RoundResponse | null>(null);

const delay = computed<string>(() => {
  return delayDisplay(round.value?.round.delay);
});

const watchedFolder = computed<string>(() => {
  if (!round.value) {
    return "";
  }

  return logs.watchProcesses.get(round.value.round.id)?.folder ?? "";
});

const relativeTime = computed<string>(() => {
  return relativeTimeDisplay(round.value?.round.startsAt);
});

const startsAt = computed<string>(() => {
  return timestampToLocalDatetime(round.value?.round.startsAt);
});

function stopWatching() {
  logs.watchProcesses.get(round.value!.round.id)?.unlisten();
  logs.watchProcesses.delete(round.value!.round.id);
}

function getRound() {
  lichessFetch(`/api/broadcast/-/-/${router.currentRoute.value.params.id}`)
    .then((response) => response.json() as Promise<RoundResponse>)
    .then((data) => {
      round.value = data;
    });
}

async function uploadExistingFilesInFolder() {
  let files = await recursiveFileList(watchedFolder.value!);
  files = files.filter((file) => isSingleGamePgn(file));

  await add_to_queue(round.value!.round.id, files);

  router.push("/");
}

getRound();
</script>

<template>
  <template v-if="round">
    <div class="md:flex md:items-center md:justify-between mb-4">
      <div class="min-w-0 flex-1">
        <h2
          class="text-2xl font-bold leading-7 text-white sm:truncate sm:text-3xl sm:tracking-tight"
        >
          {{ round.round.name }}
          <span class="ml-2 text-gray-400">/ {{ round.tour.name }}</span>
        </h2>
        <p class="text-gray-200 text-xl">{{ round.tour.description }}</p>
        <div
          v-if="startsAt"
          class="mt-3 flex items-center gap-x-2.5 text-xs leading-5 text-gray-400"
        >
          <p class="truncate">{{ startsAt }}</p>
          <svg viewBox="0 0 2 2" class="h-0.5 w-0.5 flex-none fill-gray-300">
            <circle cx="1" cy="1" r="1" />
          </svg>
          <p class="whitespace-nowrap">{{ relativeTime }}</p>
          <template v-if="delay">
            <svg viewBox="0 0 2 2" class="h-0.5 w-0.5 flex-none fill-gray-300">
              <circle cx="1" cy="1" r="1" />
            </svg>
            <p class="whitespace-nowrap">Move Delay: {{ delay }}</p>
          </template>
        </div>
      </div>
      <div class="mt-4 flex md:ml-4 md:mt-0 space-x-1">
        <button
          type="button"
          @click="getRound"
          class="inline-flex items-center rounded-md bg-white/10 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-white/20"
        >
          Refresh
        </button>
        <button
          type="button"
          @click="openPath(round.round.url)"
          class="inline-flex items-center rounded-md bg-white/10 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-white/20"
        >
          View on Lichess
        </button>
      </div>
    </div>

    <div v-if="watchedFolder">
      <div class="border-l-8 border-green-900 bg-green-800 text-gray-50 p-4">
        <div class="flex">
          <div class="ml-3">
            <p class="font-bold">This round is currently being broadcasted.</p>
            <p>
              Currently watching folder:
              <a
                class="underline"
                href=""
                @click.prevent="openPath(watchedFolder)"
                >{{ watchedFolder }}</a
              >
            </p>

            <div class="mt-4">
              <button
                type="button"
                @click="stopWatching"
                class="rounded-md bg-gray-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-gray-400"
              >
                Stop watching this folder
              </button>
            </div>

            <div
              class="mt-4 pt-2 border-t-2 border-t-green-900 text-green-100 text-sm"
            >
              If there are already ongoing games and PGN files in this directory
              that you want to bulk upload, click here:
              <a
                class="underline"
                href=""
                @click.prevent="uploadExistingFilesInFolder"
              >
                Upload PGN now
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="border-gray-700 border-2 p-6 my-4">
      <h3 class="text-white text-xl mb-2">
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
            d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5"
          />
        </svg>
        Upload PGN
      </h3>
      <p class="text-gray-300">
        As game PGNs are written to a folder, they can be automatically be
        uploaded to Lichess.
      </p>

      <FolderWatcher v-if="round.study.writeable" :round-id="round.round.id" />
      <div v-else class="mt-4 bg-yellow-50 text-yellow-800 py-4 px-4">
        <svg
          class="inline h-5 w-5"
          viewBox="0 0 20 20"
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fill-rule="evenodd"
            d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
            clip-rule="evenodd"
          />
        </svg>
        You do not have Contributor access to this study so you can't upload
        PGN.
      </div>
    </div>

    <h3 class="text-white text-xl my-4">
      Games
      <span class="text-gray-400">({{ round.games.length }})</span>
    </h3>
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
      <a
        v-for="game in round.games"
        href="#"
        @click="openPath(game.url)"
        class="bg-gray-700 text-gray-100 hover:bg-gray-600 py-2 px-4"
        :class="{ 'bg-cyan-900 hover:bg-cyan-800': game.ongoing }"
      >
        {{ game.name }}
        <span class="float-right">{{ game.res }}</span>
      </a>
    </div>
  </template>
</template>
