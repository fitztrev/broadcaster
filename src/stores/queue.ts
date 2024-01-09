import { defineStore } from "pinia";
import { ref } from "vue";

interface UploadQueue {
  roundId: string;
  path: string;
}

export const useQueueStore = defineStore("queue", () => {
  const files = ref<Set<string>>(new Set());
  const heartbeat = ref<number>(0);

  const add = (roundId: string, paths: string[]) => {
    for (const path of paths) {
      files.value.add(JSON.stringify({ roundId, path }));
    }
  };

  const remove = (item: UploadQueue) => {
    files.value.delete(JSON.stringify(item));
  };

  const next = (): UploadQueue | undefined => {
    const next = files.value.values().next().value;

    if (next) {
      return JSON.parse(next) as UploadQueue;
    }

    return undefined;
  };

  const ping = () => {
    heartbeat.value = Date.now();
  }

  return {
    files,
    add,
    remove,
    next,
    ping,
  };
});
